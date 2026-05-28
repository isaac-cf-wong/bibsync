# Providers

`bibsync` can query NASA ADS, InspireHEP, or both. Provider selection is
controlled with `--provider`:

```shell
bibsync main.tex -o references.bib --provider auto
bibsync main.tex -o references.bib --provider ads
bibsync main.tex -o references.bib --provider inspire
```

## Auto

`auto` is the default. It tries NASA ADS first when `ADS_API_TOKEN` is present,
then falls back to InspireHEP:

```shell
bibsync main.tex -o references.bib
```

This is a practical default for astrophysics projects. ADS generally has rich
astronomy metadata, while InspireHEP is useful for high-energy physics and
gravitational-wave records that are indexed there.

If `ADS_API_TOKEN` is not set, `auto` still works by using InspireHEP.

## NASA ADS

NASA ADS supports:

- arXiv IDs
- DOIs
- ADS bibcodes

ADS requires an API token:

```shell
export ADS_API_TOKEN="..."
bibsync main.tex -o references.bib --provider ads
```

Use ADS when:

- The paper is likely indexed in the NASA Astrophysics Data System.
- You want ADS-provided journal macros and ADS metadata such as `adsurl`.
- Your citekeys include ADS bibcodes.

ADS BibTeX output can include journal macros such as `\prl`. A production paper
often gets those definitions from astronomy bibliography packages or style
files. For standalone examples, define the macro in the TeX source:

```tex
\providecommand{\prl}{Phys. Rev. Lett.}
```

## InspireHEP

InspireHEP supports:

- arXiv IDs
- DOIs

It does not require an API token for the public literature API:

```shell
bibsync main.tex -o references.bib --provider inspire
```

Use InspireHEP when:

- Your project cites high-energy physics, gravitational-wave, or related records
  indexed in InspireHEP.
- You want token-free operation in CI or pre-commit.
- Your citekeys are arXiv IDs and DOIs rather than ADS bibcodes.

## Choosing A Provider

For local author workflows, `auto` is usually the best default. It uses ADS when
available and falls back to InspireHEP.

For pre-commit hooks shared by collaborators, prefer an explicit provider:

```shell
bibsync --provider inspire --output references.bib main.tex
```

This avoids requiring every contributor to configure `ADS_API_TOKEN`. If your
project needs ADS-only bibcodes, document the token requirement in the project
README and use `--provider ads`.
