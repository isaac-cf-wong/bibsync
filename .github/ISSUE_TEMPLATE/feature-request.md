---
name: ✨ Feature Request
about: Suggest an improvement to bibsync
title: "[FEATURE]: "
labels: enhancement
assignees: ""
---

## 🎯 Problem Statement

Is your request related to a problem? Describe the workflow or limitation.

> _Ex: "When syncing a thesis with many hand-curated entries, I need..."_

## 💡 Proposed Solution

A clear description of the behavior you want
(CLI flag, default change, provider behavior, Python API, pre-commit hook, etc.).

## 💻 Proposed Usage

Show how you would use the feature, if applicable.

**CLI:**

```shell
bibsync --fix main.tex -o references.bib --your-flag value
```

**Python API:**

```python
import bibsync

report = bibsync.sync_files(["main.tex"], output="references.bib")
```

**LaTeX / BibTeX** (if citekey or bibliography semantics matter):

```tex
\citep{2404.14498}
```

## 🌈 Use Case & Benefits

Why this helps you and other users
(LaTeX manuscripts, shared `.bib` files, CI/pre-commit, provider coverage, etc.).

## 🔄 Alternatives Considered

Other tools, flags, or workarounds you tried (e.g. manual `.bib` edits, `--no-update`, `.bibsyncignore`).

## ⚠️ Additional Context

Links to similar tools, provider API docs, or examples from other projects.
