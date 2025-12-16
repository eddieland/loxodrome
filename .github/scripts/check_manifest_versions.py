#!/usr/bin/env python3

from __future__ import annotations

import os
import sys
from pathlib import Path

import tomllib


def discover_repo_root() -> Path:
    workspace = os.environ.get("GITHUB_WORKSPACE")
    if workspace:
        return Path(workspace).resolve()
    return Path(__file__).resolve().parents[2]


def main() -> int:
    release_version = os.environ.get("RELEASE_VERSION")
    if not release_version:
        sys.stderr.write("RELEASE_VERSION must be set.\n")
        return 1

    repo_root = discover_repo_root()
    manifest_cargo = repo_root / "loxodrome-rs" / "Cargo.toml"
    manifest_py = repo_root / "loxodrome" / "pyproject.toml"

    if not manifest_cargo.exists():
        sys.stderr.write(f"Missing Cargo manifest: {manifest_cargo}\n")
        return 1
    if not manifest_py.exists():
        sys.stderr.write(f"Missing Python manifest: {manifest_py}\n")
        return 1

    cargo_version = tomllib.loads(manifest_cargo.read_text())["package"]["version"]
    py_version = tomllib.loads(manifest_py.read_text())["project"]["version"]

    errors: list[str] = []
    if cargo_version != release_version:
        errors.append(f"loxodrome-rs version {cargo_version} does not match tag {release_version}")
    if py_version != release_version:
        errors.append(f"loxodrome version {py_version} does not match tag {release_version}")

    if errors:
        sys.stderr.write("\n".join(errors) + "\n")
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
