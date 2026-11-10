#!/usr/bin/env python3
"""Déplace une branche de `match msg` vers une fonction `on_xxx` (programme 260, lot 5).

Usage :
    python3 -I extraire_branches.py --list
    python3 -I extraire_branches.py NomVariante [NomVariante ...]

--list     montre les branches de `handle_wrapper_message_with_peer` avec leur taille en lignes.
NomVariante  déplace la branche `WrapperToDaemon::NomVariante ... => ...` vers
             `crates/bridget-daemon/src/daemon/wrapper_dispatch.rs`.
Règles :
- le corps est copié sans changement (même texte, mêmes `return`) dans `pub(super) fn on_<nom>(...)` ;
- les paramètres sont les variables liées par le motif (types lus dans l'énumération du protocole)
  puis les variables de contexte réellement utilisées (conn_id, state, peer) ;
- refus (code 2, rien n'est écrit) si le motif contient une alternative `|`, un `ref`, si le corps
  utilise `msg`, `_update_activity`, `attach_refusal`, `service_refusal` ou `client_refusal`,
  ou si la branche est introuvable.
"""
import re
import sys
from pathlib import Path

DAEMON = Path("crates/bridget-daemon/src/daemon.rs")
DISPATCH = Path("crates/bridget-daemon/src/daemon/wrapper_dispatch.rs")
PROTO = Path("crates/bridget-transport/src/protocol.rs")
CTX = {"conn_id": "&str", "state": "&Arc<Mutex<DaemonState>>", "peer": "Option<&UnixStream>"}
FORBIDDEN = ("msg", "_update_activity", "attach_refusal", "service_refusal", "client_refusal")


class Refus(Exception):
    pass


def skip_literal(t, i):
    """Si t[i] ouvre une chaîne, un caractère ou un commentaire, retourne l'indice après lui, sinon None."""
    c = t[i]
    if t.startswith("//", i):
        j = t.find("\n", i)
        return len(t) if j < 0 else j
    if t.startswith("/*", i):
        j = t.find("*/", i + 2)
        return len(t) if j < 0 else j + 2
    m = re.match(r'b?r(#*)"', t[i:])
    if m:
        end = '"' + m.group(1)
        j = t.find(end, i + m.end())
        return len(t) if j < 0 else j + len(end)
    if c == '"' or (c == "b" and t.startswith('b"', i)):
        j = i + (2 if c == "b" else 1)
        while j < len(t):
            if t[j] == "\\":
                j += 2
                continue
            if t[j] == '"':
                return j + 1
            j += 1
        return len(t)
    m = re.match(r"b?'(\\.[^']*|[^\\'])'", t[i:])
    if m:
        return i + m.end()
    return None


def scan(t, start, stop_chars, depth0=0):
    """Premier indice >= start d'un des stop_chars à profondeur 0 (hors littéraux)."""
    depth = depth0
    i = start
    while i < len(t):
        j = skip_literal(t, i)
        if j is not None:
            i = j
            continue
        c = t[i]
        if depth == 0 and any(t.startswith(s, i) for s in stop_chars):
            return i
        if c in "({[":
            depth += 1
        elif c in ")}]":
            depth -= 1
            if depth < 0:
                return i
        i += 1
    return -1


def match_close(t, i):
    """t[i] est une ouverture : retourne l'indice de la fermeture correspondante."""
    depth = 0
    j = i
    while j < len(t):
        k = skip_literal(t, j)
        if k is not None:
            j = k
            continue
        if t[j] in "({[":
            depth += 1
        elif t[j] in ")}]":
            depth -= 1
            if depth == 0:
                return j
        j += 1
    raise Refus("délimiteur non fermé")


def snake(name):
    return re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()


def match_bounds(text):
    f = text.index("fn handle_wrapper_message_with_peer(")
    m = re.compile(r"^    match msg \{$", re.M).search(text, f)
    if not m:
        raise Refus("match msg introuvable")
    open_i = text.index("{", m.start())
    close_i = match_close(text, open_i)
    return open_i + 1, close_i


def arms(text, a, b):
    """Liste de (début, fin_motif, début_corps, fin_arm) pour chaque branche de colonne 8."""
    out = []
    i = a
    while True:
        m = re.compile(r"^        (WrapperToDaemon::\w+)", re.M).search(text, i, b)
        if not m:
            break
        s = m.start() + 8
        arrow = scan(text, s, ["=>"])
        if arrow < 0 or arrow > b:
            break
        j = arrow + 2
        while text[j] in " \n\t":
            j += 1
        if text[j] == "{":
            e = match_close(text, j) + 1
            body_start = j
            tail = e
            if text[tail] == ",":
                tail += 1
        else:
            comma = scan(text, j, [","])
            e = comma
            body_start = j
            tail = comma + 1
        out.append((s, arrow, body_start, e, tail))
        i = tail
    return out


def proto_variants():
    t = PROTO.read_text(encoding="utf-8")
    k = t.index("pub enum WrapperToDaemon")
    o = t.index("{", k)
    c = match_close(t, o)
    body = t[o + 1 : c]
    res = {}
    i = 0
    while i < len(body):
        m = re.compile(r"^    (?:#\[[^\n]*\]\n    )*(\w+)", re.M).search(body, i)
        if not m:
            break
        name = m.group(1)
        j = m.end()
        while body[j] in " \n":
            j += 1
        if body[j] == "(":
            e = match_close(body, j)
            inner = body[j + 1 : e]
            fields = [x.strip() for x in split_top(inner)]
            res[name] = ("tuple", [strip_attr(x) for x in fields if x])
            i = e + 1
        elif body[j] == "{":
            e = match_close(body, j)
            inner = body[j + 1 : e]
            d = {}
            order = []
            for part in split_top(inner):
                part = strip_attr(part.strip())
                if ":" in part:
                    n, ty = part.split(":", 1)
                    n = n.replace("pub", "").strip()
                    d[n] = ty.strip()
                    order.append(n)
            res[name] = ("struct", d)
            i = e + 1
        else:
            res[name] = ("unit", None)
            i = j
        i = body.find(",", i) + 1 if body.find(",", i) >= 0 else len(body)
    return res


def split_top(s):
    parts, depth, cur, i = [], 0, "", 0
    while i < len(s):
        j = skip_literal(s, i)
        if j is not None:
            cur += s[i:j]
            i = j
            continue
        c = s[i]
        if c in "({[<":
            depth += 1
        elif c in ")}]>":
            if not (c == ">" and s[i - 1] == "-"):
                depth -= 1
        if c == "," and depth == 0:
            parts.append(cur)
            cur = ""
        else:
            cur += c
        i += 1
    if cur.strip():
        parts.append(cur)
    return parts


def strip_attr(x):
    x = re.sub(r"(?s)#\[.*?\]\s*", "", x)
    x = re.sub(r"(?m)^\s*///.*$", "", x)
    return x.strip()


def bindings(pattern, kind, defs, variant):
    """Retourne [(mut, nom, type)] dans l'ordre du motif."""
    pat = pattern[len("WrapperToDaemon::" + variant) :].strip()
    if "|" in re.sub(r'"[^"]*"', "", pat):
        raise Refus("motif avec alternative")
    if re.search(r"\bref\b", pat):
        raise Refus("motif avec ref")
    out = []
    if kind == "tuple" and pat.startswith("("):
        inner = pat[1:-1]
        for idx, part in enumerate(split_top(inner)):
            part = part.strip()
            if part in ("_", ".."):
                continue
            m = re.fullmatch(r"(mut\s+)?(\w+)", part)
            if not m:
                raise Refus(f"motif non pris en charge : {part}")
            out.append((bool(m.group(1)), m.group(2), defs[idx]))
    elif kind == "struct" and pat.startswith("{"):
        inner = pat[1:-1]
        for part in split_top(inner):
            part = part.strip()
            if not part or part == "..":
                continue
            m = re.fullmatch(r"(\w+)\s*:\s*(mut\s+)?(\w+)", part)
            if m:
                out.append((bool(m.group(2)), m.group(3), defs[m.group(1)]))
                continue
            m = re.fullmatch(r"(mut\s+)?(\w+)", part)
            if not m:
                raise Refus(f"motif non pris en charge : {part}")
            out.append((bool(m.group(1)), m.group(2), defs[m.group(2)]))
    return out


def run(variants):
    text = DAEMON.read_text(encoding="utf-8")
    a, b = match_bounds(text)
    defs = proto_variants()
    todo = []
    for v in variants:
        found = None
        for s, arrow, bs, e, tail in arms(text, a, b):
            if text[s:].startswith("WrapperToDaemon::" + v) and not re.match(r"\w", text[s + len("WrapperToDaemon::" + v)]):
                found = (s, arrow, bs, e, tail)
                break
        if not found:
            raise Refus(f"branche {v} introuvable")
        s, arrow, bs, e, tail = found
        pattern = text[s:arrow].strip()
        kind, d = defs[v]
        bind = bindings(pattern, kind, d, v)
        body = text[bs:e]
        if body.startswith("{"):
            fn_body = body
        else:
            fn_body = "{ " + body + " }"
        for w in FORBIDDEN:
            if re.search(r"(?<![\w.])%s\b" % w, fn_body):
                raise Refus(f"le corps de {v} utilise {w}")
        ctx = [c for c in CTX if re.search(r"(?<![\w.])%s\b" % c, fn_body)]
        params = ", ".join([("mut " if m else "") + n + ": " + ty for m, n, ty in bind] + [f"{c}: {CTX[c]}" for c in ctx])
        args = ", ".join([n for m, n, ty in bind] + ctx)
        name = "on_" + snake(v)
        fn = f"pub(super) fn {name}({params}) -> Option<DaemonToWrapper> {fn_body}\n"
        call = f"{pattern} => wrapper_dispatch::{name}({args}),"
        todo.append((s, tail, call, fn, v, body.count("\n") + 1))
    # Appliquer du bas vers le haut
    new_fns = ""
    for s, tail, call, fn, v, n in sorted(todo, reverse=True):
        text = text[:s] + call + text[tail:]
    for s, tail, call, fn, v, n in sorted(todo):
        new_fns += "\n" + fn
        print(f"{v}: {n} lignes -> {DISPATCH.name}")
    disp = DISPATCH.read_text(encoding="utf-8")
    if not disp.endswith("\n"):
        disp += "\n"
    DAEMON.write_text(text, encoding="utf-8")
    DISPATCH.write_text(disp + new_fns, encoding="utf-8")


def list_arms():
    text = DAEMON.read_text(encoding="utf-8")
    a, b = match_bounds(text)
    for s, arrow, bs, e, tail in arms(text, a, b):
        n = text[s:tail].count("\n") + 1
        first = re.match(r"WrapperToDaemon::(\w+)", text[s:]).group(1)
        print(f"{n:5d} {first}")


def main(argv):
    try:
        if len(argv) == 2 and argv[1] == "--list":
            list_arms()
        elif len(argv) >= 2:
            run(argv[1:])
        else:
            print(__doc__, file=sys.stderr)
            return 1
    except Refus as err:
        print(f"REFUS : {err}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
