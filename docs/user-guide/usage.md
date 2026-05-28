# Usage

The main command accepts one or more TeX files, or a single BibTeX file in
bibliography-refresh mode:

```shell
bibsync [OPTIONS] [FILE]...
```

Most paper workflows use TeX input files:

```shell
bibsync main.tex -o references.bib
```

`bibsync` scans citation commands, resolves missing identifier-like citekeys, and
reports whether `references.bib` is current. It does not write changes unless
`--fix` is provided:

```shell
bibsync --fix main.tex -o references.bib
```

## Citekey Style

The most reliable workflow is to cite by identifier:

```tex
\cite{1602.03837}
\citep{10.1103/PhysRevLett.116.061102}
\citet{2016PhRvL.116f1102A}
```

When the provider returns a BibTeX entry, `bibsync` rewrites the entry key to
match the citekey used in TeX. For example, if InspireHEP returns
`@article{Abbott:2016blz, ...}` for `\cite{1602.03837}`, the output entry is
written as `@article{1602.03837, ...}`. This keeps the TeX source stable.

## Output Discovery

If the TeX source contains a bibliography command, `bibsync` can discover the
output file:

```tex
\bibliography{references}
```

Then this command checks `references.bib`:

```shell
bibsync main.tex
```

To update the discovered file, add `--fix`:

```shell
bibsync --fix main.tex
```

If more than one bibliography file is declared, the first one becomes the output
file and the remaining files are treated as additional read-only sources.

Use `--output` when you want explicit control:

```shell
bibsync main.tex chapter.tex --output references.bib
```

## Existing Bibliographies

By default, `bibsync` leaves existing entries untouched unless they look like
unpublished preprints. Preprint entries — those with an `archivePrefix` or
`eprinttype` field but no `journal` field — are re-queried against the provider
to detect whether they have since been published. If the provider returns a
journal record, the entry is updated; if the paper is still a preprint, the
existing entry is preserved as-is.

Use `--no-update` to skip all existing entries unconditionally:

```shell
bibsync --fix main.tex -o references.bib --no-update
```

Use `--update-all` to re-resolve every existing entry from the provider,
equivalent to the old default behavior:

```shell
bibsync --fix main.tex -o references.bib --update-all
```

Use `--force-regenerate` to re-resolve all existing entries and overwrite them
with fresh provider output regardless of whether they appear to be preprints:

```shell
bibsync --fix references.bib --force-regenerate
```

Additional read-only bibliographies can be passed with `--other`:

```shell
bibsync main.tex -o references.bib --other shared.bib software.bib
```

By default, entries found in those read-only files are not copied into the main
output file. This is useful when a project deliberately keeps shared references
or software citations in separate files.

Use `--merge-other` to copy matching entries into the output file:

```shell
bibsync --fix main.tex -o references.bib --other shared.bib --merge-other
```

## Ignoring Specific Entries

To prevent `bibsync` from touching certain entries — for example, manually
curated book or thesis records — list their citekeys in an ignore file:

```text
# .bibsyncignore
knuth1997art
smith2024thesis
```

Each line is one citekey. Lines starting with `#` are treated as comments.
Pass the file with `--ignore-file`:

```shell
bibsync --fix main.tex -o references.bib --ignore-file .bibsyncignore
```

Ignored citekeys are never sent to any provider and their existing bib entries
are never modified.

## Updating A BibTeX File

Passing a single `.bib` file uses the existing keys in that bibliography as the
identifiers to resolve:

```shell
bibsync references.bib
```

This is useful for checking whether provider metadata has changed without
scanning TeX files. Add `--fix` to refresh the file:

```shell
bibsync --fix references.bib
```

Use `--force-regenerate` to rewrite all existing entries from provider output:

```shell
bibsync --fix references.bib --force-regenerate
```

## Check And Fix

Check mode is the default. It performs resolution and merge calculation but does
not write the output file:

```shell
bibsync main.tex -o references.bib
```

The command exits with a non-zero status when the bibliography would change or
when a required citekey cannot be resolved. This is the mode used by the
pre-commit hook.

Use `--fix` to write the calculated bibliography:

```shell
bibsync --fix main.tex -o references.bib
```

## Cache

Provider requests are network-bound. Use `--cache` to store provider records and
reuse them on later runs:

```shell
bibsync --cache main.tex -o references.bib
bibsync --fix --cache main.tex -o references.bib
```

Cache entries are keyed by provider and canonical record ID. Mappings from arXiv
IDs, DOIs, and ADS bibcodes are stored separately, so a citekey such as
`1602.03837` can be resolved from cache after its provider record is known.

Preprint entries that are re-checked for publication always bypass the cache and
fetch a fresh result from the provider. This ensures `bibsync` sees the latest
publication status regardless of what is stored locally. The fresh result is
written back to the cache afterwards.

Use `--refresh-cache` to force provider calls and update cached records for all
entries:

```shell
bibsync --fix --refresh-cache main.tex -o references.bib
```

Override the cache location with `--cache-dir`:

```shell
bibsync --cache --cache-dir .bibsync-cache main.tex -o references.bib
```

## Backups

When `bibsync` writes over an existing bibliography, it creates a `.bak` file by
default. Disable this with:

```shell
bibsync --fix main.tex -o references.bib --no-backup
```

For automation, `--no-backup` is often appropriate because the repository
history already records the previous bibliography state.
