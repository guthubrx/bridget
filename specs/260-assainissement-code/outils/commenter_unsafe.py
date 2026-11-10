#!/usr/bin/env python3
"""Ajoute un commentaire `// SAFETY:` au-dessus des blocs `unsafe` signalés (programme 260, lot 6).

Usage :
    python3 -I commenter_unsafe.py LISTE.txt [--dry-run]

LISTE.txt : une ligne `chemin:ligne` par bloc signalé (sortie triée de clippy).
Méthode : le commentaire n'affirme que ce que la syntaxe du bloc permet de vérifier.
  - appel système sans pointeur (kill, close, getpid...) : phrase fixe ;
  - appel avec pointeur vers un tableau (poll, read, write) : la phrase n'est écrite que si
    le pointeur et la longueur viennent de la même variable (`x.as_mut_ptr(), x.len()`) ;
  - appel avec `&mut variable` locale (pipe, tcgetattr, getrlimit...) : phrase fixe ;
  - structure C initialisée par `zeroed()` : phrase fixe ;
  - tout le reste : phrase de repli qui dit que l'invariant n'est pas établi par lecture locale.
Le commentaire est inséré au-dessus de la ligne qui contient `unsafe`.
"""
import re
import sys
from collections import defaultdict
from pathlib import Path

NOPTR = {"kill", "close", "getpid", "getppid", "getuid", "geteuid", "getgid", "getegid", "getpgid",
         "setpgid", "setsid", "umask", "dup", "dup2", "isatty", "sysconf", "alarm", "sleep", "usleep",
         "nice", "raise", "ftruncate", "fsync", "fchmod", "fchown", "lseek", "tcsetpgrp",
         "tcgetpgrp", "sched_yield", "getpgrp", "listen", "killpg"}
LOCALPTR = {"pipe", "pipe2", "tcgetattr", "tcsetattr", "getrlimit", "setrlimit", "flock", "fstat",
            "getrusage", "gettimeofday", "clock_gettime", "waitpid", "socketpair", "getsockopt",
            "setsockopt", "openpty", "timegm", "localtime_r", "gmtime_r", "mktime", "mkfifo", "statvfs",
            "fstatvfs", "sigaction", "sigprocmask", "getpeername", "getsockname", "poll", "cfmakeraw", "ioctl"}

REPLI = "invariant non établi par lecture locale ; revue demandée (programme 260)."


def block_text(lines, i, col):
    """Texte du bloc unsafe qui commence à lines[i][col:] (accolades équilibrées, bornes de sécurité)."""
    text = "\n".join(lines[i : i + 80])
    k = text.find("unsafe", col)
    if k < 0:
        return ""
    j = text.find("{", k)
    if j < 0:
        return ""
    depth = 0
    for p in range(j, len(text)):
        if text[p] == "{":
            depth += 1
        elif text[p] == "}":
            depth -= 1
            if depth == 0:
                return text[k : p + 1]
    return text[k : k + 400]


def phrase(block):
    calls = re.findall(r"libc::([a-zA-Z_0-9]+)\s*\(", block)
    if not calls and "unsafe fn" in block:
        return REPLI
    if not calls:
        return REPLI
    names = []
    for c in calls:
        if c not in names:
            names.append(c)
    parts = []
    for c in names:
        if c in NOPTR:
            if c == "kill" and re.search(r"kill\([^,]+,\s*0\s*\)", block):
                parts.append("kill(pid, 0) est l'idiome POSIX de sonde d'existence ; aucun signal n'est envoyé")
            else:
                parts.append(f"{c} ne déréférence aucun pointeur ; ses arguments sont des entiers")
        elif c in ("read", "write"):
            m = re.search(r"libc::%s\([^,]+,\s*([A-Za-z_0-9\.\[\]&]+?)\.as_(?:mut_)?ptr\(\)[^,]*,\s*([A-Za-z_0-9\.\[\]&]+?)\.len\(\)" % c, block)
            lit = re.search(r"libc::%s\([^,]+,\s*(b\"(?:[^\"\\\\]|\\\\.)*\")\.as_ptr\(\)(?:\.cast\(\))?,\s*(\d+)\s*\)" % c, block)
            byte = re.search(r"libc::%s\([^,]+,\s*\(&mut\s+([A-Za-z_0-9]+)\s+as\s+\*mut\s+u8\)\.cast\(\),\s*1\s*\)" % c, block)
            if m and m.group(1).replace("&", "") == m.group(2).replace("&", ""):
                parts.append(f"le tampon `{m.group(1)}` est valide pour sa longueur pendant l'appel {c}")
            elif lit and len(__import__("ast").literal_eval(lit.group(1))) == int(lit.group(2)):
                parts.append(f"le littéral d'octets fait exactement {lit.group(2)} octets, longueur passée à {c}")
            elif byte:
                parts.append(f"`{byte.group(1)}` est un octet local valide ; la longueur passée à {c} vaut 1")
            else:
                return REPLI
        elif c == "fcntl":
            if re.search(r"fcntl\([^,]+,\s*libc::F_(?:GETFL|SETFL|GETFD|SETFD|DUPFD|DUPFD_CLOEXEC)\b", block) and not re.search(r"&|\bptr\b|as_ptr|as_mut_ptr", block):
                parts.append("fcntl avec une commande à argument entier ne déréférence aucun pointeur")
            elif re.search(r"fcntl\([^,]+,\s*libc::F_[A-Z_]+\s*\)", block):
                parts.append("fcntl sans argument ne déréférence aucun pointeur")
            else:
                return REPLI
        elif c == "send":
            m = re.search(r"libc::send\([^,]+,\s*([A-Za-z_0-9\.]+)\.as_ptr\(\)[^,]*,\s*([A-Za-z_0-9\.]+)\.len\(\)", block)
            if m and m.group(1) == m.group(2):
                parts.append(f"le tampon `{m.group(1)}` est valide pour sa longueur pendant l'appel send")
            else:
                return REPLI
        elif c in LOCALPTR:
            if c == "poll":
                m = re.search(r"libc::poll\(\s*([A-Za-z_0-9\.]+)\.as_mut_ptr\(\)\s*,\s*([A-Za-z_0-9\.]+)\.len\(\)", block)
                one = re.search(r"libc::poll\(\s*&mut\s+([A-Za-z_0-9\.]+)\s*,\s*1\s*,", block)
                if m and m.group(1) == m.group(2):
                    parts.append(f"`{m.group(1)}` est un tableau de pollfd valide pour sa longueur pendant l'appel")
                elif one:
                    parts.append(f"`{one.group(1)}` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1")
                else:
                    return REPLI
            elif re.search(r"&mut\s+[A-Za-z_0-9\.]+|,\s*&[A-Za-z_0-9\.]+\s*\)|as_mut_ptr\(\)|\bptr::null", block) or c in ("flock",):
                parts.append(f"les pointeurs passés à {c} désignent des variables locales valides pendant l'appel")
            else:
                return REPLI
        else:
            return REPLI
    if "zeroed()" in block:
        parts.append("la structure C initialisée à zéro est valide (entiers seulement)")
    # Un bloc qui contient autre chose que ces appels : on ne généralise pas.
    inner = re.sub(r"libc::[A-Za-z_0-9]+", "", block)
    if re.search(r"from_raw_fd|from_raw|transmute|set_var|remove_var|get_unchecked|slice::from_raw_parts|CStr::from_ptr|as_ptr\(\)\s*as\s", inner):
        return REPLI
    return "; ".join(parts) + "."


def main(argv):
    if len(argv) < 2:
        print(__doc__, file=sys.stderr)
        return 1
    dry = "--dry-run" in argv
    by_file = defaultdict(set)
    for raw in Path(argv[1]).read_text().split("\n"):
        raw = raw.strip()
        if not raw:
            continue
        m = re.match(r"^(.*?):(\d+)(?::(\d+))?$", raw)
        if not m:
            continue
        by_file[m.group(1)].add((int(m.group(2)), int(m.group(3) or 1)))
    stats = defaultdict(int)
    for f, locs in by_file.items():
        p = Path(f)
        lines = p.read_text(encoding="utf-8").split("\n")
        per_line = defaultdict(list)
        for ln, col in sorted(locs):
            per_line[ln].append(col)
        edits = []
        for ln, cols in per_line.items():
            i = ln - 1
            texts = []
            for col in cols:
                texts.append(phrase(block_text(lines, i, max(col - 1, 0))))
            uniq = []
            for t in texts:
                if t not in uniq:
                    uniq.append(t)
            if REPLI in uniq and len(uniq) > 1:
                uniq = [REPLI]
            stats["repli" if uniq == [REPLI] else "phrase"] += 1
            ind = lines[i][: len(lines[i]) - len(lines[i].lstrip())]
            prev = lines[i - 1].strip() if i > 0 else ""
            if prev.startswith("// SAFETY"):
                continue
            edits.append((i, ind + "// SAFETY: " + " ".join(uniq)))
        for i, text in sorted(edits, reverse=True):
            lines.insert(i, text)
        if not dry:
            p.write_text("\n".join(lines), encoding="utf-8")
    print(f"{stats['phrase']} commentaires précis, {stats['repli']} de repli, {len(by_file)} fichiers")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
