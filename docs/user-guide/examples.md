# Examples

The repository includes example TeX files and generated BibTeX fixtures under
`examples/`.

## InspireHEP Example

`examples/inspire-main.tex` cites the first LIGO/Virgo gravitational-wave
detection paper by arXiv ID and DOI:

```tex
\cite{1602.03837}
\cite{10.1103/PhysRevLett.116.061102}
```

Generate its bibliography with:

```shell
cargo run -- --fix --provider inspire --no-backup examples/inspire-main.tex
```

Because the TeX file contains:

```tex
\bibliography{inspire-main}
```

the output is discovered automatically as `examples/inspire-main.bib`.

## NASA ADS Example

`examples/main.tex` cites the same paper with three identifier styles:

```tex
\cite{1602.03837}
\cite{10.1103/PhysRevLett.116.061102}
\cite{2016PhRvL.116f1102A}
```

Generate its bibliography with:

```shell
cargo run -- --fix --provider ads --no-backup examples/main.tex
```

This writes `examples/main.bib`.

The example also defines `\prl`:

```tex
\providecommand{\prl}{Phys. Rev. Lett.}
```

That keeps the document self-contained when ADS returns `journal = {\prl}`.

## Explicit Output Files

The example commands can also write to ad hoc output files:

```shell
cargo run -- --fix --provider inspire --output examples/generated-inspire.bib --no-backup examples/inspire-main.tex
cargo run -- --fix --provider ads --output examples/generated-ads.bib --no-backup examples/main.tex
```

Those `generated-*.bib` files are ignored by Git because they are demonstration
outputs rather than canonical fixtures.

## Compile Smoke Test

The examples are covered by integration tests:

```shell
cargo test --test examples
```

The default tests verify that each checked-in bibliography contains every
citekey referenced by its TeX file.

There is also an ignored LaTeX compile smoke test:

```shell
cargo test --test examples -- --ignored
```

This requires `latexmk`, `pdflatex`, and `bibtex` to be installed locally. It is
ignored by default because the normal CI matrix does not install a TeX
distribution.
