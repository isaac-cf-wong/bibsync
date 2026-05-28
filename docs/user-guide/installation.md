# Installation

`bibsync` is distributed as a Rust command-line tool, a Python package with
bindings to the Rust implementation, and pre-built release binaries.

## Cargo

Use Cargo when you already have a Rust toolchain and want the native CLI:

```shell
cargo install bibsync
```

This installs the `bibsync` command from crates.io.

## PyPI

Use pip when you want Python bindings or prefer Python packaging:

```shell
pip install bibsync
```

The PyPI package requires Python 3.12 or newer. It provides:

- `import bibsync` for Python workflows.
- The `bibsync` console command, backed by the same Rust implementation used by
  the native CLI.

Example Python usage:

```python
import bibsync

report = bibsync.sync_files(
    ["main.tex"],
    output="references.bib",
    provider="inspire",
    check=True,
)
```

## Pre-built Binaries

Pre-built binaries are attached to each
[GitHub release](https://github.com/isaac-cf-wong/bibsync/releases).

Download the archive matching your platform:

| Platform | Architecture | Archive pattern                       |
| -------- | ------------ | ------------------------------------- |
| Linux    | x86_64       | `bibsync-vX.Y.Z-linux-x86_64.tar.gz`  |
| Linux    | aarch64      | `bibsync-vX.Y.Z-linux-aarch64.tar.gz` |
| macOS    | x86_64       | `bibsync-vX.Y.Z-macos-x86_64.tar.gz`  |
| macOS    | aarch64      | `bibsync-vX.Y.Z-macos-aarch64.tar.gz` |
| Windows  | x86_64       | `bibsync-vX.Y.Z-windows-x86_64.zip`   |

Then extract the archive and put the executable on your `PATH`.

On Linux or macOS:

```shell
tar -xzf bibsync-vX.Y.Z-linux-x86_64.tar.gz
sudo install -m 0755 bibsync /usr/local/bin/bibsync
```

On Windows, extract the `.zip` archive and place `bibsync.exe` in a directory
listed in `PATH`.

## Compile From Source

Build from source when you want the current development version or need a
platform without a pre-built binary:

```shell
git clone https://github.com/isaac-cf-wong/bibsync.git
cd bibsync
cargo build --release
```

The compiled binary is written to:

- `target/release/bibsync` on Linux and macOS.
- `target\release\bibsync.exe` on Windows.

To install that binary manually, copy it to a directory on your `PATH`.

## Development Install

For local development of the Python package:

```shell
git clone https://github.com/isaac-cf-wong/bibsync.git
cd bibsync
uv sync --group dev
uv run pytest
```

This builds the PyO3 extension in the local uv environment and runs the Python
binding tests.
