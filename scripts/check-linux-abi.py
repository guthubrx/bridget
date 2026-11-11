#!/usr/bin/env python3
"""Qualifier le binaire Linux commun avant distribution (Debian 12, glibc 2.36)."""
import argparse
import json
import re
import subprocess
from pathlib import Path


def qualify_versions(text):
    versions = set(re.findall(r"\bName: GLIBC_([0-9]+(?:\.[0-9]+)+)\b", text))
    if not versions:
        raise ValueError("linux_abi_unattested")
    maximum = max(versions, key=lambda v: tuple(map(int, v.split("."))))
    components = tuple(map(int, maximum.split(".")))
    if components + (0,) * max(0, 3 - len(components)) > (2, 36, 0):
        raise ValueError("linux_abi_too_recent:required=" + maximum + ":maximum=2.36")
    return {"status": "compatible", "required_glibc": maximum, "maximum_glibc": "2.36"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    args = parser.parse_args()
    try:
        with args.binary.open("rb") as stream:
            if stream.read(4) != b"\x7fELF":
                raise ValueError("linux_abi_not_elf")
        result = subprocess.run(
            ["readelf", "--version-info", "--wide", str(args.binary)],
            capture_output=True, text=True, check=True, timeout=30)
        print(json.dumps(qualify_versions(result.stdout)))
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        parser.exit(1, "Refus de distribution Linux : " + str(error) +
                    ". Reconstruire sur la cible commune Debian 12, puis qualifier ce binaire.\n")


if __name__ == "__main__":
    main()
