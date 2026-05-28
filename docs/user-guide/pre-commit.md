# Pre-commit

`bibsync` includes a pre-commit hook manifest, `.pre-commit-hooks.yaml`, so other
repositories can run it before commits.

The hook runs `bibsync --check`. It verifies that the bibliography is already
synchronized and exits without writing files. When the hook fails, run the same
command without `--check`, commit the updated `.bib`, and try again.

## Remote Hook

Use the pre-built binary hook for faster installation:

```yaml
repos:
    - repo: https://github.com/isaac-cf-wong/bibsync
      rev: v0.1.0
      hooks:
          - id: bibsync-bin
            args: [--provider, inspire, --output, references.bib]
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
    rev: v0.1.0
      hooks:
          - id: bibsync
            args: [--provider, inspire, --output, references.bib]
```

This expands to:

```shell
scripts/pre-commit-bibsync --provider inspire --output references.bib <changed files>
```

which downloads `bibsync` if needed and then runs:

```shell
bibsync --check --provider inspire --output references.bib <changed files>
```

The hook receives changed file paths from pre-commit. In most projects, restrict
the hook to TeX sources so it runs only when citations may have changed.

## Local Development Hook

When developing `bibsync` itself or testing an unreleased version, use a local
hook:

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

When the hook reports that the bibliography is out of date, run:

```shell
bibsync --provider inspire --output references.bib main.tex
```

Then review and commit the changed bibliography.

If the hook reports unresolved citekeys, either correct the citekey or choose a
provider that supports that identifier type. For example, ADS bibcodes require
NASA ADS:

```shell
bibsync --provider ads --output references.bib main.tex
```
