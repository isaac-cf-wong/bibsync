//! Integration tests for the sample command-line interface.

use assert_cmd::Command;
use predicates::prelude::predicate;

#[test]
fn cli_prints_default_greeting() {
    let mut command = Command::cargo_bin("rust-package-template").expect("binary exists");

    command
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, world!"));
}

#[test]
fn cli_prints_named_greeting() {
    let mut command = Command::cargo_bin("rust-package-template").expect("binary exists");

    command
        .arg("Ferris")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, Ferris!"));
}
