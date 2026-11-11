#!/usr/bin/env python3
"""Déplace des fonctions de premier niveau d'un fichier Kotlin vers un nouveau fichier (programme 260, lot 16).

Usage :
    python3 -I deplacer_kotlin.py SOURCE.kt NOUVEAU.kt Nom1 Nom2 ...

Chaque `Nom` est une fonction de colonne 0 (`private fun Nom(` ou `fun Nom(`, avec ses annotations
`@...` et commentaires juste au-dessus). Elle est déplacée sans modification de son corps ;
seul `private` devient `internal`. Le nouveau fichier reçoit `package`, les imports de SOURCE
effectivement utilisés par les fonctions déplacées. SOURCE perd les imports qui ne servent plus.
Refus (code 2) : fonction introuvable ou non unique, fichier cible existant.
"""
import re
import sys
from pathlib import Path


class Refus(Exception):
    pass


def block_end(lines, i):
    """Fin (indice exclu) de la déclaration qui commence à lines[i] (colonne 0)."""
    depth = 0
    j = i
    seen = False
    in_str = False
    while j < len(lines):
        line = lines[j]
        k = 0
        while k < len(line):
            c = line[k]
            if c == '"' and not line.startswith('"""', k):
                k += 1
                while k < len(line) and line[k] != '"':
                    k += 2 if line[k] == "\\" else 1
            elif line.startswith("//", k):
                break
            elif c in "{(":
                depth += 1
                seen = True
            elif c in "})":
                depth -= 1
            k += 1
        j += 1
        if seen and depth <= 0:
            return j
        if not seen and j > i and not line.rstrip().endswith(("=", "(", ",", "{")) and line.strip() and not lines[j].startswith((" ", "\t")) :
            return j  # expression simple sur une ligne
    return j


def main(argv):
    if len(argv) < 4:
        print(__doc__, file=sys.stderr)
        return 1
    src, dst, names = Path(argv[1]), Path(argv[2]), argv[3:]
    try:
        if dst.exists():
            raise Refus(f"{dst} existe déjà")
        lines = src.read_text(encoding="utf-8").split("\n")
        spans = []
        for n in names:
            idx = [i for i, l in enumerate(lines) if re.match(r"^(?:private |internal |public )?(?:inline |suspend )?fun (?:<[^>]*> )?(?:[\w.]+\.)?%s\(" % re.escape(n), l)]
            if len(idx) != 1:
                raise Refus(f"{len(idx)} déclarations pour {n}")
            i = idx[0]
            start = i
            while start > 0 and (lines[start - 1].startswith("@") or lines[start - 1].startswith("//") or lines[start - 1].startswith("/**") or lines[start - 1].startswith(" *")):
                start -= 1
            spans.append((start, block_end(lines, i)))
        spans.sort()
        moved = []
        for s, e in spans:
            moved.append("\n".join(lines[s:e]).rstrip("\n"))
        keep = [l for k, l in enumerate(lines) if not any(s <= k < e for s, e in spans)]
        # collapse triple blank lines
        text = re.sub(r"\n{3,}", "\n\n", "\n".join(keep))
        body = "\n\n".join(moved)
        body = re.sub(r"(?m)^private fun ", "internal fun ", body)
        pkg = next(l for l in lines if l.startswith("package "))
        imports = [l for l in lines if l.startswith("import ")]
        def used(imp, code):
            name = imp.split(" as ")[-1].split(".")[-1].strip()
            return name == "*" or re.search(r"\b%s\b" % re.escape(name), code) is not None
        new_imports = [i for i in imports if used(i, body)]
        # SOURCE : retirer les imports devenus inutiles (hors étoile)
        rest_code = "\n".join(l for l in text.split("\n") if not l.startswith("import "))
        text = "\n".join(l for l in text.split("\n") if not (l.startswith("import ") and not used(l, rest_code)))
        src.write_text(text, encoding="utf-8")
        dst.write_text(pkg + "\n\n" + "\n".join(new_imports) + "\n\n" + body + "\n", encoding="utf-8")
        print(f"{len(names)} fonctions -> {dst.name} ({sum(e - s for s, e in spans)} lignes)")
    except Refus as err:
        print(f"REFUS : {err}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
