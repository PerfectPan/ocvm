use anyhow::Result;
use ocvm::config;
use ocvm::paths::OcvmPaths;
use ocvm::project;
use ocvm::source::{RemoteVersion, SourceProvider};
use ocvm::version;
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

struct FakeSource {
    versions: Vec<RemoteVersion>,
    fail_install: bool,
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
                },
                RemoteVersion {
                    version: "2026.4.01".to_string(),
                    tags: vec!["latest".to_string()],
                    url: Some("latest.tgz".to_string()),
                },
                RemoteVersion {
                    version: "2026.5.01-nightly.1".to_string(),
                    tags: vec!["nightly".to_string(), "prerelease".to_string()],
                    url: Some("nightly.tgz".to_string()),
                },
            ],
            fail_install: false,
            installed_specs: RefCell::new(Vec::new()),
        }
    }
}

impl SourceProvider for FakeSource {
    fn resolve_alias(&self, requested: &str) -> Result<String> {
        Ok(self
            .versions
            .iter()
            .find(|version| version.tags.iter().any(|tag| tag == requested))
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
        let openclaw = bin.join("openclaw");
        fs::write(&openclaw, format!("#!/bin/sh\necho {version}\n"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&openclaw)?.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(openclaw, permissions)?;
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
    assert!(paths.shims.join("openclaw").exists());
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
