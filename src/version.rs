use crate::config;
use crate::paths::{executable_path, path_contains_dir, OcvmPaths};
use crate::project;
use crate::source::SourceProvider;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionSource {
    Explicit,
    Session,
    Project,
    Global,
}

impl std::fmt::Display for VersionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionSource::Explicit => write!(f, "explicit"),
            VersionSource::Session => write!(f, "session"),
            VersionSource::Project => write!(f, "project"),
            VersionSource::Global => write!(f, "global"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedVersion {
    pub version: String,
    pub source: VersionSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snapshot {
    pub name: String,
    pub created_unix_seconds: u64,
    pub default_version: Option<String>,
    pub session_version: Option<String>,
}

pub fn resolve_requested(
    paths: &OcvmPaths,
    source: &dyn SourceProvider,
    requested: &str,
) -> Result<String> {
    if requested == "latest" || requested == "stable" {
        let config = config::load(paths)?;
        if let Some(version) = config.channels.get(requested) {
            return Ok(version.clone());
        }
        return source.resolve_alias(requested);
    }
    Ok(requested.to_string())
}

pub fn resolve_active(paths: &OcvmPaths, cwd: &Path) -> Result<Option<ResolvedVersion>> {
    if let Some(project) = project::find(cwd)? {
        return Ok(Some(ResolvedVersion {
            version: project.version,
            source: VersionSource::Project,
        }));
    }
    let config = config::load(paths)?;
    Ok(config.default_version.map(|version| ResolvedVersion {
        version,
        source: VersionSource::Global,
    }))
}

pub fn resolve_executable(
    paths: &OcvmPaths,
    cwd: &Path,
    explicit: Option<&str>,
) -> Result<Option<ResolvedVersion>> {
    if let Some(version) = explicit {
        return Ok(Some(ResolvedVersion {
            version: version.to_string(),
            source: VersionSource::Explicit,
        }));
    }
    if let Ok(session) = std::fs::read_to_string(&paths.current) {
        let session = session.trim();
        if !session.is_empty() {
            return Ok(Some(ResolvedVersion {
                version: session.to_string(),
                source: VersionSource::Session,
            }));
        }
    }
    resolve_active(paths, cwd)
}

pub fn is_installed(paths: &OcvmPaths, version: &str) -> bool {
    paths.openclaw_bin(version).exists()
}

pub fn list_installed(paths: &OcvmPaths) -> Result<Vec<String>> {
    paths.ensure()?;
    let mut versions = Vec::new();
    for entry in std::fs::read_dir(&paths.versions)
        .with_context(|| format!("failed to read {}", paths.versions.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            versions.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    versions.sort();
    Ok(versions)
}

pub fn write_shim(paths: &OcvmPaths) -> Result<()> {
    paths.ensure()?;
    let shim = paths.shims.join(if cfg!(windows) {
        "openclaw.cmd"
    } else {
        "openclaw"
    });
    let body = if cfg!(windows) {
        "@echo off\r\nocvm exec -- openclaw %*\r\n".to_string()
    } else {
        "#!/bin/sh\nexec ocvm exec -- openclaw \"$@\"\n".to_string()
    };
    std::fs::write(&shim, body).with_context(|| format!("failed to write {}", shim.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&shim)?.permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&shim, permissions)?;
    }
    Ok(())
}

pub fn install(paths: &OcvmPaths, source: &dyn SourceProvider, requested: &str) -> Result<String> {
    paths.ensure()?;
    let version = resolve_requested(paths, source, requested)?;
    let staging = paths
        .home
        .join(format!(".install-{version}-{}", std::process::id()));
    let target = paths.version_dir(&version);
    let backup = paths
        .home
        .join(format!(".backup-{version}-{}", std::process::id()));

    let _ = std::fs::remove_dir_all(&staging);
    let _ = std::fs::remove_dir_all(&backup);
    std::fs::create_dir_all(&staging)?;

    let result = (|| {
        source.install(&version, &staging)?;
        source.verify_staged_install(&version, &staging)?;
        let output = Command::new(executable_path(
            staging.join("node_modules").join(".bin").join("openclaw"),
        ))
        .arg("--version")
        .output()
        .context("failed to run openclaw --version after install")?;
        if !output.status.success() {
            return Err(anyhow!(
                "openclaw --version failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        if target.exists() {
            std::fs::rename(&target, &backup)
                .with_context(|| format!("failed to move existing version {}", target.display()))?;
        }

        match std::fs::rename(&staging, &target) {
            Ok(()) => {
                let _ = std::fs::remove_dir_all(&backup);
                write_shim(paths)?;
                Ok(version.clone())
            }
            Err(error) => {
                if backup.exists() {
                    let _ = std::fs::rename(&backup, &target);
                }
                Err(error).with_context(|| format!("failed to activate {}", target.display()))
            }
        }
    })();

    if staging.exists() {
        let _ = std::fs::remove_dir_all(&staging);
    }
    if backup.exists() {
        let _ = std::fs::remove_dir_all(&backup);
    }
    result
}

pub fn snapshot(paths: &OcvmPaths, name: Option<&str>) -> Result<Snapshot> {
    paths.ensure()?;
    let name = name
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "latest".to_string());
    let config = config::load(paths)?;
    let session_version = std::fs::read_to_string(&paths.current)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let created_unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time is before UNIX_EPOCH")?
        .as_secs();
    let snapshot = Snapshot {
        name: name.clone(),
        created_unix_seconds,
        default_version: config.default_version,
        session_version,
    };
    let file = paths.snapshots.join(format!("{name}.json"));
    std::fs::write(&file, serde_json::to_string_pretty(&snapshot)? + "\n")
        .with_context(|| format!("failed to write {}", file.display()))?;
    Ok(snapshot)
}

pub fn rollback(paths: &OcvmPaths, name: Option<&str>) -> Result<Snapshot> {
    paths.ensure()?;
    let name = name.unwrap_or("latest");
    let file = paths.snapshots.join(format!("{name}.json"));
    let raw = std::fs::read_to_string(&file)
        .with_context(|| format!("failed to read snapshot {}", file.display()))?;
    let snapshot: Snapshot = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse snapshot {}", file.display()))?;

    let mut config = config::load(paths)?;
    config.default_version = snapshot.default_version.clone();
    config::save(paths, &config)?;

    match &snapshot.session_version {
        Some(version) => std::fs::write(&paths.current, format!("{version}\n"))?,
        None => {
            let _ = std::fs::remove_file(&paths.current);
        }
    }

    Ok(snapshot)
}

pub fn uninstall(paths: &OcvmPaths, version: &str) -> Result<()> {
    let dir = paths.version_dir(version);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)
            .with_context(|| format!("failed to remove {}", dir.display()))?;
    }
    Ok(())
}

pub fn use_version(paths: &OcvmPaths, version: &str) -> Result<String> {
    paths.ensure()?;
    if !is_installed(paths, version) {
        return Err(anyhow!(
            "Version {version} is not installed. Run: ocvm install {version}"
        ));
    }
    std::fs::write(&paths.current, format!("{version}\n"))?;
    Ok(format!(
        "export PATH=\"{}:$PATH\"",
        paths.bin_dir(version).display()
    ))
}

pub fn exec<I, S>(paths: &OcvmPaths, cwd: &Path, explicit: Option<&str>, command: I) -> Result<i32>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let resolved = resolve_executable(paths, cwd, explicit)?.ok_or_else(|| {
        anyhow!("No OpenClaw version resolved. Run: ocvm default <version> or ocvm pin <version>")
    })?;
    if !is_installed(paths, &resolved.version) {
        return Err(anyhow!(
            "Version {} is not installed. Run: ocvm install {}",
            resolved.version,
            resolved.version
        ));
    }
    let mut iter = command.into_iter();
    let cmd = iter
        .next()
        .ok_or_else(|| anyhow!("exec requires a command"))?;
    let path = std::env::var_os("PATH").unwrap_or_default();
    let path = std::env::join_paths(
        std::iter::once(paths.bin_dir(&resolved.version)).chain(std::env::split_paths(&path)),
    )?;
    let status = Command::new(cmd)
        .args(iter)
        .current_dir(cwd)
        .env("PATH", path)
        .env("OCVM_ACTIVE_VERSION", &resolved.version)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to execute command")?;
    Ok(status.code().unwrap_or(1))
}

pub fn doctor(
    paths: &OcvmPaths,
    cwd: &Path,
    source: &dyn SourceProvider,
) -> Result<(bool, Vec<String>)> {
    paths.ensure()?;
    write_shim(paths)?;
    let mut ok = true;
    let mut findings = Vec::new();

    let path = std::env::var("PATH").unwrap_or_default();
    if !path_contains_dir(&path, &paths.shims) {
        ok = false;
        findings.push(format!(
            "fix: PATH does not include {}\n  Add this to your shell profile: export PATH=\"{}:$PATH\"",
            paths.shims.display(),
            paths.shims.display()
        ));
    }

    match resolve_executable(paths, cwd, None)? {
        Some(resolved) if is_installed(paths, &resolved.version) => {
            let output = Command::new(paths.openclaw_bin(&resolved.version))
                .arg("--version")
                .output();
            if !matches!(output, Ok(output) if output.status.success()) {
                ok = false;
                findings.push(format!(
                    "fix: OpenClaw {} is not executable\n  Reinstall it with: ocvm install {}",
                    resolved.version, resolved.version
                ));
            }
        }
        Some(resolved) => {
            ok = false;
            let label = if resolved.source == VersionSource::Project {
                "Pinned"
            } else {
                "Resolved"
            };
            findings.push(format!(
                "fix: {label} version {} is not installed\n  Run: ocvm install {}",
                resolved.version, resolved.version
            ));
        }
        None => {
            ok = false;
            findings.push("fix: No OpenClaw version is active\n  Run: ocvm default <version> or ocvm pin <version>".to_string());
        }
    }

    if let Err(error) = source.list_remote(Some("stable")) {
        ok = false;
        findings.push(format!(
            "fix: Remote source is not reachable: {error}\n  Check network access and npm/release source configuration"
        ));
    }

    Ok((ok, findings))
}
