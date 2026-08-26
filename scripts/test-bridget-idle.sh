#!/usr/bin/env bash
# Harnais bridget-idle : partition agents + backlog Git local en lecture seule.
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
idle="${root_dir}/scripts/bridget-idle.py"
system_python="/usr/bin/python3"
fixture_root="$(mktemp -d -t bridget-idle-test.XXXXXX)"
cleanup() { rm -rf "$fixture_root"; }
trap cleanup EXIT

[[ -f "$idle" ]] || { echo "absent: $idle" >&2; exit 1; }

"$system_python" - "$idle" "$fixture_root" <<'PY'
import hashlib, importlib.util, json, pathlib, sys, os, sqlite3, subprocess, time

idle_path = pathlib.Path(sys.argv[1])
fixture = pathlib.Path(sys.argv[2])
spec = importlib.util.spec_from_file_location("bridget_idle", idle_path)
mod = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(mod)

agents = [
    {"name": "alice", "state": "connected", "domain": "bridget", "last_seen_secs": 10},
    {"name": "bob", "state": "connected", "domain": "bridget", "last_seen_secs": 5},
    {"name": "cursor10-like", "state": "connected", "domain": "cursor10-lot", "last_seen_secs": 3},
    {"name": "cursorbridget-like", "state": "busy", "domain": "bridget", "last_seen_secs": 1},
    {"name": "relec6-like", "state": "busy", "domain": "relec6-lot", "last_seen_secs": 2},
    {"name": "bridget", "state": "connected", "domain": "bridget", "last_seen_secs": 0},
    # Consommateur sain vs figé (propriété mécanique, pas des noms d'instance).
    {"name": "healthy-consumer", "state": "busy", "domain": "bridget", "last_seen_secs": 2},
    {"name": "frozen-consumer", "state": "busy", "domain": "bridget", "last_seen_secs": 2},
]
occupied = {"bob", "healthy-consumer", "frozen-consumer"}
exclude = {"bridget", "fable", "poucave", "sol", "maicie"}
daemon = {a["name"] for a in agents}

# --- Contrôle positif omission (ancienne logique) ---
legacy = mod.classify_legacy(agents, occupied, exclude=exclude, silent_after_secs=1800)
ok_legacy, detail_legacy = mod.partition_oracle(legacy, daemon)
if ok_legacy:
    raise SystemExit(f"CONTROLE POSITIF RATE: l'ancienne logique aurait dû casser l'oracle ({detail_legacy})")
print(f"controle_positif_ancien_rouge: OK ({detail_legacy})")

legacy_covered = (
    {n for n, _ in legacy["libres"]}
    | {n for n, _ in legacy["muets"]}
    | set(legacy["occupes"])
    | {n for n, _ in legacy.get("bloques", [])}
    | {n for n, _ in legacy["indetermines"]}
)
for ghost in ("cursor10-like", "cursorbridget-like", "relec6-like"):
    if ghost in legacy_covered:
        raise SystemExit(f"CONTROLE POSITIF RATE: {ghost} visible dans l'ancienne logique")
print("controle_positif_trois_fantomes_omis: OK")

# Backlog : sain sous seuil (5 min), figé au-dessus (44 min) — bornes mesurées.
backlog = {"healthy-consumer": 5 * 60, "frozen-consumer": 44 * 60}
fixed = mod.classify(
    agents,
    occupied,
    exclude=exclude,
    silent_after_secs=1800,
    backlog_ages=backlog,
    blocked_after_secs=mod.BLOCKED_AFTER_SECS,
)
ok_fixed, detail_fixed = mod.partition_oracle(fixed, daemon)
if not ok_fixed:
    raise SystemExit(f"correctif ROUGE sur oracle: {detail_fixed}")
print(f"partition_correctif: OK ({detail_fixed})")

bloques = {n: s for n, s in fixed["bloques"]}
if "frozen-consumer" not in bloques:
    raise SystemExit(f"figé absente de BLOQUES: {bloques}")
if "healthy-consumer" in bloques:
    raise SystemExit(f"consommateur sain à tort BLOQUE: {bloques}")
if "healthy-consumer" not in fixed["occupes"]:
    raise SystemExit(f"sain devrait rester OCCUPE: {fixed['occupes']}")
if "frozen-consumer" in fixed["occupes"]:
    raise SystemExit("figé ne doit pas rester OCCUPE une fois BLOQUE")
print("controle_positif_bloques_deux_sens: OK")

# Sans backlog signal, le figé retombe OCCUPE (régression anti-silence sur la catégorie).
no_backlog = mod.classify(
    agents, occupied, exclude=exclude, silent_after_secs=1800, backlog_ages={}
)
if "frozen-consumer" not in no_backlog["occupes"]:
    raise SystemExit("sans backlog, frozen-consumer doit être OCCUPE")
print("bloques_sans_signal_reste_occupe: OK")

ind = {name: reason for name, reason in fixed["indetermines"]}
if "cursorbridget-like" not in ind or "busy-sans-mission-greffe" not in ind["cursorbridget-like"]:
    raise SystemExit(f"cursorbridget-like mal classé: {ind}")
libres = {n for n, _ in fixed["libres"]}
if "cursor10-like" not in libres:
    raise SystemExit(f"cursor10-like devrait être LIBRE: libres={libres} ind={ind}")
if "alice" not in libres or "bob" not in fixed["occupes"]:
    raise SystemExit(f"branches saines cassées: libres={libres} occupes={fixed['occupes']}")
if "bridget" not in ind:
    raise SystemExit("bridget (hors-perimetre) doit apparaître en INDETERMINES")
print("semantique_correctif: OK")

# Seuil = milieu du trou mesuré [7, 44] min → 25 min ; compile 10 min sous le seuil.
if mod.BLOCKED_AFTER_SECS != 25 * 60:
    raise SystemExit(f"seuil inattendu: {mod.BLOCKED_AFTER_SECS}")
ten_min = mod.classify(
    [{"name": "compiler", "state": "busy", "domain": "bridget", "last_seen_secs": 1}],
    {"compiler"},
    exclude=set(),
    silent_after_secs=1800,
    backlog_ages={"compiler": 10 * 60},
)
if ten_min["bloques"] or ten_min["occupes"] != ["compiler"]:
    raise SystemExit(f"compile 10 min ne doit pas être BLOQUE: {ten_min}")
print("seuil_compile_10min_sous_coupure: OK")

# Lecture Maicie sur copie : sidecars source intacts
db = fixture / "maicie.sqlite3"
conn = sqlite3.connect(db)
conn.execute("PRAGMA journal_mode=WAL")
conn.execute("CREATE TABLE objectives(id TEXT PRIMARY KEY, state TEXT NOT NULL)")
conn.execute("CREATE TABLE delegations(id TEXT PRIMARY KEY, objective_id TEXT NOT NULL, payload_json TEXT NOT NULL)")
conn.execute("INSERT INTO objectives VALUES ('o1', 'en_coordination')")
conn.execute("INSERT INTO delegations VALUES ('d1', 'o1', '{\"participant\":\"bob\"}')")
conn.commit()
assert (fixture / "maicie.sqlite3-wal").exists()
wal_before = os.stat(db.with_name(db.name + "-wal")).st_mtime_ns
shm_before = os.stat(db.with_name(db.name + "-shm")).st_mtime_ns
config = fixture / "config.json"
config.write_text(json.dumps({"database_path": str(db)}), encoding="utf-8")
occ, err = mod.read_occupied_from_maicie_copy(str(config))
if err or occ != {"bob"}:
    raise SystemExit(f"lecture copie Maicie: occ={occ} err={err}")
wal_after = os.stat(db.with_name(db.name + "-wal")).st_mtime_ns
shm_after = os.stat(db.with_name(db.name + "-shm")).st_mtime_ns
if (wal_before, shm_before) != (wal_after, shm_after):
    raise SystemExit("lecture idle a touché les sidecars de production (copie obligatoire)")
print("maicie_copie_sidecars_intacts: OK")

# Bridget DB copie : requête backlog + sidecars intacts
bdb = fixture / "bridget.db"
bconn = sqlite3.connect(bdb)
bconn.execute("PRAGMA journal_mode=WAL")
bconn.execute("CREATE TABLE ledger(id TEXT PRIMARY KEY, ts INTEGER NOT NULL, sender TEXT, target TEXT NOT NULL, body TEXT, conversation_key TEXT)")
bconn.execute(
    "CREATE TABLE send_deliveries(delivery_id TEXT PRIMARY KEY, issuer_scope TEXT, operation_kind TEXT, "
    "idempotency_key TEXT, recipient_instance_id TEXT, delivery_generation INTEGER, phase TEXT, expires_at INTEGER, message_bytes BLOB)"
)
now = 1_000_000
bconn.execute("INSERT INTO ledger VALUES ('m-fresh', ?, 'a', 'healthy-consumer', 'x', 'c')", (now - 300,))
bconn.execute("INSERT INTO ledger VALUES ('m-old', ?, 'a', 'frozen-consumer', 'x', 'c')", (now - 44 * 60,))
bconn.execute(
    "INSERT INTO send_deliveries VALUES ('d1','s','send','m-fresh','r',1,'dispatching',?,NULL)",
    (now + 600000,),
)
bconn.execute(
    "INSERT INTO send_deliveries VALUES ('d2','s','send','m-old','r',1,'dispatching',?,NULL)",
    (now + 600000,),
)
# Orphelin sans ledger : ne doit PAS inventer un âge
bconn.execute(
    "INSERT INTO send_deliveries VALUES ('d3','s','send','orphan-key','r',1,'dispatching',?,NULL)",
    (now + 600000,),
)
bconn.commit()
assert (fixture / "bridget.db-wal").exists()
b_wal_before = os.stat(str(bdb) + "-wal").st_mtime_ns
ages, berr = mod.read_backlog_ages_from_bridget_copy(str(bdb), now=now)
if berr:
    raise SystemExit(berr)
if ages.get("healthy-consumer") != 300 or ages.get("frozen-consumer") != 44 * 60:
    raise SystemExit(f"backlog ages inattendus: {ages}")
if "orphan" in str(ages):
    raise SystemExit(f"orphelin sans ledger a fuité: {ages}")
b_wal_after = os.stat(str(bdb) + "-wal").st_mtime_ns
if b_wal_before != b_wal_after:
    raise SystemExit("lecture backlog a touché le WAL source")
print("backlog_copie_ledger_join_seule: OK")

# Dépôt réel jetable : présence avant absence, conflit, base périmée et zéro
# mutation de l'object store observé.
git_now = 2_000_000_000
git_env = os.environ.copy()
git_env.update(
    {
        "GIT_AUTHOR_NAME": "Test User",
        "GIT_AUTHOR_EMAIL": "test@example.invalid",
        "GIT_COMMITTER_NAME": "Test User",
        "GIT_COMMITTER_EMAIL": "test@example.invalid",
    }
)


def git(cwd, *arguments, timestamp=None, check=True):
    env = git_env.copy()
    if timestamp is not None:
        stamp = f"@{timestamp} +0000"
        env["GIT_AUTHOR_DATE"] = stamp
        env["GIT_COMMITTER_DATE"] = stamp
    completed = subprocess.run(
        ["git", *arguments],
        cwd=cwd,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if check and completed.returncode:
        raise SystemExit(
            f"git {' '.join(arguments)}: {(completed.stderr or completed.stdout).strip()}"
        )
    return completed.stdout.strip(), completed.returncode


def commit_file(repository, relative, content, message, timestamp):
    target = repository / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")
    git(repository, "add", relative)
    git(repository, "commit", "-m", message, timestamp=timestamp)


remote = fixture / "origin.git"
repository = fixture / "repository"
git(fixture, "init", "--bare", str(remote))
git(fixture, "init", "-b", "main", str(repository))
commit_file(repository, "shared.txt", "base\n", "base", git_now - 8 * 3600)
git(repository, "remote", "add", "origin", str(remote))
git(repository, "push", "-u", "origin", "main")

git(repository, "switch", "-c", "merged-lot")
commit_file(repository, "merged.txt", "livré puis fusionné\n", "merged", git_now - 6 * 3600)
git(repository, "push", "-u", "origin", "merged-lot")
git(repository, "switch", "main")
git(repository, "merge", "--ff-only", "merged-lot")
git(repository, "push", "origin", "main")

git(repository, "switch", "-c", "pending-lot")
commit_file(repository, "pending.txt", "corps exact du lot en attente\n", "pending", git_now - 2 * 3600)
git(repository, "push", "-u", "origin", "pending-lot")
pending_head, _ = git(repository, "rev-parse", "HEAD")
git(repository, "switch", "main")

git(repository, "switch", "-c", "conflict-lot")
commit_file(repository, "shared.txt", "branche\n", "conflict branch", git_now - 3 * 3600)
git(repository, "push", "-u", "origin", "conflict-lot")
git(repository, "switch", "main")
commit_file(repository, "shared.txt", "main\n", "conflict main", git_now - 3600)
git(repository, "push", "origin", "main")

git(repository, "switch", "--orphan", "stale-lot")
git(repository, "rm", "-rf", "--ignore-unmatch", ".")
commit_file(repository, "stale.txt", "histoire étrangère\n", "stale", git_now - 4 * 3600)
git(repository, "push", "-u", "origin", "stale-lot")
git(repository, "switch", "main")

# La branche existe sur le remote mais pas dans les refs locales : l'analyse
# sans fetch doit l'ignorer et l'annoncer comme limite.
git(fixture, "--git-dir", str(remote), "update-ref", "refs/heads/unseen-lot", pending_head)


def git_inventory(repo):
    refs, _ = git(repo, "show-ref")
    git_root = repo / ".git"
    git_files = sorted(
        (
            str(path.relative_to(git_root)),
            path.stat().st_size,
            path.stat().st_mtime_ns,
            hashlib.sha256(path.read_bytes()).hexdigest(),
        )
        for path in git_root.rglob("*")
        if path.is_file()
    )
    worktree_files = sorted(
        (
            str(path.relative_to(repo)),
            path.stat().st_size,
            path.stat().st_mtime_ns,
            hashlib.sha256(path.read_bytes()).hexdigest(),
        )
        for path in repo.rglob("*")
        if path.is_file() and git_root not in path.parents
    )
    return refs, git_files, worktree_files


before = git_inventory(repository)
branch_backlog, branch_error = mod.read_branch_backlog(
    str(repository), now=git_now, timeout_secs=5.0
)
if branch_error or branch_backlog is None:
    raise SystemExit(f"branche_non_fusionnee_presente: analyse absente: {branch_error}")
by_ref = {item["ref"]: item for item in branch_backlog["lots"]}

# Oracle de PRÉSENCE d'abord : la tête exacte et l'âge doivent être lus.
pending = by_ref.get("origin/pending-lot")
if pending is None or pending["head"] != pending_head or pending["age_secs"] != 2 * 3600:
    raise SystemExit(f"branche_non_fusionnee_presente: attendu pending exact, reçu {pending}")
if pending["textual_merge"] != "sans conflit textuel":
    raise SystemExit(f"branche_non_fusionnee_presente: état inattendu {pending}")
print("branche_non_fusionnee_presente: OK (origin/pending-lot, tete et age exacts)")

# Oracle d'ABSENCE seulement après la présence : une implémentation morte qui
# renverrait toujours [] ne peut donc pas passer.
if "origin/merged-lot" in by_ref:
    raise SystemExit(
        "branche_fusionnee_absente_apres_presence: origin/merged-lot est encore visible"
    )
print("branche_fusionnee_absente_apres_presence: OK")

conflict = by_ref.get("origin/conflict-lot")
if conflict is None or conflict["textual_merge"] != "en conflit" or conflict["blocking"] != "en conflit":
    raise SystemExit(f"branche_en_conflit_visible: reçu {conflict}")
print("branche_en_conflit_visible: OK")

stale = by_ref.get("origin/stale-lot")
if (
    stale is None
    or stale["base_state"] != "perimee"
    or stale["textual_merge"] != "sans conflit textuel"
    or stale["blocking"] != "base perimee, a rebaser"
):
    raise SystemExit(f"branche_base_perimee_visible: reçu {stale}")
print("branche_base_perimee_visible: OK")

if "origin/unseen-lot" in by_ref:
    raise SystemExit("refs_locales_sans_fetch: une ref distante non récupérée a été inventée")
if branch_backlog["refs_scope"] != "refs locales sans fetch":
    raise SystemExit(f"refs_locales_sans_fetch: limite absente {branch_backlog}")
print("refs_locales_sans_fetch: OK")

after = git_inventory(repository)
if before != after:
    raise SystemExit("depot_git_lecture_seule: refs, fichiers Git ou worktree modifiés")
print("depot_git_lecture_seule: OK")

# Un Git lent est borné globalement et rend une indisponibilité, jamais [].
slow_git = fixture / "git-slow"
slow_git.write_text("#!/bin/sh\nexec sleep 10\n", encoding="utf-8")
slow_git.chmod(0o700)
started = time.monotonic()
slow_backlog, slow_error = mod.read_branch_backlog(
    str(repository), now=git_now, timeout_secs=0.05, git_bin=str(slow_git)
)
elapsed = time.monotonic() - started
if slow_backlog is not None or not slow_error or "delai Git" not in slow_error:
    raise SystemExit(
        f"backlog_branches_timeout_indisponible: backlog={slow_backlog} error={slow_error}"
    )
if elapsed >= 1.0:
    raise SystemExit(f"backlog_branches_timeout_indisponible: borne dépassée ({elapsed:.3f}s)")
print("backlog_branches_timeout_indisponible: OK")

bad_backlog, bad_error = mod.read_branch_backlog(
    str(fixture / "absent"), now=git_now, timeout_secs=1.0
)
if bad_backlog is not None or not bad_error:
    raise SystemExit(
        f"backlog_branches_depot_invalide_indisponible: backlog={bad_backlog} error={bad_error}"
    )
print("backlog_branches_depot_invalide_indisponible: OK")

agents_path = fixture / "agents.json"
occupied_path = fixture / "occupied.json"
backlog_path = fixture / "backlog.json"
agents_path.write_text(json.dumps(agents), encoding="utf-8")
occupied_path.write_text(json.dumps(sorted(occupied)), encoding="utf-8")
backlog_path.write_text(json.dumps(backlog), encoding="utf-8")
print(f"FIXTURES {agents_path} {occupied_path} {backlog_path}")
PY

agents_json="${fixture_root}/agents.json"
occupied_json="${fixture_root}/occupied.json"
backlog_json="${fixture_root}/backlog.json"
git_repo="${fixture_root}/repository"
out="$("$system_python" "$idle" --agents-json "$agents_json" --occupied-json "$occupied_json" --backlog-json "$backlog_json" --git-repo "$git_repo" --now 2000000000)"
grep -q 'BLOQUES' <<<"$out"
grep -q 'frozen-consumer' <<<"$out"
grep -q 'cursor10-like' <<<"$out"
grep -q 'BACKLOG BRANCHES INDISPONIBLE (greffe sans etat exploitable)' <<<"$out"
grep -q 'origin/pending-lot' <<<"$out"
if grep -q 'origin/merged-lot' <<<"$out"; then
  echo 'branche_fusionnee_absente_cli: origin/merged-lot visible' >&2
  exit 1
fi
grep -q 'refs locales sans fetch' <<<"$out"
grep -q 'age du commit de tete uniquement' <<<"$out"
echo 'cli_texte_branches_et_limites: OK'
slow_out="$("$system_python" "$idle" --agents-json "$agents_json" --occupied-json "$occupied_json" --backlog-json "$backlog_json" --git-repo "$git_repo" --git-bin "${fixture_root}/git-slow" --git-timeout-secs 0.05 --now 2000000000)"
grep -q 'BACKLOG BRANCHES INDISPONIBLE (delai Git depasse)' <<<"$slow_out"
if grep -q 'origin/pending-lot' <<<"$slow_out"; then
  echo 'backlog_branches_timeout_cli: liste partielle visible' >&2
  exit 1
fi
echo 'cli_timeout_sans_liste_partielle: OK'
json_out="$("$system_python" "$idle" --agents-json "$agents_json" --occupied-json "$occupied_json" --backlog-json "$backlog_json" --git-repo "$git_repo" --now 2000000000 --json)"
"$system_python" -c 'import json,sys; d=json.loads(sys.argv[1]); assert d["daemon_count"]==8; assert any(x["name"]=="frozen-consumer" for x in d["bloques"]); assert not any(x["name"]=="healthy-consumer" for x in d["bloques"]); b=d["branch_backlog"]; assert b["state"]=="partial"; assert b["reason"]=="greffe sans etat exploitable"; assert any(x["ref"]=="origin/pending-lot" and x["age_secs"]==7200 for x in b["lots"]); assert not any(x["ref"]=="origin/merged-lot" for x in b["lots"])' "$json_out"
echo 'cli_json_backlog_partiel: OK'

echo "test-bridget-idle: 20 passes / 0 echec / 0 ignore"
