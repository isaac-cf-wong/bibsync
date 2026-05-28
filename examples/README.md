# bibsync examples

The example TeX files use the first LIGO/Virgo gravitational-wave detection
paper because it is resolvable by both NASA ADS and InspireHEP.

## InspireHEP

Generate from arXiv and DOI citekeys with an explicit output file:

```shell
cargo run -- --fix --provider inspire --output examples/generated-inspire.bib --no-backup examples/inspire-main.tex
```

Generate by discovering the output file from `\bibliography{inspire-main}`:

```shell
cargo run -- --fix --provider inspire --no-backup examples/inspire-main.tex
```

Check mode for pre-commit usage:

```shell
cargo run -- --provider inspire --output examples/generated-inspire.bib examples/inspire-main.tex
```

Update an existing BibTeX file in place:

```shell
cargo run -- --fix --provider inspire --force-regenerate --no-backup examples/generated-inspire.bib
```

## NASA ADS

NASA ADS needs `ADS_API_TOKEN` in the environment. It can resolve arXiv IDs,
DOIs, and ADS bibcodes:

```shell
cargo run -- --fix --provider ads --output examples/generated-ads.bib --no-backup examples/main.tex
```

## Auto Provider

The default `auto` provider tries NASA ADS when `ADS_API_TOKEN` is present, then
falls back to InspireHEP:

```shell
cargo run -- --fix --provider auto --output examples/generated-auto.bib --no-backup examples/inspire-main.tex
```
