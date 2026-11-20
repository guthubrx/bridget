#!/usr/bin/env python3
"""Check local Markdown links and language pairs in the public documentation."""
from pathlib import Path
from urllib.parse import unquote, urlsplit
import ipaddress
import json
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "docs/public-pages.json"
IPV4_RE = re.compile(r"(?<![0-9.])((?:25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(?:\.(?:25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3})(?![0-9.])")
DOC_NETWORKS = (
    ipaddress.ip_network("192.0.2.0/24"),
    ipaddress.ip_network("198.51.100.0/24"),
    ipaddress.ip_network("203.0.113.0/24"),
    ipaddress.ip_network("100.64.0.0/10"),
)
VERSION_CONTEXT_RE = re.compile(
    r"(?:version|v|v8|glibc|go|android|caddy|compose|netty|guava|protobuf|rust|release|spec|commit|build|tag|jdk|sdk)[^0-9\n]{0,16}$",
    re.IGNORECASE,
)


def is_forbidden_public_ipv4(candidate: str, line_prefix: str) -> bool:
    if VERSION_CONTEXT_RE.search(line_prefix):
        return False
    addr = ipaddress.ip_address(candidate)
    if (
        addr.is_private
        or addr.is_loopback
        or addr.is_link_local
        or addr.is_multicast
        or addr.is_reserved
        or addr.is_unspecified
        or any(addr in net for net in DOC_NETWORKS)
    ):
        return False
    # Ignore 0.x.x.x or software version quads like 1.2.3.4 / 2.11.7.0 unless preceded by host/URL indicators
    if candidate.startswith(("0.", "1.", "2.", "3.")):
        if not re.search(r"(?:@|://|ssh|host|ip|server|serveur)\s*$", line_prefix, re.IGNORECASE):
            return False
    return True


def check_public_ips() -> list[str]:
    errors = []
    tracked = subprocess.run(
        ["git", "ls-files", "*.md", "docs/", "specs/", "README.md", "SECURITY.md"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if tracked.returncode != 0:
        return []
    files = sorted({line.strip() for line in tracked.stdout.splitlines() if line.strip().endswith(".md")})
    for rel in files:
        path = ROOT / rel
        if not path.is_file():
            continue
        for lineno, line in enumerate(path.read_text(errors="ignore").splitlines(), start=1):
            for match in IPV4_RE.finditer(line):
                ip_str = match.group(1)
                prefix = line[: match.start()]
                if is_forbidden_public_ipv4(ip_str, prefix):
                    errors.append(f"{rel}:{lineno}: public IPv4 outside RFC 5737 documentation ranges: {ip_str}")
    return errors


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
    errors.extend(check_public_ips())
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"PASS: {len(pages)} pages; {checked} local links; {len(manifest['pairs'])} language pairs; 0 public IPv4")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
