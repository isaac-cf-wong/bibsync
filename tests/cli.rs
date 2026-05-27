//! Integration tests for the command-line interface.

use assert_cmd::Command;
use predicates::prelude::predicate;
use tempfile::tempdir;

#[test]
fn cli_requires_input_files() {
    let mut command = Command::cargo_bin("bibsync").expect("binary exists");

    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("at least one TeX file"));
}

#[test]
fn cli_prints_pre_commit_hook_manifest() {
    let mut command = Command::cargo_bin("bibsync").expect("binary exists");

    command
        .arg("--print-pre-commit-hook")
        .assert()
        .success()
        .stdout(predicate::str::contains("id: bibsync"))
        .stdout(predicate::str::contains("entry: bibsync --check"));
}

#[test]
fn cli_check_mode_accepts_current_empty_bib() {
    let dir = tempdir().expect("tempdir");
    let tex = dir.path().join("main.tex");
    let bib = dir.path().join("refs.bib");
    std::fs::write(&tex, "\\cite{NotAnIdentifier}\n\\bibliography{refs}").expect("write tex");
    std::fs::write(&bib, "").expect("write bib");

    let mut command = Command::cargo_bin("bibsync").expect("binary exists");
    command
        .arg("--check")
        .arg("--provider")
        .arg("inspire")
        .arg("--output")
        .arg(&bib)
        .arg(&tex)
        .assert()
        .failure()
        .stdout(predicate::str::contains("unresolved: NotAnIdentifier"));
}
