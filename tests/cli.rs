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

/// One documented hook configuration, reduced to what pre-commit uses to
/// decide whether and how to run it.
struct DocumentedHook {
    source: String,
    files: regex::Regex,
    extensions: Vec<&'static str>,
    pass_filenames: bool,
    args: Vec<String>,
}

/// File extensions pre-commit matches for each `types_or` tag bibsync uses.
fn tag_extensions(tag: &str) -> &'static [&'static str] {
    match tag {
        "tex" => &["tex"],
        "bib" => &["bib"],
        "markdown" => &["md"],
        other => panic!("unexpected file type tag {other}"),
    }
}

/// `[a, b, c]` (possibly spanning lines) after `key:` in `block`.
fn yaml_list(block: &str, key: &str) -> Option<Vec<String>> {
    let after = block.split_once(&format!("{key}:"))?.1;
    let list = after.split_once('[')?.1.split_once(']')?.0;
    Some(
        list.split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_owned)
            .collect(),
    )
}

fn yaml_scalar<'a>(block: &'a str, key: &str) -> Option<&'a str> {
    block
        .lines()
        .find_map(|line| line.trim().strip_prefix(&format!("{key}:")))
        .map(str::trim)
}

/// Every bibsync hook configuration in the README and the pre-commit guide.
fn documented_hooks() -> Vec<DocumentedHook> {
    let root = env!("CARGO_MANIFEST_DIR");
    // Git may check these files out with CRLF line endings (Windows).
    let read = |path: &str| {
        std::fs::read_to_string(format!("{root}/{path}"))
            .unwrap_or_else(|error| panic!("read {path}: {error}"))
            .replace("\r\n", "\n")
    };
    let manifest = read(".pre-commit-hooks.yaml");
    let manifest_types = |id: &str| {
        let hook = manifest
            .split("- id: ")
            .find(|hook| hook.starts_with(&format!("{id}\n")))
            .unwrap_or_else(|| panic!("manifest has hook {id}"));
        yaml_list(hook, "types_or").expect("manifest hook has types_or")
    };
    let mut hooks = Vec::new();
    for doc in ["README.md", "docs/user-guide/pre-commit.md"] {
        let text = read(doc);
        for (index, block) in text.split("```yaml").skip(1).enumerate() {
            let block = block.split_once("```").expect("closed yaml block").0;
            let Some(id) = yaml_scalar(block, "- id") else {
                continue;
            };
            let local = block.contains("repo: local");
            let types = if local {
                yaml_list(block, "types_or")
                    .or_else(|| yaml_list(block, "types"))
                    .unwrap_or_else(|| vec!["tex".into(), "bib".into(), "markdown".into()])
            } else {
                manifest_types(id)
            };
            let args = if local {
                let entry = yaml_scalar(block, "entry").expect("local hook has entry");
                let args = entry.split_once(" -- ").map_or(entry, |(_, args)| args);
                args.split_whitespace().map(str::to_owned).collect()
            } else {
                yaml_list(block, "args").unwrap_or_default()
            };
            hooks.push(DocumentedHook {
                source: format!("{doc} yaml block {}", index + 1),
                files: regex::Regex::new(yaml_scalar(block, "files").unwrap_or(""))
                    .expect("files is a regex"),
                extensions: types
                    .iter()
                    .flat_map(|tag| tag_extensions(tag))
                    .copied()
                    .collect(),
                pass_filenames: yaml_scalar(block, "pass_filenames") != Some("false"),
                args,
            });
        }
    }
    hooks
}

/// Every documented hook configuration must check the citations whether a
/// commit stages only the bibliography (a cited entry deleted) or only the
/// citing document (a new citation). pre-commit runs a hook only when a staged
/// file matches its file types and `files:`, and passes it only those files
/// unless `pass_filenames: false`; a hook that relies on that list sees one
/// side alone and passes.
#[test]
fn documented_hooks_check_citations_when_either_file_changes() {
    let hooks = documented_hooks();
    assert!(hooks.len() >= 8, "found only {} hook examples", hooks.len());
    for hook in hooks {
        let output = hook
            .args
            .iter()
            .position(|arg| arg == "--output")
            .and_then(|index| hook.args.get(index + 1))
            .unwrap_or_else(|| panic!("{}: no --output", hook.source))
            .clone();
        let documents = hook
            .args
            .iter()
            .filter(|arg| {
                std::path::Path::new(arg)
                    .extension()
                    .is_some_and(|extension| extension == "md" || extension == "tex")
            })
            .cloned()
            .collect::<Vec<_>>();
        assert!(
            !documents.is_empty(),
            "{}: no citing document in args",
            hook.source
        );
        for staged in [vec![output.clone()], documents.clone()] {
            let dir = tempdir().expect("tempdir");
            std::fs::write(
                dir.path().join("paper.md"),
                "---\ntitle: Example\nbibliography: paper.bib\n---\n\nText [@smith2024; @jones2023].\n",
            )
            .expect("write paper.md");
            std::fs::write(
                dir.path().join("main.tex"),
                "\\cite{smith2024,jones2023}\n\\bibliography{references}\n",
            )
            .expect("write main.tex");
            for bib in ["paper.bib", "references.bib"] {
                std::fs::write(
                    dir.path().join(bib),
                    "@misc{smith2024,\n  title = {Kept}\n}\n",
                )
                .expect("write bib");
            }
            std::fs::write(dir.path().join(".bibsyncignore"), "").expect("write ignore file");

            let matched = staged
                .iter()
                .filter(|path| {
                    hook.files.is_match(path)
                        && hook
                            .extensions
                            .iter()
                            .any(|extension| path.ends_with(&format!(".{extension}")))
                })
                .collect::<Vec<_>>();
            assert!(
                !matched.is_empty(),
                "{}: the hook does not run when only {staged:?} changes",
                hook.source
            );
            let mut command = Command::cargo_bin("bibsync").expect("binary exists");
            command
                .current_dir(dir.path())
                .env("BIBSYNC_CACHE_DIR", dir.path().join("cache"))
                .args(&hook.args);
            if hook.pass_filenames {
                command.args(&matched);
            }
            let result = command.output().expect("run bibsync");
            assert!(
                !result.status.success()
                    && String::from_utf8_lossy(&result.stdout).contains("jones2023"),
                "{}: passed when only {staged:?} changed\nstdout: {}\nstderr: {}",
                hook.source,
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr),
            );
        }
    }
}

#[test]
fn cli_accepts_full_journal_names() {
    let dir = tempdir().expect("tempdir");
    let bib = dir.path().join("refs.bib");
    std::fs::write(
        &bib,
        "@article{smith2024,\n  author = {Smith, Jane},\n  journal = {Phys. Rev. Z},\n  eprint = {2001.00001},\n  archivePrefix = {arXiv},\n  year = {2024}\n}\n",
    )
    .expect("write bib");
    // The entry is not re-resolved (it has a journal), so no network is used;
    // this checks that the flag is accepted and the run stays clean.
    let mut command = Command::cargo_bin("bibsync").expect("binary exists");
    command
        .arg("--provider")
        .arg("inspire")
        .arg("--full-journal-names")
        .arg(&bib)
        .assert()
        .success()
        .stdout(predicate::str::contains("bibliography is up to date"));
}
