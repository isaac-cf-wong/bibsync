# bibsync

[![CI](https://github.com/isaac-cf-wong/bibsync/actions/workflows/ci.yml/badge.svg)](https://github.com/isaac-cf-wong/bibsync/actions/workflows/ci.yml)
[![Documentation Status](https://github.com/isaac-cf-wong/bibsync/actions/workflows/documentation.yml/badge.svg)](https://isaac-cf-wong.github.io/bibsync/)
[![Crates.io](https://img.shields.io/crates/v/bibsync)](https://crates.io/crates/bibsync)
[![Docs.rs](https://docs.rs/bibsync/badge.svg)](https://docs.rs/bibsync)
[![PyPI Version](https://img.shields.io/pypi/v/bibsync)](https://pypi.org/project/bibsync/)
[![Python Versions](https://img.shields.io/pypi/pyversions/bibsync)](https://pypi.org/project/bibsync/)
[![License](https://img.shields.io/badge/License-BSD_3--Clause-blue.svg)](LICENSE)
[![Ruff](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ruff/main/assets/badge/v2.json)](https://github.com/astral-sh/ruff)
[![DOI](https://zenodo.org/badge/1251520994.svg)](https://doi.org/10.5281/zenodo.20422622)
[![SPEC 0 — Minimum Supported Dependencies](https://img.shields.io/badge/SPEC-0-green?labelColor=%23004811&color=%235CA038)](https://scientific-python.org/specs/spec-0000/)

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

Markdown sources are supported too, including
[Journal of Open Source Software](https://joss.theoj.org/) `paper.md` files.
`bibsync` reads pandoc citations and the `bibliography` field from the YAML
frontmatter:

```markdown
---
bibliography: paper.bib
---

Prior work [@1602.03837; @10.1103/PhysRevLett.116.061102].
```

```shell
bibsync --fix paper.md
```

When a citekey cannot be resolved, the command prints the key together with a
reason and the likely fix. For example, an unsupported citekey is reported as an
identifier-format problem, while an arXiv ID or DOI that the provider cannot find
is reported as a provider miss.

## Installation

Install the Rust CLI from crates.io:

```shell
cargo install bibsync
```

Install the Python package from PyPI:

```shell
pip install bibsync
```

The PyPI package includes Python bindings and installs the `bibsync` command.
Python 3.13 or newer is required.

You can also download a pre-built binary from the
[GitHub releases page](https://github.com/isaac-cf-wong/bibsync/releases). Pick
the archive for your platform, extract it, and place the `bibsync` executable
somewhere on your `PATH`.

To compile from source:

```shell
git clone https://github.com/isaac-cf-wong/bibsync.git
cd bibsync
cargo build --release
```

The compiled binary is written to `target/release/bibsync` on Unix-like systems
or `target\release\bibsync.exe` on Windows.

## Citing

If `bibsync` contributes to a scientific publication, please cite it using the
Zenodo record:

[https://doi.org/10.5281/zenodo.20422622](https://doi.org/10.5281/zenodo.20422622)

Citation formats including BibTeX can be exported directly from the Zenodo page.
For convenience, you can also use:

```bibtex
@software{wong2026bibsync,
  author  = {Wong, Isaac C. F.},
  title   = {bibsync: A Rust package to automatically resolve, synchronize, and validate LaTeX citations across BibTeX databases},
  version = {v1.0.2},
  year    = {2026},
  month   = nov,
  doi     = {10.5281/zenodo.20422622},
  url     = {https://doi.org/10.5281/zenodo.20422622}
}
```

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

## Python API

`bibsync` can also be installed from PyPI:

```shell
pip install bibsync
```

The PyPI package provides Python bindings backed by the Rust implementation:

```python
import bibsync

report = bibsync.sync_files(
    ["main.tex"],
    output="references.bib",
    provider="inspire",
    check=True,
)
```

It also installs the `bibsync` command. The command delegates to the same Rust
CLI implementation as the Cargo-installed binary, so command-line behavior is
kept in one place.

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

Existing input files are validated before resolution. A missing single `.bib`
input, `--other` bibliography, or `--ignore-file` is reported as an error with
the path that could not be read. Existing bibliography files are also parsed
strictly, so malformed BibTeX reports the file and the approximate failing
entry instead of being treated as an empty or partial bibliography.

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

### Author Lists

InspireHEP and NASA ADS truncate long author lists in their BibTeX
(`Abac, A. G. and others`, `... and et al.`). Citation styles that treat
`others` as a name, such as the one JOSS uses, then print "Abac & others"
instead of "Abac et al.". Add `--full-author-lists` to fetch the complete list
from the provider:

```shell
bibsync --fix paper.md --full-author-lists
```

Papers with more than 100 authors that name a collaboration are credited to
the collaboration(s) instead, as one brace-protected corporate author, for
example `{LIGO Scientific Collaboration, VIRGO Collaboration and KAGRA
Collaboration}`. The option is off by default, costs one extra provider request
per truncated entry, and, like `--full-journal-names`, applies only to entries
resolved from a provider in that run (use `--force-regenerate` once to convert
an existing file). An entry whose provider has no full list is left truncated
and reported.

### Journal Names

Providers abbreviate journal names: InspireHEP writes `Phys. Rev. D` and NASA
ADS writes AAS macros such as `\prd`. Some venues, including JOSS, require full
names. Add `--full-journal-names` to write them in full:

```shell
bibsync --fix paper.md --full-journal-names
```

This applies to entries resolved from a provider in that run (new entries,
refreshed preprints, and everything with `--update-all` or
`--force-regenerate`); entries already in the bibliography are not rewritten,
so run once with `--force-regenerate` to convert an existing file. The full
names come from a table maintained in `bibsync`
([`src/journals.rs`](src/journals.rs)), which also lists journals whose short
title is already their full name (`Nature`, `SoftwareX`). Any other journal is
left unchanged and reported, never guessed, so you can check it by hand:

```text
bibsync: journal "Phys. Rev. Z" in 2001.00001 is not in bibsync's journal table; left unchanged, check that it is the full name
```

To catch abbreviations that `--full-journal-names` cannot fix (journals missing
from the table, and hand-written or ignored entries), add `--check-venue-names`.
It checks the `journal` and `booktitle` of every entry in the bibliography,
including entries listed in the ignore file, names each one that looks
abbreviated, and makes the run fail:

```text
bibsync: journal in smith2024 looks abbreviated: "Phys. Rev. D"; write the full venue name
```

An abbreviation in bibsync's journal table is always reported and a full name
in it never is, even when written with a period where the table has a colon
(`Journal of Physics. Conference Series`). Any other name counts as abbreviated
when it is an unexpanded macro (`\prd`, `prd`), contains a word cut short with
a period (`Phys.`, `J.Phys.`; `St.`, `Ste.`, `Mt.` and `Ft.` are allowed), or is
a single short word with two or more capitals (`JCAP`). Concatenated values and
entries whose fields bibsync cannot read are reported so you can check them by
hand.

That last rule is a heuristic, so a real full title such as `RNA` can be
reported. List the venue names to accept, one per line, in a file and pass it
with `--accepted-venues`:

```text
# accepted-venues.txt
RNA
```

```shell
bibsync paper.md --check-venue-names --accepted-venues accepted-venues.txt
```

For a JOSS paper's pre-commit hook, use both options together.

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

If a cache file is corrupt, `bibsync` reports the exact cache path and asks you
to refresh or remove the bad cache entry. Provider request failures include the
provider and citekey or batch being resolved.

## Pre-commit

The repository includes `.pre-commit-hooks.yaml`, so other projects can use
`bibsync` as a pre-commit hook.

Both hooks run on TeX (`.tex`), BibTeX (`.bib`), and Markdown (`.md`) files.
Every `@key` in a scanned file is treated as a citation, so scanning unrelated
Markdown can raise false positives — for example a `README.md` that uses GitHub
`@mentions`. **The recommended setup is to scope the hook to your citation
sources with `files:`** rather than relying on the default file matching.

For a JOSS paper (`paper.md` + `paper.bib`):

```yaml
repos:
    - repo: https://github.com/isaac-cf-wong/bibsync
      rev: v1.0.2
      hooks:
          - id: bibsync-bin
            files: ^paper\.(md|bib)$
            pass_filenames: false
            args: [--fix, --cache, --provider, inspire, --output, paper.bib, paper.md]
```

**Name the citing document in `args` and set `pass_filenames: false`.**
pre-commit passes the hook only the staged files that match. When a commit
changes `paper.bib` but not `paper.md`, `bibsync` would receive only
`paper.bib`; it then checks that file's own entries and never reads which keys
`paper.md` cites, so deleting a cited entry from `paper.bib` would pass. With
`pass_filenames: false` the hook always scans `paper.md`, and `files:` makes it
run whenever either file changes. For a LaTeX project, list the `.tex` files
that cite (for example `main.tex`) in `args` and match them and the `.bib` file
in `files:`.

**Every configuration needs both settings.** The examples below use a LaTeX
project whose `main.tex` cites into `references.bib`; replace those names with
your own citing files and bibliography in both `files:` and `args`.

Use the pre-built binary hook for faster installs:

```yaml
repos:
    - repo: https://github.com/isaac-cf-wong/bibsync
      rev: v1.0.2
      hooks:
          - id: bibsync-bin
            files: ^(main\.tex|references\.bib)$
            pass_filenames: false
            args: [--cache, --provider, inspire, --output, references.bib, main.tex]
```

The binary hook downloads a platform-specific archive from the GitHub release
matching `rev` and caches it under pre-commit's cache directory. The source hook
is available for Linux x86_64/aarch64, macOS x86_64/aarch64, and Windows x86_64.
The source hook is still available, but it compiles the Rust crate during hook
installation:

```yaml
repos:
    - repo: https://github.com/isaac-cf-wong/bibsync
      rev: v1.0.2
      hooks:
          - id: bibsync
            files: ^(main\.tex|references\.bib)$
            pass_filenames: false
            args: [--provider, inspire, --output, references.bib, main.tex]
```

By default, the hook checks whether the bibliography is current and fails without
writing changes. To let the hook update files, add `--fix` to the hook args:

```yaml
repos:
    - repo: https://github.com/isaac-cf-wong/bibsync
      rev: v1.0.2
      hooks:
          - id: bibsync-bin
            files: ^(main\.tex|references\.bib)$
            pass_filenames: false
            args: [--fix, --cache, --provider, inspire, --output, references.bib, main.tex]
```

To skip manually curated entries, add `--ignore-file`:

```yaml
repos:
    - repo: https://github.com/isaac-cf-wong/bibsync
      rev: v1.0.2
      hooks:
          - id: bibsync-bin
            files: ^(main\.tex|references\.bib)$
            pass_filenames: false
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
                    main.tex,
                ]
```

For a project-local hook while developing `bibsync` itself:

```yaml
repos:
    - repo: local
      hooks:
          - id: bibsync
            name: bibsync
            entry: cargo run -- --fix --provider inspire --output references.bib main.tex
            language: system
            files: ^(main\.tex|references\.bib)$
            pass_filenames: false
```
