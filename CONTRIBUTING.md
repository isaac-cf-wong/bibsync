# Contributing to rust-package-template

Thank you for your interest in contributing to `rust-package-template`.

## Development

1. Fork and clone the repository.
2. Install the stable Rust toolchain with `rustup`.
3. Install optional local tools:

    ```shell
    cargo install cargo-deny cargo-audit cargo-nextest git-cliff
    pipx install pre-commit
    ```

4. Install hooks:

    ```shell
    pre-commit install
    ```

5. Run the local checks:

    ```shell
    cargo fmt --all --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --all-features
    ```

Use Conventional Commits for commit messages and pull request titles. The
changelog is generated with `git-cliff`.

## Licensing

By contributing, you agree that your contributions will be licensed under the
project's 3-Clause BSD License.
