use anyhow::{Context, Result};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct OcvmPaths {
    pub home: PathBuf,
    pub config: PathBuf,
    pub versions: PathBuf,
    pub shims: PathBuf,
    pub cache: PathBuf,
    pub logs: PathBuf,
    pub snapshots: PathBuf,
    pub current: PathBuf,
}

impl OcvmPaths {
    pub fn from_env() -> Result<Self> {
        let home = match env::var_os("OCVM_HOME") {
            Some(value) => PathBuf::from(value),
            None => dirs::home_dir()
                .context("could not determine home directory")?
                .join(".ocvm"),
        };
        Ok(Self::new(home))
    }

    pub fn new(home: impl Into<PathBuf>) -> Self {
        let home = home.into();
        Self {
            config: home.join("config.json"),
            versions: home.join("versions"),
            shims: home.join("shims"),
            cache: home.join("cache"),
            logs: home.join("logs"),
            snapshots: home.join("snapshots"),
            current: home.join("current"),
            home,
        }
    }

    pub fn ensure(&self) -> Result<()> {
        for dir in [
            &self.home,
            &self.versions,
            &self.shims,
            &self.cache,
            &self.logs,
            &self.snapshots,
        ] {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("failed to create {}", dir.display()))?;
        }
        Ok(())
    }

    pub fn version_dir(&self, version: &str) -> PathBuf {
        self.versions.join(version)
    }

    pub fn bin_dir(&self, version: &str) -> PathBuf {
        self.version_dir(version).join("node_modules").join(".bin")
    }

    pub fn openclaw_bin(&self, version: &str) -> PathBuf {
        executable_path(self.bin_dir(version).join("openclaw"))
    }
}

pub fn executable_path(path: PathBuf) -> PathBuf {
    if cfg!(windows) {
        let exe = path.with_extension("cmd");
        if exe.exists() {
            return exe;
        }
        path.with_extension("exe")
    } else {
        path
    }
}

pub fn path_contains_dir(path_env: &str, dir: &Path) -> bool {
    env::split_paths(path_env).any(|entry| entry == dir)
}
