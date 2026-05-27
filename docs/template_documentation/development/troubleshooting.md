# Troubleshooting

If `setup_repo.sh` fails, confirm that `remote.origin.url` points to the new
repository and that the target crate paths do not already exist.

If clippy fails in generated or experimental code, fix the warning or narrow the
lint setting in `Cargo.toml` with a clear reason.
