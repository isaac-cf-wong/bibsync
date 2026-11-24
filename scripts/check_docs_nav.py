"""Fail when zensical.toml's nav names a page that does not exist under docs/.

zensical builds the site without a warning when a nav entry points at a
missing file, so `zensical build --strict` alone does not catch it.
"""

import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def nav_pages(entries: list) -> list[str]:
    """Every page path in a zensical nav list, depth first."""
    pages = []
    for entry in entries:
        if isinstance(entry, str):
            pages.append(entry)
        elif isinstance(entry, dict):
            for value in entry.values():
                pages.extend(nav_pages(value) if isinstance(value, list) else [value])
    return pages


def main() -> int:
    config = tomllib.loads((ROOT / "zensical.toml").read_text(encoding="utf-8"))
    docs = ROOT / config["project"].get("docs_dir", "docs")
    missing = [
        page
        for page in nav_pages(config["project"].get("nav", []))
        if "://" not in page and not (docs / page).is_file()
    ]
    for page in missing:
        print(f"zensical.toml nav: {docs.name}/{page} does not exist", file=sys.stderr)
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())
