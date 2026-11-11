#!/usr/bin/env python3
"""Regroupe les paramètres d'une fonction Rust dans une structure d'arguments (programme 260, lot 9).

Usage :
    python3 -I regrouper_arguments.py FICHIER NOM_FONCTION

La fonction `NOM_FONCTION` définie dans FICHIER reçoit un seul paramètre `args: <Nom>Args<'_>`.
Première ligne du corps : `let <Nom>Args { champs.. } = args;` (le corps ne change pas).
Les appels de tout le dépôt (`crates/**/*.rs`) sont réécrits en littéral de structure.
Refus (code 2, rien n'est écrit) : `self`, génériques, `impl`, `dyn Fn`, `fn`, durées de vie
explicites, motif de paramètre, nombre d'arguments d'un appel différent, plus de 30 appels.
"""
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from extraire_branches import Refus, match_close, skip_literal, split_top  # noqa: E402


def camel(name):
    return "".join(p.capitalize() for p in name.split("_"))


def find_def(text, name):
    ms = list(re.finditer(r"^[ \t]*((?:pub(?:\([a-z]+\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?)fn\s+%s\b" % re.escape(name), text, re.M))
    if len(ms) != 1:
        raise Refus(f"{len(ms)} définitions de {name} dans le fichier")
    return ms[0]


def main(argv):
    if len(argv) != 3:
        print(__doc__, file=sys.stderr)
        return 1
    path, name = Path(argv[1]), argv[2]
    try:
        text = path.read_text(encoding="utf-8")
        m = find_def(text, name)
        open_i = text.index("(", m.end() - 0)
        if "<" in text[m.end() : open_i]:
            raise Refus("fonction générique")
        close_i = match_close(text, open_i)
        raw = text[open_i + 1 : close_i]
        params = []
        for part in split_top(raw):
            part = re.sub(r"(?s)#\[.*?\]\s*", "", part).strip()
            if not part:
                continue
            if part.startswith(("self", "&self", "&mut self", "mut self")):
                raise Refus("méthode avec self")
            mm = re.fullmatch(r"(?s)(mut\s+)?(\w+)\s*:\s*(.+)", part)
            if not mm:
                raise Refus(f"paramètre non pris en charge : {part[:40]}")
            ty = mm.group(3).strip()
            if re.search(r"\bimpl\b|\bfn\b|Fn\(|dyn\s+Fn|'[a-z_]", ty.replace("'static", "")):
                raise Refus(f"type non pris en charge : {ty[:40]}")
            params.append((bool(mm.group(1)), mm.group(2), ty))
        if len(params) < 2:
            raise Refus("moins de deux paramètres")
        # corps
        brace = text.index("{", close_i)
        body_end = match_close(text, brace)
        sig_tail = text[close_i + 1 : brace]
        has_ref = any("&" in t for _, _, t in params)
        sname = camel(name) + "Args"
        gen = "<'a>" if has_ref else ""
        use_gen = "<'_>" if has_ref else ""
        def lt(t):
            return re.sub(r"&(?!')(mut\s+)?", lambda x: "&'a " + (x.group(1) or ""), t) if has_ref else t
        fields = "\n".join(f"    pub {n}: {lt(t)}," for _, n, t in params)
        vis = re.sub(r"\b(const|async|unsafe)\b\s*", "", m.group(1)).strip()
        struct = f"{vis + ' ' if vis else ''}struct {sname}{gen} {{\n{fields}\n}}\n\n"
        names = ", ".join(("mut " if mu else "") + n for mu, n, _ in params)
        indent = re.match(r"[ \t]*", text[m.start() :]).group(0)
        # appels
        edits = []
        total = 0
        for p in sorted(Path("crates").rglob("*.rs")):
            t = p.read_text(encoding="utf-8")
            if not re.search(r"\b%s\b" % re.escape(name), t):
                continue
            pos = 0
            local = []
            for c in re.finditer(r"(?<![\w])((?:[A-Za-z_]\w*::)*)%s\s*\(" % re.escape(name), t):
                before = t[max(0, c.start() - 12) : c.start()]
                if re.search(r"fn\s+$", before) or t[c.start() - 1 : c.start()] == ".":
                    continue
                o = c.end() - 1
                e = match_close(t, o)
                args = [a.strip() for a in split_top(t[o + 1 : e]) if a.strip()]
                if len(args) != len(params):
                    raise Refus(f"{p}: appel à {len(args)} arguments (attendu {len(params)})")
                lits = ",\n".join(n if a == n else f"{n}: {a}" for (mu, n, ty), a in zip(params, args))
                local.append((c.start(), e + 1, f"{c.group(1)}{name}({c.group(1)}{sname} {{\n{lits},\n}})", c.group(1)))
                total += 1
            if local:
                edits.append((p, t, local))
        if total > 30:
            raise Refus(f"{total} appels (maximum 30)")
        # écriture : appels d'abord (hors fichier de définition géré ensuite)
        for p, t, local in edits:
            for s, e, rep, qual in sorted(local, reverse=True):
                t = t[:s] + rep + t[e:]
            if p == path:
                text = t
                # recalculer la définition dans le texte modifié
                m = find_def(text, name)
            else:
                # import de la structure à côté de l'import de la fonction
                def addimp(mo):
                    return mo.group(0).replace(name, f"{name}, {sname}", 1) if re.search(r"\b%s\b" % re.escape(name), mo.group(0)) else mo.group(0)
                t = re.sub(r"(?m)^(\s*(?:pub\s+)?use\s[^;]*\b%s\b[^;]*;)" % re.escape(name), addimp, t)
                p.write_text(t, encoding="utf-8")
        # définition
        open_i = text.index("(", m.end())
        close_i = match_close(text, open_i)
        brace = text.index("{", close_i)
        new_sig = text[: open_i + 1] + f"args: {sname}{use_gen}" + text[close_i : brace + 1] + f"\n    let {sname} {{ {names} }} = args;"
        text = new_sig + text[brace + 1 :]
        mm2 = find_def(text, name)
        block = "".join((indent + l if l else l) + "\n" for l in struct.rstrip("\n").split("\n")) + "\n"
        line_start = text.rfind("\n", 0, mm2.start()) + 1
        text = text[:line_start] + block + text[line_start:]
        path.write_text(text, encoding="utf-8")
        print(f"{name}: {len(params)} paramètres -> {sname}, {total} appels réécrits")
    except Refus as err:
        print(f"REFUS : {err}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
