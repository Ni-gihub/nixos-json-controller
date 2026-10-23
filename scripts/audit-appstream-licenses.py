#!/usr/bin/env python3
"""Summarize AppStream metadata_license identifiers in a local data output.

Usage:
  python3 scripts/audit-appstream-licenses.py /nix/store/...-nixos-appstream-data-all

The script only reports identifiers from the generated XML. It does not decide
whether redistribution is permitted or replace review of individual records.
"""
from __future__ import annotations

import gzip
import sys
import xml.etree.ElementTree as ET
from collections import Counter
from pathlib import Path


def local_name(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {Path(sys.argv[0]).name} APPSTREAM_DATA_DIR", file=sys.stderr)
        return 2

    root = Path(sys.argv[1]).expanduser().resolve()
    xml_root = root / "share" / "swcatalog" / "xml"
    files = sorted(xml_root.glob("*.xml.gz"))
    if not files:
        print(f"no AppStream XML files found under {xml_root}", file=sys.stderr)
        return 1

    license_counts: Counter[str] = Counter()
    missing_count = 0
    record_count = 0
    errors: list[str] = []

    for xml_path in files:
        try:
            with gzip.open(xml_path, "rb") as stream:
                for _, element in ET.iterparse(stream, events=("end",)):
                    if local_name(element.tag) != "component":
                        continue

                    record_count += 1
                    values = [
                        (child.text or "").strip()
                        for child in element
                        if local_name(child.tag) == "metadata_license"
                    ]
                    value = values[0] if values else ""
                    if value:
                        license_counts[value] += 1
                    else:
                        missing_count += 1
                    element.clear()
        except (OSError, ET.ParseError) as error:
            errors.append(f"{xml_path.name}: {error}")

    print(f"XML files: {len(files)}")
    print(f"Components: {record_count}")
    print(f"Components without metadata_license: {missing_count}")
    print("metadata_license counts:")
    for license_id, count in sorted(license_counts.items()):
        print(f"  {license_id}: {count}")

    if errors:
        print("Errors while reading files:", file=sys.stderr)
        for error in errors:
            print(f"  {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
