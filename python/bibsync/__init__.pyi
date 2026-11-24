from collections.abc import Sequence
from typing import TypedDict

class AbbreviatedVenue(TypedDict):
    key: str
    # Empty for an unreadable entry.
    field: str
    # Joined text of the value; a macro as written; empty for an unreadable entry.
    value: str
    # "known_abbreviation", "looks_abbreviated", "macro" or "unreadable_entry".
    problem: str
    # The journal table's full name for a known abbreviation, otherwise empty.
    full_name: str

class SyncReport(TypedDict):
    output: str
    added: list[str]
    updated: list[str]
    existing: list[str]
    found_in_other: list[str]
    unresolved: list[str]
    unresolved_details: list[dict[str, str]]
    unexpanded_journals: list[dict[str, str]]
    truncated_author_lists: list[str]
    abbreviated_venues: list[AbbreviatedVenue]
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
    full_journal_names: bool = False,
    full_author_lists: bool = False,
    check_venue_names: bool = False,
    accepted_venues: str | None = None,
) -> SyncReport: ...
def main(argv: Sequence[str] | None = None) -> int: ...
