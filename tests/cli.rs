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
        .stdout(predicate::str::contains("entry: bibsync"));
}

#[test]
fn cli_defaults_to_check_mode() {
    let dir = tempdir().expect("tempdir");
    let tex = dir.path().join("main.tex");
    let bib = dir.path().join("refs.bib");
    std::fs::write(&tex, "\\cite{NotAnIdentifier}\n\\bibliography{refs}").expect("write tex");
    std::fs::write(&bib, "").expect("write bib");

    let mut command = Command::cargo_bin("bibsync").expect("binary exists");
    command
        .arg("--provider")
        .arg("inspire")
        .arg("--output")
        .arg(&bib)
        .arg(&tex)
        .assert()
        .failure()
        .stdout(predicate::str::contains("unresolved:"))
        .stdout(predicate::str::contains(
            "NotAnIdentifier: unsupported identifier format",
        ));
}

#[test]
fn cli_scans_joss_markdown_paper() {
    let dir = tempdir().expect("tempdir");
    let paper = dir.path().join("paper.md");
    let bib = dir.path().join("refs.bib");
    std::fs::write(
        &paper,
        "---\ntitle: Example\nbibliography: refs.bib\n---\n\nText citing [@NotAnIdentifier].\n",
    )
    .expect("write paper.md");
    std::fs::write(&bib, "").expect("write bib");

    let mut command = Command::cargo_bin("bibsync").expect("binary exists");
    command
        .arg("--provider")
        .arg("inspire")
        .arg("--output")
        .arg(&bib)
        .arg(&paper)
        .assert()
        .failure()
        .stdout(predicate::str::contains("unresolved:"))
        .stdout(predicate::str::contains(
            "NotAnIdentifier: unsupported identifier format",
        ));
}

#[test]
fn cli_reports_missing_ignore_file() {
    let dir = tempdir().expect("tempdir");
    let tex = dir.path().join("main.tex");
    let bib = dir.path().join("refs.bib");
    let ignore = dir.path().join(".bibsyncignore");
    std::fs::write(&tex, "\\cite{NotAnIdentifier}").expect("write tex");

    let mut command = Command::cargo_bin("bibsync").expect("binary exists");
    command
        .arg("--output")
        .arg(&bib)
        .arg("--ignore-file")
        .arg(&ignore)
        .arg(&tex)
        .assert()
        .failure()
        .stderr(predicate::str::contains("ignore file not found"))
        .stderr(predicate::str::contains(ignore.display().to_string()));
}

#[test]
fn cli_reports_malformed_bibtex() {
    let dir = tempdir().expect("tempdir");
    let bib = dir.path().join("refs.bib");
    std::fs::write(&bib, "@article{broken,\n  title = {Missing close}\n")
        .expect("write malformed bib");

    let mut command = Command::cargo_bin("bibsync").expect("binary exists");
    command
        .arg(&bib)
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid BibTeX"))
        .stderr(predicate::str::contains("missing a closing"));
}

#[test]
fn cli_rejects_check_with_fix() {
    let mut command = Command::cargo_bin("bibsync").expect("binary exists");

    command
        .arg("--check")
        .arg("--fix")
        .arg("example.tex")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn cli_exposes_cache_flags() {
    let mut command = Command::cargo_bin("bibsync").expect("binary exists");

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--cache"))
        .stdout(predicate::str::contains("--refresh-cache"))
        .stdout(predicate::str::contains("--cache-dir"));
}
