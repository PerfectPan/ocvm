use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub const PROJECT_VERSION_FILE: &str = ".openclaw-version";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectVersion {
    pub version: String,
    pub file: PathBuf,
}

pub fn find(start: impl AsRef<Path>) -> Result<Option<ProjectVersion>> {
    let mut current = start
        .as_ref()
        .canonicalize()
        .unwrap_or_else(|_| start.as_ref().to_path_buf());
    loop {
        let file = current.join(PROJECT_VERSION_FILE);
        if file.exists() {
            let version = std::fs::read_to_string(&file)
                .with_context(|| format!("failed to read {}", file.display()))?
                .trim()
                .to_string();
            if !version.is_empty() {
                return Ok(Some(ProjectVersion { version, file }));
            }
        }

        if !current.pop() {
            return Ok(None);
        }
    }
}

pub fn pin(cwd: impl AsRef<Path>, version: &str) -> Result<PathBuf> {
    let file = cwd.as_ref().join(PROJECT_VERSION_FILE);
    std::fs::write(&file, format!("{version}\n"))
        .with_context(|| format!("failed to write {}", file.display()))?;
    Ok(file)
}

pub fn unpin(cwd: impl AsRef<Path>) -> Result<PathBuf> {
    let file = cwd.as_ref().join(PROJECT_VERSION_FILE);
    match std::fs::remove_file(&file) {
        Ok(()) => Ok(file),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(file),
        Err(error) => Err(error).with_context(|| format!("failed to remove {}", file.display())),
    }
}
