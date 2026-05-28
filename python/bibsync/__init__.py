"""Python bindings for bibsync."""

from __future__ import annotations

import sys
from collections.abc import Sequence

from ._bibsync import run_cli as _run_cli
from ._bibsync import sync_files

__all__ = ["main", "sync_files"]


def main(argv: Sequence[str] | None = None) -> int:
    """Run the Rust CLI implementation."""
    if argv is None:
        argv = sys.argv
    return _run_cli(list(argv))
