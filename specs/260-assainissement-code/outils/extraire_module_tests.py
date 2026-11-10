#!/usr/bin/env python3
"""Sort un module de tests en ligne vers un fichier voisin (programme 260, lot 4).

Usage :
    python3 -I extraire_module_tests.py --list FICHIER.rs
    python3 -I extraire_module_tests.py --extract FICHIER.rs LIGNE

--list     montre les modules `#[cfg(test)]` de premier niveau et leur taille.
--extract  déplace le module dont la ligne `mod X {` est LIGNE (numéro à partir de 1).

Règles (déterministes, sans deviner) :
- seul un module de colonne 0, précédé de `#[cfg(test)]`, de plus de 200 lignes ;
- la fin du module est la première ligne `}` en colonne 0 après son début ;
- le nouveau fichier est créé dans le même dossier. Nom : `<stem>_tests.rs` si le
  module s'appelle `tests`, `<mod>.rs` si son nom commence par le stem, sinon
  `<stem>_<mod>.rs` ;
- le corps n'est pas désindenté (le lot 10 s'en charge avec rustfmt) ;
- refus si le fichier cible existe, si le corps contient `mod y;`, si une
  ligne d'attribut est sur plusieurs lignes, ou si la reconstruction du texte
  d'origine ne redonne pas le fichier exact.
Code de retour : 0 réussi, 2 refus (message sur stderr), 1 erreur d'usage.
"""
import re
import sys
from pathlib import Path

MIN_LINES = 200
MOD_RE = re.compile(r"^(?P<vis>pub(?:\([a-z: ]+\))?\s+)?mod\s+(?P<name>\w+)\s*\{\s*$")
MOD_DECL_RE = re.compile(r"^\s*(?:pub(?:\([a-z: ]+\))?\s+)?mod\s+\w+\s*;")


class Refus(Exception):
    pass


def attrs_above(lines, idx):
    """Renvoie l'indice de la première ligne d'attributs collée au-dessus de idx."""
    start = idx
    while start > 0 and lines[start - 1].startswith("#["):
        if not lines[start - 1].rstrip().endswith("]"):
            raise Refus(f"attribut sur plusieurs lignes avant la ligne {idx + 1}")
        start -= 1
    return start


def find_end(lines, idx):
    for j in range(idx + 1, len(lines)):
        if lines[j].rstrip() == "}":
            return j
    raise Refus(f"aucune ligne '}}' en colonne 0 après la ligne {idx + 1}")


def candidates(lines):
    out = []
    for i, line in enumerate(lines):
        m = MOD_RE.match(line)
        if not m:
            continue
        a = attrs_above(lines, i)
        if not any(x.strip() == "#[cfg(test)]" for x in lines[a:i]):
            continue
        end = find_end(lines, i)
        out.append((i, a, end, m.group("name"), (m.group("vis") or "")))
    return out


def target_name(stem, mod):
    if mod == "tests":
        return f"{stem}_tests.rs"
    if mod.startswith(stem):
        return f"{mod}.rs"
    return f"{stem}_{mod}.rs"


def cmd_list(path):
    lines = path.read_text(encoding="utf-8").split("\n")
    for i, a, end, name, _vis in candidates(lines):
        size = end - i + 1
        verdict = "a sortir" if size > MIN_LINES else "reste (petit)"
        print(f"{path}:{i + 1}: mod {name} - {size} lignes - {verdict}")


def cmd_extract(path, line_no):
    text = path.read_text(encoding="utf-8")
    lines = text.split("\n")
    found = [c for c in candidates(lines) if c[0] == line_no - 1]
    if not found:
        raise Refus(f"la ligne {line_no} n'est pas un module de tests de colonne 0")
    i, a, end, name, vis = found[0]
    size = end - i + 1
    if size <= MIN_LINES:
        raise Refus(f"module {name} trop petit ({size} lignes, minimum {MIN_LINES + 1})")
    body = lines[i + 1 : end]
    for b in body:
        if MOD_DECL_RE.match(b):
            raise Refus(f"le corps contient une déclaration de module fichier : {b.strip()}")
    stem = path.stem
    if stem == "mod":
        raise Refus("fichier mod.rs non pris en charge")
    new_name = target_name(stem, name)
    new_path = path.with_name(new_name)
    if new_path.exists():
        raise Refus(f"le fichier cible existe déjà : {new_path}")
    attrs = lines[a:i]
    decl = attrs + [f'#[path = "{new_name}"]', f"{vis}mod {name};"]
    new_lines = lines[:a] + decl + lines[end + 1 :]
    new_body = "\n".join(body) + "\n"
    # Reconstruction : remettre le corps en ligne doit redonner le fichier d'origine.
    rebuilt = new_lines[: a + len(attrs)] + lines[i : end + 1] + new_lines[a + len(decl) :]
    if "\n".join(rebuilt) != text:
        raise Refus("la reconstruction du texte d'origine diffère : rien n'est écrit")
    new_path.write_text(new_body, encoding="utf-8")
    path.write_text("\n".join(new_lines), encoding="utf-8")
    print(f"{path}: module {name} ({size} lignes) -> {new_path}")


def main(argv):
    try:
        if len(argv) == 3 and argv[1] == "--list":
            cmd_list(Path(argv[2]))
        elif len(argv) == 4 and argv[1] == "--extract":
            cmd_extract(Path(argv[2]), int(argv[3]))
        else:
            print(__doc__, file=sys.stderr)
            return 1
    except FileNotFoundError as err:
        print(f"REFUS : fichier introuvable : {err.filename}", file=sys.stderr)
        return 2
    except Refus as err:
        print(f"REFUS : {err}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
