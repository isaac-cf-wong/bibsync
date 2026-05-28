# bibsync

[![CI](https://github.com/isaac-cf-wong/bibsync/actions/workflows/ci.yml/badge.svg)](https://github.com/isaac-cf-wong/bibsync/actions/workflows/ci.yml)
[![Documentation Status](https://github.com/isaac-cf-wong/bibsync/actions/workflows/documentation.yml/badge.svg)](https://isaac-cf-wong.github.io/bibsync/)
[![Crates.io](https://img.shields.io/crates/v/bibsync)](https://crates.io/crates/bibsync)
[![Docs.rs](https://docs.rs/bibsync/badge.svg)](https://docs.rs/bibsync)
[![License](https://img.shields.io/badge/License-BSD_3--Clause-blue.svg)](LICENSE)

`bibsync` synchronizes BibTeX files from citation keys in LaTeX sources. It is
inspired by `adstex`, with provider support for both NASA ADS and InspireHEP.

The primary workflow is to cite papers by identifier, especially arXiv ID:

```tex
\citep{2404.14498}
\citet{arXiv:2312.00752}
```

Then check the bibliography:

```shell
bibsync main.tex -o references.bib
```

To update the file, add `--fix`:

```shell
bibsync --fix main.tex -o references.bib
```

`bibsync` scans the TeX file, resolves missing identifier-like citekeys through
NASA ADS and/or InspireHEP, rewrites provider BibTeX entries so the citekey stays
the key used in TeX, and reports whether the output `.bib` file is current. With
`--fix`, it writes the merged bibliography.

## Providers

By default `bibsync` tries NASA ADS first and InspireHEP second:

```shell
bibsync --fix main.tex -o references.bib --provider auto
```

NASA ADS requires an API token:

```shell
export ADS_API_TOKEN="..."
```

You can choose a single provider:

```shell
bibsync --fix main.tex -o references.bib --provider ads
bibsync --fix main.tex -o references.bib --provider inspire
```

InspireHEP supports arXiv IDs and DOIs. NASA ADS supports arXiv IDs, DOIs, and
ADS bibcodes.

## Existing Bibliographies

If the TeX source contains `\bibliography{references}`, `bibsync` can discover
`references.bib` automatically:

```shell
bibsync --fix main.tex
```

Additional read-only bibliographies can be used to avoid duplicating entries:

```shell
bibsync --fix main.tex -o references.bib -r shared.bib software.bib
```

Use `--merge-other` to copy matching entries from those read-only files into the
main output file.

To update a bibliography in place, pass a single `.bib` file:

```shell
bibsync --fix references.bib --force-regenerate
```

### Update Behavior

By default `bibsync` leaves published entries untouched. Only entries that look
like unpublished preprints — those with an `archivePrefix` or `eprinttype` field
but no `journal` field — are re-queried to check whether they have been
published. If so, the entry is updated; otherwise it is preserved.

| Flag                 | Behavior                                      |
| -------------------- | --------------------------------------------- |
| _(default)_          | Re-check preprints; skip published entries    |
| `--no-update`        | Skip all existing entries                     |
| `--update-all`       | Re-resolve all existing entries               |
| `--force-regenerate` | Re-resolve and overwrite all existing entries |

### Ignoring Entries

To exclude specific entries from all resolution — for example, books or theses
you have curated by hand — list their citekeys in a `.bibsyncignore` file:

```text
# .bibsyncignore
knuth1997art
smith2024thesis
```

Pass the file with `--ignore-file`:

```shell
bibsync --fix main.tex -o references.bib --ignore-file .bibsyncignore
```

## Cache

Use `--cache` to avoid repeated provider API calls:

```shell
bibsync --cache main.tex -o references.bib
bibsync --fix --cache main.tex -o references.bib
```

The cache stores provider records and mappings from arXiv IDs or DOIs to the
provider's canonical record ID. Preprint entries that are re-checked for
publication always bypass the cache and fetch a fresh result, then write it back.
Use `--refresh-cache` to force a fresh fetch for all entries:

```shell
bibsync --fix --refresh-cache main.tex -o references.bib
```

Override the cache location with `--cache-dir DIR`.

## Pre-commit

The repository includes `.pre-commit-hooks.yaml`, so other projects can use
`bibsync` as a pre-commit hook.

Use the pre-built binary hook for faster installs:

```yaml
repos:
    - repo: https://github.com/isaac-cf-wong/bibsync
      rev: v0.1.0
      hooks:
          - id: bibsync-bin
            args: [--cache, --provider, inspire, --output, references.bib]
```

The binary hook downloads a platform-specific archive from the GitHub release
matching `rev` and caches it under pre-commit's cache directory. The source hook
is available for Linux x86_64/aarch64, macOS x86_64/aarch64, and Windows x86_64.
The source hook is still available, but it compiles the Rust crate during hook
installation:

```yaml
repos:
    - repo: https://github.com/isaac-cf-wong/bibsync
      rev: v0.1.0
      hooks:
          - id: bibsync
            args: [--provider, inspire, --output, references.bib]
```

By default, the hook checks whether the bibliography is current and fails without
writing changes. To let the hook update files, add `--fix` to the hook args:

```yaml
repos:
    - repo: https://github.com/isaac-cf-wong/bibsync
      rev: v0.1.0
      hooks:
          - id: bibsync-bin
            args: [--fix, --cache, --provider, inspire, --output, references.bib]
```

To skip manually curated entries, add `--ignore-file`:

```yaml
repos:
    - repo: https://github.com/isaac-cf-wong/bibsync
      rev: v0.1.0
      hooks:
          - id: bibsync-bin
            args:
                [
                    --fix,
                    --cache,
                    --provider,
                    inspire,
                    --output,
                    references.bib,
                    --ignore-file,
                    .bibsyncignore,
                ]
```

For a project-local hook while developing `bibsync` itself:

```yaml
repos:
    - repo: local
      hooks:
          - id: bibsync
            name: bibsync
            entry: cargo run -- --fix --provider inspire --output references.bib
            language: system
            files: \.tex$
```
