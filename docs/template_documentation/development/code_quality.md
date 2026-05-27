# Code Quality

The default quality gate is intentionally simple:

```shell
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Add crate-specific checks as the project grows.
