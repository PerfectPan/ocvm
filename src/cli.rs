use crate::config;
use crate::paths::OcvmPaths;
use crate::project;
use crate::source::provider_from_config;
use crate::version;
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use std::ffi::OsString;

#[derive(Debug, Parser)]
#[command(name = "ocvm", version, about = "OpenClaw Version Manager")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Install an OpenClaw version, latest, or stable.
    Install { version: String },
    /// Uninstall an installed OpenClaw version.
    Uninstall { version: String },
    /// Select a version for the current shell session.
    Use { version: String },
    /// Set the global default version.
    Default { version: String },
    /// Show the current resolved version.
    Current,
    /// List local installed versions.
    List,
    /// List remote installable versions.
    ListRemote {
        #[arg(long)]
        channel: Option<String>,
    },
    /// Pin a project version in .openclaw-version.
    Pin { version: String },
    /// Remove .openclaw-version from the current directory.
    Unpin,
    /// Execute a command with a resolved or explicit OpenClaw version.
    Exec {
        version: Option<String>,
        #[arg(last = true, required = true)]
        command: Vec<OsString>,
    },
    /// Diagnose ocvm and OpenClaw setup.
    Doctor,
}

pub fn run<I, T>(args: I) -> Result<u8>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    let paths = OcvmPaths::from_env()?;
    let cwd = std::env::current_dir()?;

    match cli.command {
        Commands::Install { version: requested } => {
            let source = provider_from_config(&paths)?;
            let version = version::install(&paths, source.as_ref(), &requested)?;
            println!(
                "installed {} at {}",
                version,
                paths.version_dir(&version).display()
            );
            Ok(0)
        }
        Commands::Uninstall { version } => {
            version::uninstall(&paths, &version)?;
            println!("uninstalled {version}");
            Ok(0)
        }
        Commands::Use { version: requested } => {
            let version = resolve_for_local_command(&paths, &requested)?;
            let shell = version::use_version(&paths, &version)?;
            println!("using {version}");
            println!("Run this in your shell: {shell}");
            Ok(0)
        }
        Commands::Default { version: requested } => {
            let version = resolve_for_local_command(&paths, &requested)?;
            config::set_default(&paths, version.clone())?;
            version::write_shim(&paths)?;
            println!("default {version}");
            Ok(0)
        }
        Commands::Current => {
            let resolved = version::resolve_executable(&paths, &cwd, None)?
                .ok_or_else(|| anyhow!("No OpenClaw version is active"))?;
            println!("version: {}", resolved.version);
            println!("source: {}", resolved.source);
            println!("path: {}", paths.version_dir(&resolved.version).display());
            Ok(0)
        }
        Commands::List => {
            let versions = version::list_installed(&paths)?;
            if versions.is_empty() {
                println!("No versions installed");
            } else {
                for version in versions {
                    println!("{version}");
                }
            }
            Ok(0)
        }
        Commands::ListRemote { channel } => {
            let source = provider_from_config(&paths)?;
            for remote in source.list_remote(channel.as_deref())? {
                if remote.tags.is_empty() {
                    println!("{}", remote.version);
                } else {
                    println!("{}  {}", remote.version, remote.tags.join(", "));
                }
            }
            Ok(0)
        }
        Commands::Pin { version: requested } => {
            let version = resolve_for_local_command(&paths, &requested)?;
            let file = project::pin(&cwd, &version)?;
            println!("pinned {version} in {}", file.display());
            Ok(0)
        }
        Commands::Unpin => {
            let file = project::unpin(&cwd)?;
            println!("removed {}", file.display());
            Ok(0)
        }
        Commands::Exec { version, command } => {
            let explicit = version.as_deref();
            let code = version::exec(&paths, &cwd, explicit, command)?;
            Ok(code as u8)
        }
        Commands::Doctor => {
            let source = provider_from_config(&paths)?;
            let (ok, findings) = version::doctor(&paths, &cwd, source.as_ref())?;
            if findings.is_empty() {
                println!("ocvm doctor: ok");
            } else {
                for finding in findings {
                    println!("{finding}");
                }
            }
            Ok(if ok { 0 } else { 1 })
        }
    }
}

fn resolve_for_local_command(paths: &OcvmPaths, requested: &str) -> Result<String> {
    if requested != "latest" && requested != "stable" {
        return Ok(requested.to_string());
    }

    let config = config::load(paths)?;
    if let Some(version) = config.channels.get(requested) {
        return Ok(version.clone());
    }

    let source = provider_from_config(paths)?;
    source.resolve_alias(requested)
}
