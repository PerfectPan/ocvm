use crate::config;
use crate::paths::{executable_path, path_contains_dir, OcvmPaths};
use crate::project;
use crate::source::SourceProvider;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
    let config = config::load(paths)?;
    if let Some(version) = config.channels.get(requested) {
        return Ok(version.clone());
    }
    if !crate::source::is_channel_name(requested) {
        return Ok(requested.to_string());
    }
    source.resolve_alias(requested)
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
    versions.sort_by(|left, right| compare_versions(left, right));
    Ok(versions)
}

/// Compare OpenClaw calendar versions such as `2026.10.9` and
/// `2026.10.10-beta.5`.
///
/// OpenClaw does not zero-pad version segments, so plain string ordering puts
/// `2026.10.9` before `2026.9.36`. Compare numeric segments numerically and
/// treat a prerelease as lower than its matching final release.
pub fn compare_versions(left: &str, right: &str) -> Ordering {
    let (left_core, left_prerelease) = split_version(left);
    let (right_core, right_prerelease) = split_version(right);
    let core = compare_segments(left_core, right_core);
    if core != Ordering::Equal {
        return core;
    }
    match (left_prerelease, right_prerelease) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(left), Some(right)) => compare_segments(left, right),
    }
}

fn split_version(version: &str) -> (&str, Option<&str>) {
    let version = version.split('+').next().unwrap_or(version);
    match version.split_once('-') {
        Some((core, prerelease)) => (core, Some(prerelease)),
        None => (version, None),
    }
}

fn compare_segments(left: &str, right: &str) -> Ordering {
    let left: Vec<&str> = left.split('.').collect();
    let right: Vec<&str> = right.split('.').collect();
    for index in 0..left.len().max(right.len()) {
        let left = left.get(index).copied().unwrap_or("0");
        let right = right.get(index).copied().unwrap_or("0");
        let ordering = compare_segment(left, right);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    Ordering::Equal
}

fn compare_segment(left: &str, right: &str) -> Ordering {
    match (left.parse::<u64>(), right.parse::<u64>()) {
        (Ok(left), Ok(right)) => left.cmp(&right),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => left.cmp(right),
    }
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

const NODE_MINIMUM: &str = "Node.js 24.16+ (24.x) or 26.1+ (26.x)";
const NODE_RECOMMENDED: &str = "24.22.1+ or 26.12.1+";
const NODE_REQUIREMENTS_URL: &str = "https://docs.openclaw.ai/install/node";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeVersion {
    major: u64,
    minor: u64,
    patch: u64,
    prerelease: bool,
}

impl NodeVersion {
    fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        let raw = raw.strip_prefix('v').unwrap_or(raw);
        let (raw, _build) = raw.split_once('+').unwrap_or((raw, ""));
        let (core, prerelease) = match raw.split_once('-') {
            Some((core, suffix)) => (core, !suffix.is_empty()),
            None => (raw, false),
        };
        let mut segments = core.split('.');
        Some(Self {
            major: segments.next()?.parse().ok()?,
            minor: segments.next().unwrap_or("0").parse().ok()?,
            patch: segments.next().unwrap_or("0").parse().ok()?,
            prerelease,
        })
    }

    fn is_supported(self) -> bool {
        if self.prerelease && self.major <= 26 {
            return false;
        }
        (self.major == 24 && self.minor >= 16)
            || (self.major == 26 && self.minor >= 1)
            || self.major >= 27
    }

    fn is_recommended(self) -> bool {
        if self.prerelease && self.major <= 26 {
            return false;
        }
        (self.major == 24 && (self.minor > 22 || (self.minor == 22 && self.patch >= 1)))
            || (self.major == 26 && (self.minor > 12 || (self.minor == 12 && self.patch >= 1)))
            || self.major >= 27
    }
}

impl std::fmt::Display for NodeVersion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.prerelease {
            write!(
                formatter,
                "v{}.{}.{} (prerelease)",
                self.major, self.minor, self.patch
            )
        } else {
            write!(formatter, "v{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

fn detect_node_version() -> Option<NodeVersion> {
    let output = Command::new("node").arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    NodeVersion::parse(&String::from_utf8_lossy(&output.stdout))
}

fn node_warning() -> Option<String> {
    match detect_node_version() {
        None => Some(format!(
            "warning: Node.js was not found; OpenClaw requires {NODE_MINIMUM}.\n  See {NODE_REQUIREMENTS_URL}"
        )),
        Some(version) if !version.is_supported() => Some(format!(
            "warning: Node.js {version} is not supported by OpenClaw; install {NODE_MINIMUM}. Node 22, 23, and 25 are unsupported.\n  See {NODE_REQUIREMENTS_URL}"
        )),
        Some(version) if !version.is_recommended() => Some(format!(
            "warning: Node.js {version} is below OpenClaw's recommended security patch level; upgrade to {NODE_RECOMMENDED}."
        )),
        Some(_) => None,
    }
}

pub fn install(paths: &OcvmPaths, source: &dyn SourceProvider, requested: &str) -> Result<String> {
    paths.ensure()?;
    if let Some(warning) = node_warning() {
        eprintln!("{warning}");
    }
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
        let output = run_openclaw_version_with_retry(&executable_path(
            staging.join("node_modules").join(".bin").join("openclaw"),
        ))
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

fn run_openclaw_version_with_retry(openclaw: &Path) -> Result<Output, std::io::Error> {
    const MAX_ATTEMPTS: usize = 8;

    for attempt in 1..=MAX_ATTEMPTS {
        match Command::new(openclaw).arg("--version").output() {
            Err(error) if is_text_file_busy(&error) && attempt < MAX_ATTEMPTS => {
                std::thread::sleep(Duration::from_millis(50));
            }
            result => return result,
        }
    }

    unreachable!("retry loop always returns on the final attempt")
}

fn is_text_file_busy(error: &std::io::Error) -> bool {
    #[cfg(unix)]
    {
        error.raw_os_error() == Some(26)
    }
    #[cfg(not(unix))]
    {
        let _ = error;
        false
    }
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
    let cmd = resolve_command_in_active_bin(paths, &resolved.version, cmd.as_ref());
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

fn resolve_command_in_active_bin(
    paths: &OcvmPaths,
    version: &str,
    command: &OsStr,
) -> std::ffi::OsString {
    let command_path = Path::new(command);
    if command_path.components().count() == 1 {
        let active_bin_command = executable_path(paths.bin_dir(version).join(command_path));
        if active_bin_command.exists() {
            return active_bin_command.into_os_string();
        }
    }
    command.to_os_string()
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

    match detect_node_version() {
        None => {
            ok = false;
            findings.push(format!(
                "fix: Node.js was not found; OpenClaw requires {NODE_MINIMUM}\n  See {NODE_REQUIREMENTS_URL}"
            ));
        }
        Some(version) if !version.is_supported() => {
            ok = false;
            findings.push(format!(
                "fix: Node.js {version} is not supported by OpenClaw; install {NODE_MINIMUM}. Node 22, 23, and 25 are unsupported.\n  See {NODE_REQUIREMENTS_URL}"
            ));
        }
        Some(version) if !version.is_recommended() => findings.push(format!(
            "warn: Node.js {version} is below OpenClaw's recommended security patch level; upgrade to {NODE_RECOMMENDED}."
        )),
        Some(_) => {}
    }

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

#[cfg(test)]
mod tests {
    use super::{compare_versions, is_text_file_busy, NodeVersion};
    use std::cmp::Ordering;

    #[test]
    #[cfg(unix)]
    fn text_file_busy_errors_are_retryable() {
        let error = std::io::Error::from_raw_os_error(26);

        assert!(is_text_file_busy(&error));
    }

    #[test]
    #[cfg(unix)]
    fn other_io_errors_are_not_text_file_busy() {
        let error = std::io::Error::from_raw_os_error(2);

        assert!(!is_text_file_busy(&error));
    }

    #[test]
    fn calendar_versions_sort_numerically() {
        let mut versions = vec!["2026.10.9", "2026.9.36", "2026.10.10", "2026.3.28"];

        versions.sort_by(|left, right| compare_versions(left, right));

        assert_eq!(
            versions,
            ["2026.3.28", "2026.9.36", "2026.10.9", "2026.10.10"]
        );
    }

    #[test]
    fn prereleases_sort_before_their_final_release() {
        assert_eq!(
            compare_versions("2026.10.10-beta.5", "2026.10.10"),
            Ordering::Less
        );
        assert_eq!(
            compare_versions("2026.10.10-beta.5", "2026.10.10-beta.4"),
            Ordering::Greater
        );
        assert_eq!(
            compare_versions("2026.10.10-beta.10", "2026.10.10-beta.9"),
            Ordering::Greater
        );
    }

    #[test]
    fn node_version_parsing_accepts_prefixes_and_prereleases() {
        assert_eq!(
            NodeVersion::parse("v26.13.0\n"),
            Some(NodeVersion {
                major: 26,
                minor: 13,
                patch: 0,
                prerelease: false
            })
        );
        assert_eq!(
            NodeVersion::parse("27.0.0-nightly20261018f88a1d25e9"),
            Some(NodeVersion {
                major: 27,
                minor: 0,
                patch: 0,
                prerelease: true
            })
        );
        assert_eq!(NodeVersion::parse("not-a-version"), None);
    }

    #[test]
    fn node_support_matrix_matches_openclaw_requirements() {
        let supported = |raw: &str| NodeVersion::parse(raw).unwrap().is_supported();
        let recommended = |raw: &str| NodeVersion::parse(raw).unwrap().is_recommended();

        assert!(!supported("22.14.0"));
        assert!(!supported("23.10.0"));
        assert!(!supported("25.0.0"));
        assert!(supported("24.16.0"));
        assert!(supported("26.1.0"));
        assert!(!supported("24.16.0-rc.1"));
        assert!(!supported("24.22.1-rc.1"));
        assert!(!supported("26.12.1-rc.1"));
        assert!(supported("27.0.0-head"));
        assert!(supported("27.0.0"));

        assert!(!recommended("24.16.0"));
        assert!(!recommended("24.22.0"));
        assert!(recommended("24.22.1"));
        assert!(!recommended("26.12.0"));
        assert!(recommended("26.12.1"));
        assert!(!recommended("26.12.1-rc.1"));
    }
}
