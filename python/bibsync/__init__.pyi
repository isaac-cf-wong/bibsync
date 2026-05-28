from collections.abc import Sequence
from typing import TypedDict

class SyncReport(TypedDict):
    output: str
    added: list[str]
    updated: list[str]
    existing: list[str]
    found_in_other: list[str]
    unresolved: list[str]
    changed: bool
    check_mode: bool

def sync_files(
    files: list[str],
    output: str | None = None,
    other_bibliographies: list[str] | None = None,
    provider: str = "auto",
    update_mode: str = "preprints-only",
    force_regenerate: bool = False,
    merge_other: bool = False,
    backup: bool = True,
    check: bool = True,
    cache: bool = False,
    refresh_cache: bool = False,
    cache_dir: str | None = None,
    ignore_file: str | None = None,
) -> SyncReport: ...
def main(argv: Sequence[str] | None = None) -> int: ...
