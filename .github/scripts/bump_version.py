"""Update loxodrome manifest versions to a provided MAJOR.MINOR.PATCH string."""

from __future__ import annotations

import re
import sys
from pathlib import Path


def main(version: str) -> int:
    """Update version strings in manifest files.

    This will update:
    1. `loxodrome-rs/Cargo.toml`
    2. `loxodrome/pyproject.toml`

    It sets the `version` field in both files to the provided version.
    """
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        sys.stderr.write(f"Invalid version '{version}'; expected MAJOR.MINOR.PATCH.\n")
        return 1

    targets = [
        ("loxodrome-rs/Cargo.toml", r'(?m)^version = "(.*?)"'),
        ("loxodrome/pyproject.toml", r'(?m)^version = "(.*?)"'),
    ]

    for path, pattern in targets:
        text = Path(path).read_text()
        new_text, count = re.subn(pattern, f'version = "{version}"', text, count=1)
        if count != 1:
            sys.stderr.write(f"Could not update version in {path}; pattern not found.\n")
            return 1
        Path(path).write_text(new_text)

    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.stderr.write("Usage: python .github/scripts/bump_version.py MAJOR.MINOR.PATCH\n")
        sys.exit(1)
    sys.exit(main(sys.argv[1]))
