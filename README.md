# bibsync

`bibsync` synchronizes BibTeX files from citation keys in LaTeX sources. It is
inspired by `adstex`, with provider support for both NASA ADS and InspireHEP.

The primary workflow is to cite papers by identifier, especially arXiv ID:

```tex
\citep{2404.14498}
\citet{arXiv:2312.00752}
```

Then run:

```shell
bibsync main.tex -o references.bib
```

`bibsync` scans the TeX file, resolves missing identifier-like citekeys through
NASA ADS and/or InspireHEP, rewrites provider BibTeX entries so the citekey stays
the key used in TeX, and merges the result into the output `.bib` file.

## Providers

By default `bibsync` tries NASA ADS first and InspireHEP second:

```shell
bibsync main.tex -o references.bib --provider auto
```

NASA ADS requires an API token:

```shell
export ADS_API_TOKEN="..."
```

You can choose a single provider:

```shell
bibsync main.tex -o references.bib --provider ads
bibsync main.tex -o references.bib --provider inspire
```

InspireHEP supports arXiv IDs and DOIs. NASA ADS supports arXiv IDs, DOIs, and
ADS bibcodes.

## Existing Bibliographies

If the TeX source contains `\bibliography{references}`, `bibsync` can discover
`references.bib` automatically:

```shell
bibsync main.tex
```

Additional read-only bibliographies can be used to avoid duplicating entries:

```shell
bibsync main.tex -o references.bib -r shared.bib software.bib
```

Use `--merge-other` to copy matching entries from those read-only files into the
main output file.

To update a bibliography in place, pass a single `.bib` file:

```shell
bibsync references.bib --force-regenerate
```

## Pre-commit

The repository includes `.pre-commit-hooks.yaml`, so other projects can use
`bibsync` as a pre-commit hook.

Example configuration:

```yaml
repos:
    - repo: https://github.com/isaac-cf-wong/bibsync
      rev: v0.1.0
      hooks:
          - id: bibsync
            args: [--provider, inspire, --output, references.bib]
```

The hook runs `bibsync --check`. If the bibliography is out of date, the hook
fails without writing changes. Run the same command without `--check` to update
the file:

```shell
bibsync --provider inspire --output references.bib main.tex
```

For a project-local hook while developing `bibsync` itself:

```yaml
repos:
    - repo: local
      hooks:
          - id: bibsync
            name: bibsync
            entry: cargo run -- --check --provider inspire --output references.bib
            language: system
            files: \.tex$
```
