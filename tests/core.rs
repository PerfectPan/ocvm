use anyhow::Result;
use ocvm::config;
use ocvm::paths::{executable_path, OcvmPaths};
use ocvm::project;
use ocvm::source::{RemoteVersion, SourceProvider};
use ocvm::version;
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct FakeSource {
    versions: Vec<RemoteVersion>,
    fail_install: bool,
    fail_verify: bool,
    installed_specs: RefCell<Vec<String>>,
}

impl FakeSource {
    fn new() -> Self {
        Self {
            versions: vec![
                RemoteVersion {
                    version: "2026.3.28".to_string(),
                    tags: vec!["stable".to_string()],
                    url: Some("stable.tgz".to_string()),
                    executable_sha256: None,
                },
                RemoteVersion {
                    version: "2026.4.01".to_string(),
                    tags: vec!["latest".to_string()],
                    url: Some("latest.tgz".to_string()),
                    executable_sha256: None,
                },
                RemoteVersion {
                    version: "2026.5.01-nightly.1".to_string(),
                    tags: vec!["nightly".to_string(), "prerelease".to_string()],
                    url: Some("nightly.tgz".to_string()),
                    executable_sha256: None,
                },
            ],
            fail_install: false,
            fail_verify: false,
            installed_specs: RefCell::new(Vec::new()),
        }
    }
}

fn fake_openclaw_path(bin: PathBuf) -> PathBuf {
    if cfg!(windows) {
        bin.join("openclaw.cmd")
    } else {
        bin.join("openclaw")
    }
}

fn fake_openclaw_body(output: &str) -> String {
    if cfg!(windows) {
        format!("@echo off\r\necho {output}\r\n")
    } else {
        format!("#!/bin/sh\necho {output}\n")
    }
}

impl SourceProvider for FakeSource {
    fn resolve_alias(&self, requested: &str) -> Result<String> {
        Ok(self
            .versions
            .iter()
            .find(|version| version.tags.iter().any(|tag| tag == requested))
            .or_else(|| {
                if requested == "stable" {
                    self.versions
                        .iter()
                        .find(|version| version.tags.iter().any(|tag| tag == "latest"))
                } else {
                    None
                }
            })
            .map(|version| version.version.clone())
            .unwrap_or_else(|| requested.to_string()))
    }

    fn list_remote(&self, channel: Option<&str>) -> Result<Vec<RemoteVersion>> {
        Ok(self
            .versions
            .iter()
            .filter(|version| {
                channel
                    .map(|wanted| version.tags.iter().any(|tag| tag == wanted))
                    .unwrap_or(true)
            })
            .cloned()
            .collect())
    }

    fn install(&self, version: &str, staging_dir: &Path) -> Result<()> {
        if self.fail_install {
            anyhow::bail!("registry unavailable");
        }
        self.installed_specs.borrow_mut().push(version.to_string());
        let bin = staging_dir.join("node_modules").join(".bin");
        fs::create_dir_all(&bin)?;
        let openclaw = fake_openclaw_path(bin);
        fs::write(&openclaw, fake_openclaw_body(version))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&openclaw)?.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(openclaw, permissions)?;
        }
        Ok(())
    }

    fn verify_staged_install(&self, _version: &str, _staging_dir: &Path) -> Result<()> {
        if self.fail_verify {
            anyhow::bail!("checksum mismatch");
        }
        Ok(())
    }
}

fn paths(tmp: &TempDir) -> OcvmPaths {
    OcvmPaths::new(tmp.path().join("home"))
}

#[test]
fn install_stages_verifies_and_creates_shim() {
    let tmp = TempDir::new().unwrap();
    let paths = paths(&tmp);
    let source = FakeSource::new();

    let installed = version::install(&paths, &source, "stable").unwrap();

    assert_eq!(installed, "2026.3.28");
    assert!(paths.openclaw_bin("2026.3.28").exists());
    assert!(executable_path(paths.shims.join("openclaw")).exists());
    assert_eq!(source.installed_specs.borrow().as_slice(), ["2026.3.28"]);
}

#[test]
fn failed_install_preserves_existing_version() {
    let tmp = TempDir::new().unwrap();
    let paths = paths(&tmp);
    let source = FakeSource::new();
    version::install(&paths, &source, "2026.3.28").unwrap();
    let original = fs::read_to_string(paths.openclaw_bin("2026.3.28")).unwrap();

    let failing = FakeSource {
        fail_install: true,
        ..FakeSource::new()
    };
    let error = version::install(&paths, &failing, "2026.3.28").unwrap_err();

    assert!(error.to_string().contains("registry unavailable"));
    assert_eq!(
        fs::read_to_string(paths.openclaw_bin("2026.3.28")).unwrap(),
        original
    );
}

#[test]
fn failed_staged_verification_preserves_existing_version() {
    let tmp = TempDir::new().unwrap();
    let paths = paths(&tmp);
    let source = FakeSource::new();
    version::install(&paths, &source, "2026.3.28").unwrap();
    let original = fs::read_to_string(paths.openclaw_bin("2026.3.28")).unwrap();

    let failing = FakeSource {
        fail_verify: true,
        ..FakeSource::new()
    };
    let error = version::install(&paths, &failing, "2026.3.28").unwrap_err();

    assert!(error.to_string().contains("checksum mismatch"));
    assert_eq!(
        fs::read_to_string(paths.openclaw_bin("2026.3.28")).unwrap(),
        original
    );
}

#[test]
fn project_pin_precedes_global_default() {
    let tmp = TempDir::new().unwrap();
    let paths = paths(&tmp);
    config::set_default(&paths, "2026.3.28".to_string()).unwrap();
    let project = tmp.path().join("repo");
    let child = project.join("plugins").join("demo");
    fs::create_dir_all(&child).unwrap();
    project::pin(&project, "2026.4.01").unwrap();

    let resolved = version::resolve_active(&paths, &child).unwrap().unwrap();

    assert_eq!(resolved.version, "2026.4.01");
    assert_eq!(resolved.source.to_string(), "project");
}

#[test]
fn remote_versions_can_be_filtered_by_channel() {
    let source = FakeSource::new();

    let nightly = source.list_remote(Some("nightly")).unwrap();

    assert_eq!(nightly.len(), 1);
    assert_eq!(nightly[0].version, "2026.5.01-nightly.1");
    assert!(nightly[0].tags.contains(&"prerelease".to_string()));
}

#[test]
fn stable_alias_can_fallback_to_latest_when_source_has_no_stable() {
    let tmp = TempDir::new().unwrap();
    let paths = paths(&tmp);
    let source = FakeSource {
        versions: vec![RemoteVersion {
            version: "2026.4.01".to_string(),
            tags: vec!["latest".to_string()],
            url: Some("latest.tgz".to_string()),
            executable_sha256: None,
        }],
        fail_install: false,
        fail_verify: false,
        installed_specs: RefCell::new(Vec::new()),
    };

    let resolved = version::resolve_requested(&paths, &source, "stable").unwrap();

    assert_eq!(resolved, "2026.4.01");
}

#[test]
fn release_manifest_versions_parse_sha256_alias() {
    let parsed: serde_json::Value = serde_json::from_str(
        r#"{
          "version": "2026.3.28",
          "tags": ["stable"],
          "url": "openclaw.tgz",
          "sha256": "abc123"
        }"#,
    )
    .unwrap();
    let version: RemoteVersion = serde_json::from_value(parsed).unwrap();

    assert_eq!(version.executable_sha256.as_deref(), Some("abc123"));
}

#[test]
fn snapshot_and_rollback_restore_default_and_session_without_deleting_versions() {
    let tmp = TempDir::new().unwrap();
    let paths = paths(&tmp);
    let source = FakeSource::new();
    version::install(&paths, &source, "2026.3.28").unwrap();
    version::install(&paths, &source, "2026.4.01").unwrap();
    config::set_default(&paths, "2026.3.28".to_string()).unwrap();
    version::use_version(&paths, "2026.3.28").unwrap();

    let snapshot = version::snapshot(&paths, Some("before-upgrade")).unwrap();
    config::set_default(&paths, "2026.4.01".to_string()).unwrap();
    version::use_version(&paths, "2026.4.01").unwrap();

    version::rollback(&paths, Some("before-upgrade")).unwrap();

    assert_eq!(snapshot.name, "before-upgrade");
    assert_eq!(
        config::load(&paths).unwrap().default_version.as_deref(),
        Some("2026.3.28")
    );
    assert_eq!(
        fs::read_to_string(&paths.current).unwrap().trim(),
        "2026.3.28"
    );
    assert!(paths.openclaw_bin("2026.4.01").exists());
}
