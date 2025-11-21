"""Determine the release version for CI workflows.

Behaviors:
- pull_request: read the Python package version from pygeodist/pyproject.toml
- tag push / manual run on tag: validate tag format vMAJOR.MINOR.PATCH and emit version
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

import tomllib


def repo_root() -> Path:
    workspace = os.environ.get("GITHUB_WORKSPACE")
    if workspace:
        return Path(workspace).resolve()
    return Path(__file__).resolve().parents[2]


def version_from_manifest(root: Path) -> str:
    manifest = root / "pygeodist" / "pyproject.toml"
    data = tomllib.loads(manifest.read_text())
    return data["project"]["version"]


def version_from_tag(tag: str) -> str:
    if not re.fullmatch(r"v\d+\.\d+\.\d+", tag):
        raise ValueError(f"Release tags must look like vMAJOR.MINOR.PATCH (got {tag}).")
    return tag[1:]


def main() -> int:
    event = os.environ.get("GITHUB_EVENT_NAME", "")
    ref_type = os.environ.get("GITHUB_REF_TYPE", "")
    ref_name = os.environ.get("GITHUB_REF_NAME", "")

    try:
        if event == "pull_request":
            version = version_from_manifest(repo_root())
        else:
            if ref_type != "tag":
                raise ValueError(f"This workflow only runs from tags (got {ref_type}).")
            version = version_from_tag(ref_name)
    except Exception as exc:  # noqa: BLE001
        sys.stderr.write(f"{exc}\n")
        return 1

    sys.stdout.write(f"version={version}\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
