//! Integration tests for checked-in examples.

use std::path::{Path, PathBuf};
use std::process::Command;

fn citekeys(path: &str) -> Vec<String> {
    bibsync::citation_keys(&[PathBuf::from(path)])
        .expect("example TeX can be scanned")
        .into_iter()
        .collect()
}

fn bib_contains_key(path: &str, key: &str) -> bool {
    let content = std::fs::read_to_string(path).expect("example BibTeX can be read");
    content.contains(&format!("{{{key},"))
}

#[test]
fn inspire_example_bibliography_matches_tex_citekeys() {
    for key in citekeys("examples/inspire-main.tex") {
        assert!(
            bib_contains_key("examples/inspire-main.bib", &key),
            "examples/inspire-main.bib is missing {key}"
        );
    }
}

#[test]
fn ads_example_bibliography_matches_tex_citekeys() {
    for key in citekeys("examples/main.tex") {
        assert!(
            bib_contains_key("examples/main.bib", &key),
            "examples/main.bib is missing {key}"
        );
    }
}

#[test]
#[ignore = "requires a local TeX installation; run with `cargo test --test examples -- --ignored`"]
fn example_latex_files_compile() {
    if Command::new("latexmk").arg("-v").output().is_err() {
        eprintln!("latexmk is not installed; skipping LaTeX compile smoke test");
        return;
    }

    for tex in ["main.tex", "inspire-main.tex"] {
        let status = Command::new("latexmk")
            .args(["-pdf", "-interaction=nonstopmode", "-halt-on-error", tex])
            .current_dir(Path::new("examples"))
            .status()
            .expect("latexmk can be launched");
        assert!(status.success(), "{tex} did not compile");

        let _ = Command::new("latexmk")
            .args(["-c", tex])
            .current_dir(Path::new("examples"))
            .status();
    }
}
