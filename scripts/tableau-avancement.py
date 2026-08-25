#!/usr/bin/env python3
"""Régénère le tableau d'avancement des blocs dans exigences-coordination-v2.md.

Compte les cases `- [ ]` / `- [x]` par bloc, réécrit le tableau entre les
marqueurs, et rend un code de sortie non nul si le document n'a pas pu être
mis à jour — un tableau périmé qui se croit à jour est pire qu'un tableau
absent.

Usage :
    tableau-avancement.py            # met à jour le document
    tableau-avancement.py --check    # affiche sans écrire
"""

import re
import sys
from pathlib import Path

DOC = Path(__file__).resolve().parent.parent / "specs/011-maicie-orchestration/exigences-coordination-v2.md"
DEBUT = "<!-- TABLEAU-AVANCEMENT:DEBUT -->"
FIN = "<!-- TABLEAU-AVANCEMENT:FIN -->"

# Un en-tête de bloc : "## Bloc G — ..." ou "**B. Piste GUI**"
HDR = re.compile(r"^(?:#{2,3}\s+|\*\*)(Bloc [A-Z][^\n*]{0,60}|[A-Z]\.\s[^\n*]{0,50})")
CASE = re.compile(r"^\s*[-*] \[( |x|X)\]")


def compter(texte):
    """Rend [(nom_bloc, faites, total)] dans l'ordre d'apparition."""
    bloc, stats, ordre = None, {}, []
    for ligne in texte.split("\n"):
        m = HDR.match(ligne.strip())
        if m:
            bloc = m.group(1).strip().rstrip("*").strip()
            if bloc not in stats:
                stats[bloc] = [0, 0]
                ordre.append(bloc)
            continue
        c = CASE.match(ligne)
        if c and bloc:
            stats[bloc][1] += 1
            if c.group(1).lower() == "x":
                stats[bloc][0] += 1
    return [(b, *stats[b]) for b in ordre if stats[b][1]]


def rendre(lignes):
    """Tableau markdown, blocs triés par avancement décroissant."""
    lignes = sorted(lignes, key=lambda x: (-x[1] / x[2], x[0]))
    out = ["| Bloc | Fait | % |", "|---|---|---|"]
    tf = tt = 0
    for nom, fait, total in lignes:
        tf += fait
        tt += total
        pct = round(100 * fait / total)
        gras = "**" if pct in (0, 100) else ""
        out.append(f"| {nom} | {fait}/{total} | {gras}{pct}{gras} |")
    out.append(f"| **Total** | **{tf}/{tt}** | **{round(100 * tf / tt)}** |")
    return "\n".join(out)


def main():
    if not DOC.exists():
        print(f"ABSENT : {DOC}", file=sys.stderr)
        return 2
    texte = DOC.read_text(encoding="utf-8")
    tableau = rendre(compter(texte))

    if "--check" in sys.argv:
        print(tableau)
        return 0

    if DEBUT not in texte or FIN not in texte:
        print(f"MARQUEURS ABSENTS de {DOC} : ajouter {DEBUT} et {FIN}", file=sys.stderr)
        print(tableau)
        return 3

    avant, reste = texte.split(DEBUT, 1)
    _, apres = reste.split(FIN, 1)
    DOC.write_text(f"{avant}{DEBUT}\n{tableau}\n{FIN}{apres}", encoding="utf-8")
    print(tableau)
    return 0


if __name__ == "__main__":
    sys.exit(main())
