# Pre-commit

`bibsync` includes a pre-commit hook manifest, `.pre-commit-hooks.yaml`, so other
repositories can run it before commits.

Every example on this page names the citing document in `args` and sets
`pass_filenames: false`; see
[Run The Hook When Either File Changes](#run-the-hook-when-either-file-changes)
for why both are needed. The examples use a LaTeX project whose `main.tex` cites
into `references.bib`; replace those names with your own files.

By default, the hook runs `bibsync` in check mode. If new entries are needed,
the hook fails without writing changes. Add `--fix` to the hook arguments when
you want the hook to update the configured bibliography file.

## Remote Hook

Use the pre-built binary hook for faster installation:

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
matching `rev` and caches it under pre-commit's cache directory. This avoids the
long first-install compile time of Rust hooks.

Release binaries are published for Linux x86_64, Linux aarch64, macOS x86_64,
macOS aarch64, and Windows x86_64.

The source hook is still available when you prefer to build from source:

```yaml
repos:
    - repo: https://github.com/isaac-cf-wong/bibsync
      rev: v1.0.2
      hooks:
          - id: bibsync
            files: ^(main\.tex|references\.bib)$
            pass_filenames: false
            args: [--cache, --provider, inspire, --output, references.bib, main.tex]
```

## Run The Hook When Either File Changes

pre-commit passes the hook only the staged files that match it. If a commit
changes the bibliography but not the document that cites it, `bibsync` receives
only the `.bib` file; it then checks that file's own entries and never reads
which keys the document cites, so deleting a cited entry from the bibliography
would pass. Name the citing documents in `args`, set `pass_filenames: false`,
and match both the documents and the bibliography in `files:`:

```yaml
repos:
    - repo: https://github.com/isaac-cf-wong/bibsync
      rev: v1.0.2
      hooks:
          - id: bibsync-bin
            files: ^paper\.(md|bib)$
            pass_filenames: false
            args:
                [
                    --fix,
                    --cache,
                    --provider,
                    inspire,
                    --full-author-lists,
                    --full-journal-names,
                    --output,
                    paper.bib,
                    paper.md,
                ]
```

`--full-author-lists` and `--full-journal-names` make the bibliography meet
JOSS's reference rules: in-text citations read "et al." rather than
"& others", and journal names are written in full (see
[Usage](usage.md)). Both options need bibsync 2.0.0 or later.

For a LaTeX project, list the citing `.tex` files (for example `main.tex`) in
`args` instead of `paper.md`, and match them and the `.bib` file in `files:`.

For update behavior:

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

To skip specific citekeys such as manually curated books or theses, add an
`--ignore-file` argument pointing to a `.bibsyncignore` file in the repository:

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

## Local Development Hook

When developing `bibsync` itself or testing an unreleased version, use a local
hook:

```yaml
repos:
    - repo: local
      hooks:
          - id: bibsync
            name: bibsync
            entry: cargo run -- --fix --cache --provider inspire --output references.bib main.tex
            language: system
            files: ^(main\.tex|references\.bib)$
            pass_filenames: false
```

This uses the local checkout rather than installing from a tagged release.

## Provider Choice In Hooks

Prefer `--provider inspire` when possible because it does not require an API
token. This makes the hook easier for collaborators and CI systems.

Use `--provider ads` only when the project needs ADS-specific support such as
ADS bibcode citekeys. In that case, every environment running the hook must have
`ADS_API_TOKEN` set.

`--provider auto` is useful locally, but less predictable in shared hooks
because behavior changes depending on whether `ADS_API_TOKEN` is present.

## Updating After A Hook Failure

When the hook reports that the bibliography is out of date, update it with:

```shell
bibsync --fix --cache --provider inspire --output references.bib main.tex
```

Then review and commit the changed bibliography.

If the hook reports unresolved citekeys, either correct the citekey or choose a
provider that supports that identifier type. The hook output includes a reason
for each unresolved key, such as an unsupported identifier format or a provider
miss. For example, ADS bibcodes require NASA ADS:

```shell
bibsync --fix --cache --provider ads --output references.bib main.tex
```

Missing `--ignore-file` or `--other` paths and malformed existing BibTeX are
reported as input errors with the affected path. Fix those files or hook
arguments before rerunning the hook.
