use assert_cmd::Command;
use predicates::prelude::*;
use std::{fs, path::PathBuf};
use tempfile::TempDir;

fn cmd(home: &TempDir) -> Command {
    let mut command = Command::cargo_bin("ocvm").unwrap();
    command.env("OCVM_HOME", home.path().join("home"));
    command.env("OCVM_NPM_PACKAGE", "openclaw-test");
    command
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

fn install_fake(home: &TempDir, version: &str, output: &str) {
    let bin = home
        .path()
        .join("home")
        .join("versions")
        .join(version)
        .join("node_modules")
        .join(".bin");
    fs::create_dir_all(&bin).unwrap();
    let openclaw = fake_openclaw_path(bin);
    fs::write(&openclaw, fake_openclaw_body(output)).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&openclaw).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(openclaw, permissions).unwrap();
    }
}

#[test]
fn help_is_available_without_initializing_home() {
    let home = TempDir::new().unwrap();
    cmd(&home)
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("OpenClaw Version Manager"));
    assert!(!home.path().join("home").exists());
}

#[test]
fn default_current_list_use_and_uninstall_work_on_local_versions() {
    let home = TempDir::new().unwrap();
    install_fake(&home, "2026.3.28", "2026.3.28");
    install_fake(&home, "2026.4.01", "2026.4.01");

    cmd(&home).args(["default", "2026.3.28"]).assert().success();
    cmd(&home)
        .arg("current")
        .assert()
        .success()
        .stdout(predicate::str::contains("version: 2026.3.28"))
        .stdout(predicate::str::contains("source: global"));
    cmd(&home)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("2026.3.28"))
        .stdout(predicate::str::contains("2026.4.01"));
    cmd(&home)
        .args(["use", "2026.4.01"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Run this in your shell: export PATH=",
        ));
    cmd(&home)
        .arg("current")
        .assert()
        .success()
        .stdout(predicate::str::contains("source: session"));
    cmd(&home)
        .args(["uninstall", "2026.4.01"])
        .assert()
        .success();
    cmd(&home)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("2026.4.01").not());
}

#[test]
fn project_pin_wins_and_exec_explicit_does_not_change_default() {
    let home = TempDir::new().unwrap();
    install_fake(&home, "2026.3.28", "default");
    install_fake(&home, "2026.4.01", "explicit");
    cmd(&home).args(["default", "2026.3.28"]).assert().success();

    let project = home.path().join("project");
    let child = project.join("plugins").join("demo");
    fs::create_dir_all(&child).unwrap();
    fs::write(project.join(".openclaw-version"), "2026.4.01\n").unwrap();

    cmd(&home)
        .current_dir(&child)
        .arg("current")
        .assert()
        .success()
        .stdout(predicate::str::contains("source: project"))
        .stdout(predicate::str::contains("2026.4.01"));
    cmd(&home)
        .current_dir(&child)
        .args(["exec", "2026.4.01", "--", "openclaw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("explicit"));
    cmd(&home)
        .arg("current")
        .assert()
        .success()
        .stdout(predicate::str::contains("2026.3.28"));
}

#[test]
fn damaged_default_does_not_block_explicit_healthy_version() {
    let home = TempDir::new().unwrap();
    fs::create_dir_all(
        home.path()
            .join("home")
            .join("versions")
            .join("2026.3.28")
            .join("node_modules")
            .join(".bin"),
    )
    .unwrap();
    install_fake(&home, "2026.4.01", "healthy");
    cmd(&home).args(["default", "2026.3.28"]).assert().success();
    cmd(&home)
        .args(["exec", "2026.4.01", "--", "openclaw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("healthy"));
}

#[test]
fn init_outputs_shell_helpers() {
    let home = TempDir::new().unwrap();

    cmd(&home)
        .args(["init", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ocvm-use"))
        .stdout(predicate::str::contains("eval \"$"))
        .stdout(predicate::str::contains("ocvm use"));

    cmd(&home)
        .args(["init", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("function ocvm-use"))
        .stdout(predicate::str::contains("ocvm use $version"));
}

#[test]
fn snapshot_and_rollback_work_from_cli() {
    let home = TempDir::new().unwrap();
    install_fake(&home, "2026.3.28", "old");
    install_fake(&home, "2026.4.01", "new");
    cmd(&home).args(["default", "2026.3.28"]).assert().success();
    cmd(&home).args(["use", "2026.3.28"]).assert().success();

    cmd(&home)
        .args(["snapshot", "before-upgrade"])
        .assert()
        .success()
        .stdout(predicate::str::contains("snapshot before-upgrade"));

    cmd(&home).args(["default", "2026.4.01"]).assert().success();
    cmd(&home).args(["use", "2026.4.01"]).assert().success();

    cmd(&home)
        .args(["rollback", "before-upgrade"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rolled back to before-upgrade"));

    cmd(&home)
        .arg("current")
        .assert()
        .success()
        .stdout(predicate::str::contains("version: 2026.3.28"));
}
