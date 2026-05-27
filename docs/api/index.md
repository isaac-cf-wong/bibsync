# API Overview

Most users interact with `bibsync` through the command-line interface. The Rust
library is still useful for tests, custom automation, and tools that want to
provide their own provider or cache layer.

## Synchronization Entry Points

Use `sync_files` for normal provider-backed synchronization:

```rust
use std::path::PathBuf;

use bibsync::{ProviderChoice, SyncOptions, sync_files};

let files = vec![PathBuf::from("main.tex")];
let options = SyncOptions {
    output: Some(PathBuf::from("references.bib")),
    provider: ProviderChoice::Inspire,
    ..SyncOptions::default()
};

let report = sync_files(&files, &options)?;
println!("added {} entries", report.added.len());
# Ok::<(), bibsync::BibsyncError>(())
```

Use `sync_files_with_provider` when you want to inject your own provider:

```rust
use bibsync::{BibliographyProvider, ResolvedEntry};

struct CachedProvider;

impl BibliographyProvider for CachedProvider {
    fn name(&self) -> &'static str {
        "cached"
    }

    fn resolve(&self, key: &str) -> bibsync::Result<Option<ResolvedEntry>> {
        let _ = key;
        Ok(None)
    }
}
```

This is how the test suite avoids network access for core synchronization
behavior.

## Main Types

`SyncOptions` controls a run:

- `output`: explicit output bibliography file.
- `other_bibliographies`: read-only bibliography sources.
- `provider`: `auto`, NASA ADS, or InspireHEP.
- `update_existing`: refresh existing entries when possible.
- `force_regenerate`: rewrite existing entries from provider output.
- `merge_other`: copy matching read-only entries into the output file.
- `backup`: create a `.bak` file before overwriting.
- `check`: compare only and do not write.

`SyncReport` describes the outcome:

- `added`: citekeys added to the bibliography.
- `updated`: citekeys regenerated from provider output.
- `existing`: citekeys already present.
- `found_in_other`: citekeys found in read-only bibliographies.
- `unresolved`: citekeys that could not be resolved.
- `changed`: whether the output would change.
- `check_mode`: whether the run was check-only.

## Providers

The built-in providers are:

- `AdsProvider`, configured from `ADS_API_TOKEN`
- `InspireProvider`, configured without authentication

The `BibliographyProvider` trait has one required operation:

```rust
fn resolve(&self, key: &str) -> bibsync::Result<Option<ResolvedEntry>>;
```

Returning `Ok(None)` means the provider does not have a match. Returning an
error means the request or response handling failed.

## Citation Scanning

Use `citation_keys` when you only need to inspect a TeX source:

```rust
use std::path::PathBuf;

let keys = bibsync::citation_keys(&[PathBuf::from("main.tex")])?;
# Ok::<(), bibsync::BibsyncError>(())
```

This returns a sorted set of citekeys discovered in supported citation commands.

## Generated Rustdoc

The canonical API reference is generated with:

```shell
cargo doc --no-deps --all-features
```

When the crate is released, docs.rs provides the public Rust API documentation.
