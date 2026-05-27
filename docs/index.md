# Rust Package Template

This repository is a starting point for Rust crates that need a library,
optional CLI, tests, documentation, CI, release automation, and project hygiene
files.

## Local Checks

```shell
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## Example

```rust
use rust_package_template::greeting;

assert_eq!(greeting("Ferris").unwrap(), "Hello, Ferris!");
```
