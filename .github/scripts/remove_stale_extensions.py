"""Clean up compiled extension artifacts from the Python package tree.

Intended for GitHub Actions so stale binaries do not get folded into wheels.
"""

from __future__ import annotations

import pathlib


def main() -> int:
    repo_root = pathlib.Path(__file__).resolve().parents[2]
    target_dir = repo_root / "pygeodist" / "src" / "geodist"

    if not target_dir.is_dir():
        print(f"Target directory not found: {target_dir}")
        return 0

    removed: list[str] = []
    for path in target_dir.glob("_geodist_rs.*"):
        if path.suffix.lower() in {".so", ".pyd", ".dylib", ".dll"}:
            path.unlink()
            removed.append(path.name)

    if removed:
        print("Removed stale artifacts:", ", ".join(sorted(removed)))
    else:
        print("No stale compiled extensions found.")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
