#!/usr/bin/env python3
"""Check local Markdown links and language pairs in the public documentation."""
from pathlib import Path
from urllib.parse import unquote, urlsplit
import json
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "docs/public-pages.json"


def prose(text):
    return re.sub(r"(?ms)^(`{3,}|~{3,})[^\n]*\n.*?^\1[^\n]*$", "", text)


def anchors(path):
    text = prose(path.read_text())
    counts = {}
    result = set()
    for title in re.findall(r"(?m)^#{1,6}\s+(.+?)(?:\s+#+)?$", text):
        slug = re.sub(r"[^\w\- ]", "", title.replace("`", "").lower()).replace(" ", "-")
        n = counts.get(slug, 0)
        counts[slug] = n + 1
        result.add(slug if n == 0 else f"{slug}-{n}")
    result.update(re.findall(r'(?:id|name)=["\x27]([^"\x27]+)["\x27]', text))
    return result


def main():
    manifest = json.loads(MANIFEST.read_text())
    errors = []
    pages = set(manifest["additional"])
    for pair in manifest["pairs"]:
        pages.update(pair.values())
        for language, name in pair.items():
            if not (ROOT / name).is_file():
                errors.append(f"{language} page missing: {name}")
    checked = 0
    for name in sorted(pages):
        page = ROOT / name
        if not page.is_file():
            errors.append(f"Page missing: {name}")
            continue
        text = prose(page.read_text())
        links = re.findall(r'\]\(([^\s)]+)(?:\s+["\x27].*?["\x27])?\)', text)
        links += re.findall(r'<(?:img|a)\b[^>]*(?:src|href)=["\x27]([^"\x27]+)', text)
        for raw in links:
            target = urlsplit(raw.strip("<>"))
            if target.scheme or target.netloc:
                continue
            dest = (page.parent / unquote(target.path)).resolve() if target.path else page
            if not dest.is_relative_to(ROOT):
                errors.append(f"{name}: link escapes repository: {raw}")
            elif not dest.exists():
                errors.append(f"{name}: missing target: {raw}")
            elif target.fragment and dest.suffix == ".md":
                if unquote(target.fragment) not in anchors(dest):
                    errors.append(f"{name}: missing anchor: {raw}")
            checked += 1
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"PASS: {len(pages)} pages; {checked} local links; {len(manifest['pairs'])} language pairs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
