---
name: 🚀 Pull Request
about: Submit your changes for review
---

## 📝 Summary

Briefly describe the changes in this PR. Link related issues (e.g. `Closes #123`).

Use a [Conventional Commits](https://www.conventionalcommits.org/) style **PR title** (enforced by CI),
for example `feat: add --foo flag` or `fix: handle empty .bib files`.

## ✨ Type of Change

- [ ] 🐛 Bug fix (non-breaking change which fixes an issue)
- [ ] ✨ New feature (non-breaking change which adds functionality)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality
      to not work as expected)
- [ ] 📝 Documentation update
- [ ] 🎨 Refactoring (no functional changes, no API changes)

## 🧪 How Has This Been Tested?

Describe what you ran locally. Check what applies:

- [ ] **Rust:** `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`
- [ ] **Python:** `uv sync --group dev --frozen` and `uv run --frozen pytest`
- [ ] **Manual:** (e.g. `bibsync --fix` on a sample `.tex` / `.bib`, Python API call, pre-commit hook)
- [ ] **Docs:** built or previewed site changes under `docs/` if applicable

## 🏗️ Checklist

- [ ] I read [CONTRIBUTING.md](https://github.com/isaac-cf-wong/bibsync/blob/main/CONTRIBUTING.md).
- [ ] Rust changes follow `rustfmt` and pass `clippy` with `-D warnings`.
- [ ] Python changes follow **Ruff** formatting/linting (`pyproject.toml`).
- [ ] I updated user-facing docs (`README.md`, `docs/`, or rustdoc) when behavior changed.
- [ ] I added or updated tests where behavior changed (Rust integration/unit tests and/or Python tests under `tests/`).

## 📷 Screenshots (if applicable)

If this is a UI change or produces a specific output/graph, add screenshots
here.

## 🔍 Notes for reviewers

Optional: areas that need extra scrutiny (provider APIs, citekey rewriting, PyO3 bindings, release artifacts).
