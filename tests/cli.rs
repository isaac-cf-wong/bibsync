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

/// The hook configuration the README recommends for a JOSS paper, as pre-commit
/// would run it: `(files regex, pass_filenames, args)`.
fn readme_joss_hook() -> (regex::Regex, bool, Vec<String>) {
    let readme = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))
        .expect("read README.md");
    let section = readme
        .split_once("For a JOSS paper")
        .expect("README has a JOSS hook example")
        .1;
    let block = section
        .split_once("```yaml")
        .and_then(|(_, rest)| rest.split_once("```"))
        .expect("JOSS example is a yaml block")
        .0;
    let files = block
        .lines()
        .find_map(|line| line.trim().strip_prefix("files:"))
        .map_or(".*", str::trim);
    let pass_filenames = !block
        .lines()
        .any(|line| line.trim() == "pass_filenames: false");
    let args = block
        .split_once("args:")
        .and_then(|(_, rest)| rest.split_once('['))
        .and_then(|(_, rest)| rest.split_once(']'))
        .expect("hook args are a list")
        .0
        .split(',')
        .map(str::trim)
        .filter(|arg| !arg.is_empty())
        .map(str::to_owned)
        .collect();
    (
        regex::Regex::new(files).expect("files is a regex"),
        pass_filenames,
        args,
    )
}

/// A commit that only edits `paper.bib` must still be checked against the keys
/// `paper.md` cites. pre-commit passes the hook only the staged files that
/// match, so a hook that relies on the file list either does not run or sees
/// `paper.bib` alone, and a removed entry for a cited key goes unnoticed.
#[test]
fn readme_joss_hook_checks_citations_when_only_bib_changes() {
    let (files, pass_filenames, args) = readme_joss_hook();
    let staged = ["paper.bib"];
    let matched = staged
        .iter()
        .filter(|path| files.is_match(path))
        .collect::<Vec<_>>();
    assert!(
        !matched.is_empty(),
        "the documented hook does not run when only paper.bib changes"
    );

    let dir = tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("paper.md"),
        "---\ntitle: Example\nbibliography: paper.bib\n---\n\nText [@smith2024; @jones2023].\n",
    )
    .expect("write paper.md");
    // The commit removed the entry for jones2023, which paper.md still cites.
    std::fs::write(
        dir.path().join("paper.bib"),
        "@misc{smith2024,\n  title = {Kept}\n}\n",
    )
    .expect("write paper.bib");

    let mut command = Command::cargo_bin("bibsync").expect("binary exists");
    command
        .current_dir(dir.path())
        .env("BIBSYNC_CACHE_DIR", dir.path().join("cache"))
        .args(&args);
    if pass_filenames {
        command.args(&matched);
    }
    command
        .assert()
        .failure()
        .stdout(predicate::str::contains("jones2023"));
}
