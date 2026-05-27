# Testing

Use unit tests near implementation code and integration tests under `tests/`.

```shell
cargo test --all-features
```

The CI workflow also runs formatting, clippy, docs, and dependency audit checks.
