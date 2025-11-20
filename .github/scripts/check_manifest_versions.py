#!/usr/bin/env python3

from __future__ import annotations

import os
import sys
from pathlib import Path

import tomllib


def main() -> int:
    release_version = os.environ.get("RELEASE_VERSION")
    if not release_version:
        sys.stderr.write("RELEASE_VERSION must be set.\n")
        return 1

    repo_root = Path(__file__).resolve().parent.parent
    cargo_version = tomllib.loads((repo_root / "geodist-rs/Cargo.toml").read_text())["package"]["version"]
    py_version = tomllib.loads((repo_root / "pygeodist/pyproject.toml").read_text())["project"]["version"]

    errors: list[str] = []
    if cargo_version != release_version:
        errors.append(f"geodist-rs version {cargo_version} does not match tag {release_version}")
    if py_version != release_version:
        errors.append(f"pygeodist version {py_version} does not match tag {release_version}")

    if errors:
        sys.stderr.write("\n".join(errors) + "\n")
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
