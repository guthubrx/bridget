#!/usr/bin/env python3
"""Liste les fonctions Rust longues ou à nombreux paramètres (programme 260, lot 9).

Usage :
    python3 -I longues_fonctions.py DOSSIER [--min-lignes 300] [--min-params 0]

Sortie, une ligne par fonction, triée par longueur décroissante :
    <lignes> <paramètres> <fichier>:<ligne> <nom>

Méthode simple, sans analyseur Rust : une fonction commence à une ligne `fn nom(`
et se termine à la première ligne qui n'a que `}` à la même indentation.
Elle exclut les fichiers de tests (`*_test.rs`, `*_tests.rs`, `tests.rs`, dossiers `tests/`)
et les fonctions situées dans un module `#[cfg(test)] mod x {` de colonne 0.
Les chiffres sont une mesure de contrôle : ils peuvent différer de 1 ou 2
fonctions d'un relevé fait avec un autre outil.
"""
import re
import sys
from pathlib import Path

FN_RE = re.compile(r"^(?P<ind>\s*)(?:pub(?:\([a-z: ]+\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+(?P<name>\w+)")


def count_params(sig):
    start = sig.find("(")
    if start < 0:
        return 0
    depth = 0
    commas = 0
    seen = False
    prev = ""
    for ch in sig[start:]:
        if ch == ">" and prev == "-":
            prev = ch
            continue
        prev = ch
        if ch in "([{<":
            depth += 1
        elif ch in ")]}>":
            depth -= 1
            if depth == 0:
                break
        elif ch == "," and depth == 1:
            commas += 1
        elif depth >= 1 and not ch.isspace():
            seen = True
    return commas + 1 if seen else 0


def scan(path):
    lines = path.read_text(encoding="utf-8", errors="replace").split("\n")
    out = []
    limit = len(lines)
    skip = set()
    for i, l in enumerate(lines):
        if l.startswith("#[cfg(test)]") and i + 1 < len(lines) and re.match(r"^(pub(\([a-z: ]+\))?\s+)?mod\s+\w+\s*\{", lines[i + 1]):
            for k in range(i + 1, len(lines)):
                skip.add(k)
                if lines[k].rstrip() == "}":
                    break
    for i in range(limit):
        m = FN_RE.match(lines[i])
        if not m or i in skip:
            continue
        ind = m.group("ind")
        sig = []
        j = i
        while j < limit:
            sig.append(lines[j])
            if "{" in lines[j] or lines[j].rstrip().endswith(";"):
                break
            j += 1
        if sig and sig[-1].rstrip().endswith(";") and "{" not in sig[-1]:
            continue
        end = None
        for k in range(j, limit):
            if lines[k].rstrip() == ind + "}":
                end = k
                break
        if end is None:
            continue
        out.append((end - i + 1, count_params(" ".join(sig)), f"{path}:{i + 1}", m.group("name")))
    return out


def main(argv):
    args = argv[1:]
    min_lines, min_params = 300, 0
    if "--min-lignes" in args:
        k = args.index("--min-lignes")
        min_lines = int(args[k + 1])
        del args[k : k + 2]
    if "--min-params" in args:
        k = args.index("--min-params")
        min_params = int(args[k + 1])
        del args[k : k + 2]
        min_lines = min_lines if "--min-lignes" in argv else 10**9
    if len(args) != 1:
        print(__doc__, file=sys.stderr)
        return 1
    rows = []
    for p in sorted(Path(args[0]).rglob("*.rs")):
        s = str(p)
        if "/tests/" in s or re.search(r"(_tests?|/tests)\.rs$", s):
            continue
        rows += scan(p)
    for n, params, where, name in sorted(rows, reverse=True):
        if n >= min_lines or (min_params and params >= min_params):
            print(f"{n} {params} {where} {name}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
