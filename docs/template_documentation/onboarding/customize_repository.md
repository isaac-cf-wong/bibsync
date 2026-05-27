# Customize And Clean Up

The setup script renames the crate, Rust import path, binary name, repository
URLs, and docs references from `rust-package-template` to the new repository
name.

After running it, review:

- `Cargo.toml` description, keywords, categories, and crate features
- `README.md` badges and getting-started text
- `docs/` pages
- `.github/workflows/` release and publication settings
- `SECURITY.md` support policy

Delete `docs/template_documentation/` when it is no longer useful.
