from __future__ import annotations

from pathlib import Path

import bibsync
import pytest

COMPLETE_ENTRY = """@article{smith2024,
  author = {Smith, Jane},
  title = {A Complete Entry},
  journal = {Journal of Local Fixtures},
  year = {2024}
}
"""


def test_sync_files_reports_existing_complete_bib_entry(tmp_path: Path) -> None:
    bibliography = tmp_path / "references.bib"
    bibliography.write_text(COMPLETE_ENTRY, encoding="utf-8")

    report = bibsync.sync_files([str(bibliography)], provider="inspire")

    assert report == {
        "output": str(bibliography),
        "added": [],
        "updated": [],
        "existing": ["smith2024"],
        "found_in_other": [],
        "unresolved": [],
        "changed": False,
        "check_mode": True,
    }
    assert bibliography.read_text(encoding="utf-8") == COMPLETE_ENTRY


def test_sync_files_accepts_underscore_update_mode_alias(tmp_path: Path) -> None:
    bibliography = tmp_path / "references.bib"
    bibliography.write_text(COMPLETE_ENTRY, encoding="utf-8")

    report = bibsync.sync_files(
        [str(bibliography)],
        provider="inspire",
        update_mode="preprints_only",
    )

    assert report["existing"] == ["smith2024"]
    assert report["changed"] is False


def test_sync_files_rejects_unknown_provider(tmp_path: Path) -> None:
    bibliography = tmp_path / "references.bib"
    bibliography.write_text(COMPLETE_ENTRY, encoding="utf-8")

    with pytest.raises(ValueError, match="provider must be"):
        bibsync.sync_files([str(bibliography)], provider="unknown")


def test_main_delegates_to_rust_cli(capfd: pytest.CaptureFixture[str]) -> None:
    exit_code = bibsync.main(["bibsync", "--version"])

    captured = capfd.readouterr()
    assert exit_code == 0
    assert captured.out.startswith("bibsync ")
