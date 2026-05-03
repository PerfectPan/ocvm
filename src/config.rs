use crate::paths::OcvmPaths;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub default_version: Option<String>,
    pub source: SourceKind,
    pub channels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    Npm,
    Release,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_version: None,
            source: SourceKind::Npm,
            channels: BTreeMap::new(),
        }
    }
}

pub fn load(paths: &OcvmPaths) -> Result<Config> {
    paths.ensure()?;
    if !paths.config.exists() {
        return Ok(Config::default());
    }
    let raw = std::fs::read_to_string(&paths.config)
        .with_context(|| format!("failed to read {}", paths.config.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", paths.config.display()))
}

pub fn save(paths: &OcvmPaths, config: &Config) -> Result<()> {
    paths.ensure()?;
    let raw = serde_json::to_string_pretty(config)?;
    std::fs::write(&paths.config, format!("{raw}\n"))
        .with_context(|| format!("failed to write {}", paths.config.display()))
}

pub fn set_default(paths: &OcvmPaths, version: String) -> Result<()> {
    let mut config = load(paths)?;
    config.default_version = Some(version);
    save(paths, &config)
}
