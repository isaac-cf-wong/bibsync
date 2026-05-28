# bibsync

`bibsync` synchronizes BibTeX files from citation keys in LaTeX sources. It is
designed for papers where the citation key can be a stable scholarly identifier,
especially an arXiv ID:

```tex
\citep{2404.14498}
\citet{arXiv:2312.00752}
```

After scanning the TeX source, `bibsync` resolves missing entries through NASA
ADS or InspireHEP, rewrites the provider's BibTeX entry so the citekey stays the
same as the TeX citekey, and checks whether the target `.bib` file is current.

```shell
bibsync main.tex -o references.bib
```

Add `--fix` to write the merged bibliography:

```shell
bibsync --fix main.tex -o references.bib
```

## What It Does

`bibsync` handles the repetitive part of bibliography maintenance:

- Finds citekeys in common LaTeX citation commands such as `\cite`, `\citet`,
  `\citep`, and related variants.
- Resolves identifier-like citekeys through NASA ADS and InspireHEP.
- Preserves arXiv IDs, DOIs, and ADS bibcodes as citekeys when writing BibTeX.
- Merges generated entries with an existing bibliography instead of replacing
  the whole file blindly when `--fix` is provided.
- Checks whether a bibliography is up to date by default, which makes it
  suitable for safe pre-commit hooks.

## Supported Identifiers

NASA ADS can resolve:

- arXiv IDs, for example `1602.03837`
- DOIs, for example `10.1103/PhysRevLett.116.061102`
- ADS bibcodes, for example `2016PhRvL.116f1102A`

InspireHEP can resolve:

- arXiv IDs
- DOIs

Author-year interactive lookup, as provided by `adstex`, is intentionally not
part of the initial `bibsync` workflow. The current implementation focuses on
deterministic identifier-based synchronization.

## Quick Start

Install `bibsync`, then choose a provider:

```shell
cargo install bibsync
```

or:

```shell
pip install bibsync
```

Then run:

```shell
bibsync --fix main.tex -o references.bib --provider inspire
```

If you use NASA ADS, set an API token first:

```shell
export ADS_API_TOKEN="..."
bibsync --fix main.tex -o references.bib --provider ads
```

The default provider mode is `auto`. In that mode, `bibsync` uses NASA ADS when
`ADS_API_TOKEN` is available and then falls back to InspireHEP.

## Where To Go Next

- Read the [installation guide](user-guide/installation.md) for Cargo, PyPI,
  pre-built binaries, and source builds.
- Read the [usage guide](user-guide/usage.md) for command-line workflows.
- Read the [provider guide](user-guide/providers.md) to choose between NASA ADS,
  InspireHEP, and `auto`.
- Read the [pre-commit guide](user-guide/pre-commit.md) to enforce synchronized
  bibliographies in a repository.
- See the [examples guide](user-guide/examples.md) for tested example files.
