# Rust Package Template

[![CI](https://github.com/isaac-cf-wong/rust-package-template/actions/workflows/ci.yml/badge.svg)](https://github.com/isaac-cf-wong/rust-package-template/actions/workflows/ci.yml)
[![Documentation Status](https://github.com/isaac-cf-wong/rust-package-template/actions/workflows/documentation.yml/badge.svg)](https://isaac-cf-wong.github.io/rust-package-template/)
[![Crates.io](https://img.shields.io/crates/v/rust-package-template)](https://crates.io/crates/rust-package-template)
[![Docs.rs](https://docs.rs/rust-package-template/badge.svg)](https://docs.rs/rust-package-template)
[![License](https://img.shields.io/badge/License-BSD_3--Clause-blue.svg)](LICENSE)

This project is a template for creating Rust packages with a standardized
library, CLI, documentation, testing, linting, release, and repository
customization setup.

## Getting Started

Use **Use this template** on GitHub, then read
[`docs/template_documentation/`](docs/template_documentation/) for onboarding,
development, and customization notes. Delete that directory and the
**Template documentation** `nav` block in `zensical.toml` when you no longer
need them. Develop your library under `src/`.

```shell
cargo test
cargo run -- Ferris
```
