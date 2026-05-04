use crate::config::{load, SourceKind};
use crate::paths::OcvmPaths;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct RemoteVersion {
    pub version: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(alias = "tarball", alias = "npm")]
    pub url: Option<String>,
    #[serde(default, alias = "sha256", alias = "executableSha256")]
    pub executable_sha256: Option<String>,
}

pub trait SourceProvider {
    fn resolve_alias(&self, requested: &str) -> Result<String>;
    fn list_remote(&self, channel: Option<&str>) -> Result<Vec<RemoteVersion>>;
    fn install(&self, version: &str, staging_dir: &Path) -> Result<()>;
    fn verify_staged_install(&self, _version: &str, _staging_dir: &Path) -> Result<()> {
        Ok(())
    }
}

pub struct NpmSource {
    package: String,
}

impl NpmSource {
    pub fn from_env() -> Self {
        Self {
            package: std::env::var("OCVM_NPM_PACKAGE").unwrap_or_else(|_| "openclaw".to_string()),
        }
    }

    fn npm_json(&self, args: &[&str]) -> Result<serde_json::Value> {
        let output = Command::new("npm")
            .args(args)
            .output()
            .with_context(|| format!("failed to run npm {}", args.join(" ")))?;
        if !output.status.success() {
            return Err(anyhow!(
                "npm {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        serde_json::from_slice(&output.stdout).context("npm returned invalid JSON")
    }
}

impl SourceProvider for NpmSource {
    fn resolve_alias(&self, requested: &str) -> Result<String> {
        if requested != "latest" && requested != "stable" {
            return Ok(requested.to_string());
        }
        let tags = self.npm_json(&["view", &self.package, "dist-tags", "--json"])?;
        tags.get(requested)
            .or_else(|| tags.get("latest"))
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned)
            .ok_or_else(|| anyhow!("npm dist-tags did not include {requested} or latest"))
    }

    fn list_remote(&self, channel: Option<&str>) -> Result<Vec<RemoteVersion>> {
        let versions = self.npm_json(&["view", &self.package, "versions", "--json"])?;
        let tags = self.npm_json(&["view", &self.package, "dist-tags", "--json"])?;
        let versions = versions
            .as_array()
            .ok_or_else(|| anyhow!("npm versions response was not an array"))?;

        let mut rows = Vec::new();
        for version in versions {
            let version = version
                .as_str()
                .ok_or_else(|| anyhow!("npm versions response included a non-string version"))?;
            let mut labels = Vec::new();
            for tag in ["stable", "latest", "beta"] {
                if tags.get(tag).and_then(|value| value.as_str()) == Some(version) {
                    labels.push(tag.to_string());
                }
            }
            if !labels.iter().any(|label| label == "stable")
                && tags.get("stable").is_none()
                && tags.get("latest").and_then(|value| value.as_str()) == Some(version)
            {
                labels.push("stable".to_string());
            }
            if version.contains('-') {
                labels.push("prerelease".to_string());
            }
            if version.to_ascii_lowercase().contains("nightly") {
                labels.push("nightly".to_string());
            }
            if channel
                .map(|wanted| labels.iter().any(|label| label == wanted))
                .unwrap_or(true)
            {
                rows.push(RemoteVersion {
                    version: version.to_string(),
                    tags: labels,
                    url: None,
                    executable_sha256: None,
                });
            }
        }
        Ok(rows)
    }

    fn install(&self, version: &str, staging_dir: &Path) -> Result<()> {
        let spec = format!("{}@{}", self.package, version);
        let output = Command::new("npm")
            .args(["install", "--prefix"])
            .arg(staging_dir)
            .args(["--omit=dev", "--no-save", &spec])
            .output()
            .with_context(|| format!("failed to install {spec}"))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow!(
                "npm install failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
struct ReleaseManifest {
    versions: Vec<RemoteVersion>,
}

pub struct ReleaseSource {
    manifest_url: String,
}

impl ReleaseSource {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            manifest_url: std::env::var("OCVM_RELEASE_MANIFEST_URL")
                .context("OCVM_RELEASE_MANIFEST_URL is required when source is release")?,
        })
    }

    fn versions(&self) -> Result<Vec<RemoteVersion>> {
        let response = reqwest::blocking::get(&self.manifest_url)
            .with_context(|| format!("failed to fetch {}", self.manifest_url))?;
        if !response.status().is_success() {
            return Err(anyhow!(
                "release manifest returned HTTP {}",
                response.status()
            ));
        }
        let raw = response.text().context("failed to read release manifest")?;
        if let Ok(manifest) = serde_json::from_str::<ReleaseManifest>(&raw) {
            return Ok(manifest.versions);
        }
        serde_json::from_str::<Vec<RemoteVersion>>(&raw).context("failed to parse release manifest")
    }
}

impl SourceProvider for ReleaseSource {
    fn resolve_alias(&self, requested: &str) -> Result<String> {
        if requested != "latest" && requested != "stable" {
            return Ok(requested.to_string());
        }
        self.versions()?
            .into_iter()
            .find(|version| version.tags.iter().any(|tag| tag == requested))
            .map(|version| version.version)
            .ok_or_else(|| anyhow!("release manifest did not include {requested}"))
    }

    fn list_remote(&self, channel: Option<&str>) -> Result<Vec<RemoteVersion>> {
        let versions = self.versions()?;
        Ok(match channel {
            Some(channel) => versions
                .into_iter()
                .filter(|version| version.tags.iter().any(|tag| tag == channel))
                .collect(),
            None => versions,
        })
    }

    fn install(&self, version: &str, staging_dir: &Path) -> Result<()> {
        let version = self
            .versions()?
            .into_iter()
            .find(|candidate| candidate.version == version)
            .ok_or_else(|| anyhow!("version {version} was not found in release manifest"))?;
        let spec = version
            .url
            .context("release manifest entry did not include url")?;
        let output = Command::new("npm")
            .args(["install", "--prefix"])
            .arg(staging_dir)
            .args(["--omit=dev", "--no-save", &spec])
            .output()
            .with_context(|| format!("failed to install {spec}"))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow!(
                "npm install failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }

    fn verify_staged_install(&self, version: &str, staging_dir: &Path) -> Result<()> {
        let Some(expected) = self
            .versions()?
            .into_iter()
            .find(|candidate| candidate.version == version)
            .and_then(|version| version.executable_sha256)
        else {
            return Ok(());
        };

        let executable = staging_dir
            .join("node_modules")
            .join(".bin")
            .join("openclaw");
        let bytes = std::fs::read(&executable)
            .with_context(|| format!("failed to read {}", executable.display()))?;
        let actual = hex::encode(Sha256::digest(bytes));
        if actual.eq_ignore_ascii_case(&expected) {
            Ok(())
        } else {
            Err(anyhow!(
                "checksum mismatch for {}: expected {}, got {}",
                executable.display(),
                expected,
                actual
            ))
        }
    }
}

pub fn provider_from_config(paths: &OcvmPaths) -> Result<Box<dyn SourceProvider>> {
    let source = std::env::var("OCVM_SOURCE")
        .ok()
        .map(|value| match value.as_str() {
            "npm" => Ok(SourceKind::Npm),
            "release" => Ok(SourceKind::Release),
            other => Err(anyhow!("unknown OCVM_SOURCE: {other}")),
        })
        .transpose()?
        .unwrap_or(load(paths)?.source);

    match source {
        SourceKind::Npm => Ok(Box::new(NpmSource::from_env())),
        SourceKind::Release => Ok(Box::new(ReleaseSource::from_env()?)),
    }
}
