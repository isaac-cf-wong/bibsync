---
name: 🐛 Bug Report
about: Report unexpected behavior in the CLI, Python API, or pre-commit hook
title: "[BUG]: "
labels: bug
assignees: ""
---

## 📝 Description

A clear and concise description of what went wrong.

## 🚀 Reproduction Steps

Steps to reproduce the behavior:

1. ...
2. ...
3. ...

## 💻 Minimal Reproducible Example

Provide the smallest example that triggers the issue. Include the interface you used.

**CLI** (preferred when reporting command-line behavior):

```shell
bibsync --fix main.tex -o references.bib --provider inspire
```

**Python API** (if the issue is in the PyPI bindings):

```python
import bibsync

report = bibsync.sync_files(["main.tex"], output="references.bib", provider="inspire")
```

Attach or paste minimal `.tex` / `.bib` inputs when they matter.
Redact secrets (API tokens, private paths, unpublished manuscript text).

## 📋 Expected Behavior

What you expected `bibsync` to do.

## 💥 Actual Behavior / Output

Paste the command output, error message, or traceback **after redacting secrets**
(`ADS_API_TOKEN`, credentials, private URLs/paths, personal data).

```text
(paste output here)
```

If this may be a **security vulnerability**, do not post details publicly.
Use [GitHub private vulnerability reporting](https://github.com/isaac-cf-wong/bibsync/security/advisories/new)
(see [SECURITY.md](https://github.com/isaac-cf-wong/bibsync/blob/main/SECURITY.md)).

## 🛠 Environment

- **bibsync version:** (e.g. `bibsync --version`, crates.io, PyPI, or git commit)
- **How installed:** (e.g. `cargo install`, PyPI `pip install`, GitHub release binary, pre-commit hook)
- **Python version** (if using PyPI bindings or pytest): (e.g. 3.13)
- **Rust toolchain** (if built from source): (e.g. stable 1.86)
- **Operating system:** (e.g. macOS 15, Ubuntu 24.04, Windows 11)
- **Provider / flags:** (e.g. `--provider ads`, `--cache`, `--fix`)

## 📎 Additional Context

Logs, screenshots, or related issues/discussions.
