#!/usr/bin/env bash
# Harnais bridget-idle : partition + bornes de tour + backlog Git local + contrôles positifs.
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
    {"name": "bob", "state": "connected", "domain": "bridget", "transport": "codex_app_server", "last_seen_secs": 5},
    {"name": "cursor10-like", "state": "connected", "domain": "cursor10-lot", "last_seen_secs": 3},
    {"name": "cursorbridget-like", "state": "busy", "domain": "bridget", "last_seen_secs": 1},
    {"name": "relec6-like", "state": "busy", "domain": "relec6-lot", "last_seen_secs": 2},
    {"name": "bridget", "state": "connected", "domain": "bridget", "last_seen_secs": 0},
    # La mission ne suffit pas : la borne du dernier tour tranche l'activité.
    {"name": "healthy-consumer", "state": "connected", "domain": "bridget", "transport": "codex_app_server", "last_seen_secs": 2},
    {"name": "frozen-consumer", "state": "connected", "domain": "bridget", "transport": "codex_app_server", "last_seen_secs": 2},
    {"name": "freshly-finished", "state": "connected", "domain": "bridget", "transport": "acp", "last_seen_secs": 2},
    {"name": "resumed-consumer", "state": "connected", "domain": "bridget", "transport": "codex_app_server", "last_seen_secs": 2},
    {"name": "corrupt-journal", "state": "connected", "domain": "bridget", "transport": "codex_app_server", "last_seen_secs": 2},
    {"name": "missing-journal", "state": "connected", "domain": "bridget", "transport": "codex_app_server", "last_seen_secs": 2},
    {"name": "busy-without-journal", "state": "busy", "domain": "bridget", "transport": "ssh-unix", "last_seen_secs": 2},
    {"name": "contradictory-consumer", "state": "busy", "domain": "bridget", "transport": "codex_app_server", "last_seen_secs": 2},
    {"name": "ssh-open-only", "state": "connected", "domain": "bridget", "transport": "ssh-unix", "last_seen_secs": 2},
    {"name": "acp-anomaly-live", "state": "connected", "domain": "bridget", "transport": "acp", "last_seen_secs": 2},
    {"name": "ambiguous-error", "state": "connected", "domain": "bridget", "transport": "acp", "last_seen_secs": 2},
    {"name": "timeout-sans-kind", "state": "connected", "domain": "bridget", "transport": "codex_app_server", "last_seen_secs": 2},
    {"name": "unsafe-stop", "state": "connected", "domain": "bridget", "transport": "acp", "last_seen_secs": 2},
    {"name": "unsafe-error", "state": "connected", "domain": "bridget", "transport": "acp", "last_seen_secs": 2},
]
occupied = {
    "bob",
    "healthy-consumer",
    "frozen-consumer",
    "freshly-finished",
    "resumed-consumer",
    "corrupt-journal",
    "missing-journal",
    "busy-without-journal",
    "contradictory-consumer",
    "ssh-open-only",
    "acp-anomaly-live",
    "ambiguous-error",
    "timeout-sans-kind",
    "unsafe-stop",
    "unsafe-error",
}
exclude = {"bridget", "fable", "poucave", "sol", "maicie"}
daemon = {a["name"] for a in agents}

journal_root = fixture / "journals"

def write_journal(agent, events, *, invalid_line=False):
    directory = journal_root / agent
    directory.mkdir(parents=True)
    path = directory / "2026-08-25.jsonl"
    with path.open("w", encoding="utf-8") as stream:
        for event in events:
            stream.write(json.dumps(event, ensure_ascii=False) + "\n")
        if invalid_line:
            stream.write('{"v":1,"seq":999,"event":"turn_start"\n')

def boundary(seq, event, message_id, payload=None):
    return {
        "v": 1,
        "seq": seq,
        "ts": f"2026-08-25T10:00:{seq:02d}Z",
        "session_id": "session-fixture",
        "event": event,
        "message_id": message_id,
        "payload": payload or {},
    }

write_journal("bob", [boundary(1, "turn_start", "bob-tour")])
write_journal(
    "healthy-consumer",
    [
        boundary(1, "turn_start", "healthy-tour"),
        boundary(2, "provider_request", "healthy-tour", {"state": "pending"}),
    ],
)
write_journal(
    "frozen-consumer",
    [
        boundary(1, "turn_start", "frozen-tour"),
        boundary(
            2,
            "error",
            "frozen-tour",
            {"reason": "échéance fournisseur dépassée", "terminal_kind": "turn_failed"},
        ),
    ],
)
write_journal(
    "freshly-finished",
    [
        boundary(1, "turn_start", "fresh-tour"),
        boundary(2, "turn_end", "fresh-tour", {"stop_reason": "inconnu"}),
    ],
)
write_journal(
    "resumed-consumer",
    [
        boundary(1, "turn_start", "old-tour"),
        boundary(2, "turn_end", "old-tour", {"stop_reason": "end_turn"}),
        boundary(3, "turn_start", "new-tour"),
        # Un terminal tardif de l'ancien message ne ferme pas le nouveau tour.
        boundary(4, "error", "old-tour", {"reason": "retard ancien"}),
    ],
)
write_journal(
    "corrupt-journal",
    [boundary(1, "turn_start", "corrupt-tour")],
    invalid_line=True,
)
write_journal(
    "contradictory-consumer",
    [
        boundary(1, "turn_start", "contradictory-tour"),
        boundary(2, "turn_end", "contradictory-tour", {"stop_reason": "end_turn"}),
    ],
)
write_journal("ssh-open-only", [boundary(1, "turn_start", "ssh-tour")])
write_journal(
    "acp-anomaly-live",
    [
        boundary(1, "turn_start", "message-live"),
        boundary(2, "error", "message-live", {"reason": "notification ACP inconnue: vendor/future"}),
        boundary(3, "update", "message-live", {"kind": "text", "content": "le tour continue"}),
    ],
)
write_journal(
    "ambiguous-error",
    [
        boundary(1, "turn_start", "message-ambiguous"),
        boundary(2, "error", "message-ambiguous", {"reason": "ancien terminal ou anomalie"}),
    ],
)
# Tour tué par échéance SANS terminal_kind (journaux pré-bornes / binaire non
# relancé) : doit quand même sortir des OCCUPES — sinon la ronde ment.
write_journal(
    "timeout-sans-kind",
    [
        boundary(1, "turn_start", "timeout-tour"),
        boundary(
            2,
            "error",
            "timeout-tour",
            {"reason": "échéance Codex dépassée"},
        ),
    ],
)
unsafe_detail = "provider\x1b[2J\refface\u202ele diagnostic"
if not all(control in unsafe_detail for control in ("\x1b", "\r", "\u202e")):
    raise SystemExit("CONTROLE POSITIF RATE: les contrôles dangereux manquent à la fixture")
write_journal(
    "unsafe-stop",
    [
        boundary(1, "turn_start", "unsafe-stop-tour"),
        boundary(2, "turn_end", "unsafe-stop-tour", {"stop_reason": unsafe_detail}),
    ],
)
write_journal(
    "unsafe-error",
    [
        boundary(1, "turn_start", "unsafe-error-tour"),
        boundary(
            2,
            "error",
            "unsafe-error-tour",
            {"reason": unsafe_detail, "terminal_kind": "turn_failed"},
        ),
    ],
)
for unsafe_agent, detail_key in (("unsafe-stop", "stop_reason"), ("unsafe-error", "reason")):
    path = journal_root / unsafe_agent / "2026-08-25.jsonl"
    terminal = json.loads(path.read_text(encoding="utf-8").splitlines()[-1])
    if terminal["payload"][detail_key] != unsafe_detail:
        raise SystemExit(f"CONTROLE POSITIF RATE: détail dangereux absent de {unsafe_agent}")
print("controle_positif_details_dangereux_presents: OK")

turns = mod.read_turn_observations(journal_root, occupied)
if turns["healthy-consumer"]["state"] != "open":
    raise SystemExit(f"tour sain non ouvert: {turns['healthy-consumer']}")
if turns["frozen-consumer"]["state"] != "ended":
    raise SystemExit(f"fin sans reprise non détectée: {turns['frozen-consumer']}")
if turns["resumed-consumer"]["state"] != "open":
    raise SystemExit(f"reprise non prioritaire: {turns['resumed-consumer']}")
if turns["acp-anomaly-live"]["state"] != "open":
    raise SystemExit(f"une error ACP non terminale a fermé le tour: {turns['acp-anomaly-live']}")
if turns["ambiguous-error"]["state"] != "unknown" or turns["ambiguous-error"].get("reason") != "error-terminalite-non-attestee":
    raise SystemExit(f"une ancienne error ambiguë a produit une certitude: {turns['ambiguous-error']}")
if turns["timeout-sans-kind"]["state"] != "ended":
    raise SystemExit(
        f"échéance sans terminal_kind doit quand même fermer le tour: {turns['timeout-sans-kind']}"
    )
for unsafe_agent, terminal_kind in (("unsafe-stop", "turn_completed"), ("unsafe-error", "turn_failed")):
    if turns[unsafe_agent].get("state") != "ended" or turns[unsafe_agent].get("terminal_kind") != terminal_kind:
        raise SystemExit(f"code terminal fermé absent pour {unsafe_agent}: {turns[unsafe_agent]}")
if turns["corrupt-journal"]["state"] != "unknown" or not turns["corrupt-journal"]["reason"].startswith("journal-json-invalide:"):
    raise SystemExit(f"journal corrompu conclu à tort: {turns['corrupt-journal']}")
if turns["missing-journal"]["state"] != "unknown" or "absent" not in turns["missing-journal"]["reason"]:
    raise SystemExit(f"journal absent conclu à tort: {turns['missing-journal']}")
print("lecture_bornes_tour_et_incertitudes: OK")

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

# Le backlog ne décide plus : ouvert >1 h reste sain, terminal frais reste fini.
backlog = {
    "healthy-consumer": 65 * 60,
    "frozen-consumer": 44 * 60,
    "freshly-finished": 1,
    "resumed-consumer": 65 * 60,
    "corrupt-journal": 44 * 60,
    "missing-journal": 44 * 60,
    "busy-without-journal": 44 * 60,
    "contradictory-consumer": 44 * 60,
    "ssh-open-only": 65 * 60,
    "acp-anomaly-live": 65 * 60,
    "ambiguous-error": 44 * 60,
    "timeout-sans-kind": 44 * 60,
    "unsafe-stop": 44 * 60,
    "unsafe-error": 44 * 60,
}
fixed = mod.classify(
    agents,
    occupied,
    exclude=exclude,
    silent_after_secs=1800,
    backlog_ages=backlog,
    turn_observations=turns,
)
ok_fixed, detail_fixed = mod.partition_oracle(fixed, daemon)
if not ok_fixed:
    raise SystemExit(f"correctif ROUGE sur oracle: {detail_fixed}")
print(f"partition_correctif: OK ({detail_fixed})")

bloques = {item["name"]: item for item in fixed["bloques"]}
if "frozen-consumer" not in bloques:
    raise SystemExit(f"figé absente de BLOQUES: {bloques}")
if "timeout-sans-kind" not in bloques:
    raise SystemExit(f"échéance sans kind absente de BLOQUES: {bloques}")
if "timeout-sans-kind" in fixed["occupes"]:
    raise SystemExit("échéance sans kind ne doit pas rester OCCUPE")
if "healthy-consumer" in bloques:
    raise SystemExit(f"consommateur sain à tort BLOQUE: {bloques}")
if "freshly-finished" not in bloques:
    raise SystemExit(f"terminal frais masqué par l'âge de remise: {bloques}")
if "healthy-consumer" not in fixed["occupes"]:
    raise SystemExit(f"sain devrait rester OCCUPE: {fixed['occupes']}")
if "frozen-consumer" in fixed["occupes"]:
    raise SystemExit("figé ne doit pas rester OCCUPE une fois BLOQUE")
if "resumed-consumer" not in fixed["occupes"]:
    raise SystemExit(f"tour repris devrait rester OCCUPE: {fixed['occupes']}")
if "busy-without-journal" not in fixed["occupes"]:
    raise SystemExit(f"état busy positif perdu: {fixed['occupes']}")
if "acp-anomaly-live" not in fixed["occupes"]:
    raise SystemExit(f"anomalie ACP non terminale sortie des OCCUPES: {fixed['occupes']}")
ind_fixed = {name: reason for name, reason in fixed["indetermines"]}
for uncertain in ("corrupt-journal", "missing-journal", "ambiguous-error"):
    if uncertain not in ind_fixed or "activite-tour=" not in ind_fixed[uncertain]:
        raise SystemExit(f"incertitude de journal non nommée pour {uncertain}: {ind_fixed}")
if "contradictory-consumer" not in ind_fixed or "contradiction-state-busy" not in ind_fixed["contradictory-consumer"]:
    raise SystemExit(f"contradiction de sources conclue à tort: {ind_fixed}")
if "ssh-open-only" not in ind_fixed or "source-sans-borne-terminale:ssh-unix" not in ind_fixed["ssh-open-only"]:
    raise SystemExit(f"source interactive incomplète conclue à tort: {ind_fixed}")
if bloques["frozen-consumer"]["condition"] != "dernier-tour-termine-sans-reprise":
    raise SystemExit(f"condition terminale absente: {bloques['frozen-consumer']}")
for unsafe_agent in ("unsafe-stop", "unsafe-error"):
    if unsafe_agent not in bloques:
        raise SystemExit(f"univers terminal incomplet, {unsafe_agent} absent: {bloques}")
print("controle_positif_bloques_deux_sens: OK")

# Sans backlog, la borne terminale reste concluante : l'âge n'est qu'un contexte.
no_backlog = mod.classify(
    agents,
    occupied,
    exclude=exclude,
    silent_after_secs=1800,
    backlog_ages={},
    turn_observations=turns,
)
if "frozen-consumer" not in {item["name"] for item in no_backlog["bloques"]}:
    raise SystemExit("sans backlog, la fin sans reprise doit rester visible")
print("borne_terminale_independante_du_backlog: OK")

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

# Le contrôle positif porte sur la présence d'un tour ouvert, pas un seuil.
if fixed["occupes"].count("healthy-consumer") != 1:
    raise SystemExit(f"tour ouvert long non conservé exactement une fois: {fixed['occupes']}")
print("tour_ouvert_65min_reste_occupe: OK")

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
print(f"FIXTURES {agents_path} {occupied_path} {backlog_path} {journal_root}")
PY

agents_json="${fixture_root}/agents.json"
occupied_json="${fixture_root}/occupied.json"
backlog_json="${fixture_root}/backlog.json"
journal_root="${fixture_root}/journals"
git_repo="${fixture_root}/repository"
out="$("$system_python" "$idle" --agents-json "$agents_json" --occupied-json "$occupied_json" --backlog-json "$backlog_json" --journal-root "$journal_root" --git-repo "$git_repo" --now 2000000000)"
grep -q 'BLOQUES' <<<"$out"
grep -q 'frozen-consumer' <<<"$out"
grep -q 'dernier-tour-termine-sans-reprise' <<<"$out"
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
"$system_python" - "$out" <<'PY'
import sys

rendered = sys.argv[1]
for control in ("\x1b", "\r", "\u202e"):
    if control in rendered:
        raise SystemExit(f"contrôle terminal brut dans stdout: U+{ord(control):04X}")
for escaped in (r"\u001b", r"\u000d", r"\u202e"):
    if escaped not in rendered:
        raise SystemExit(f"détail neutralisé absent du rendu texte: {escaped}")
print("rendu_texte_details_inertes: OK")
PY
slow_out="$("$system_python" "$idle" --agents-json "$agents_json" --occupied-json "$occupied_json" --backlog-json "$backlog_json" --journal-root "$journal_root" --git-repo "$git_repo" --git-bin "${fixture_root}/git-slow" --git-timeout-secs 0.05 --now 2000000000)"
grep -q 'BACKLOG BRANCHES INDISPONIBLE (delai Git depasse)' <<<"$slow_out"
if grep -q 'origin/pending-lot' <<<"$slow_out"; then
  echo 'backlog_branches_timeout_cli: liste partielle visible' >&2
  exit 1
fi
echo 'cli_timeout_sans_liste_partielle: OK'
json_out="$("$system_python" "$idle" --agents-json "$agents_json" --occupied-json "$occupied_json" --backlog-json "$backlog_json" --journal-root "$journal_root" --git-repo "$git_repo" --now 2000000000 --json)"
"$system_python" - "$json_out" <<'PY'
import json, sys

raw = sys.argv[1]
data = json.loads(raw)
assert data["daemon_count"] == 20
assert any(item["name"] == "frozen-consumer" and item["condition"] == "dernier-tour-termine-sans-reprise" for item in data["bloques"])
assert any(item["name"] == "timeout-sans-kind" and item["condition"] == "dernier-tour-termine-sans-reprise" for item in data["bloques"])
assert "timeout-sans-kind" not in data["occupes"]
assert not any(item["name"] == "healthy-consumer" for item in data["bloques"])
assert "healthy-consumer" in data["occupes"]
assert "acp-anomaly-live" in data["occupes"]
assert any(item["name"] == "missing-journal" and "journal-absent" in item["reason"] for item in data["indetermines"])
assert any(item["name"] == "ssh-open-only" and "source-sans-borne-terminale:ssh-unix" in item["reason"] for item in data["indetermines"])
assert any(item["name"] == "ambiguous-error" and "error-terminalite-non-attestee" in item["reason"] for item in data["indetermines"])
unsafe_detail = "provider\x1b[2J\refface\u202ele diagnostic"
unsafe = {item["name"]: item for item in data["bloques"] if item["name"].startswith("unsafe-")}
assert unsafe["unsafe-stop"]["terminal_kind"] == "turn_completed"
assert unsafe["unsafe-error"]["terminal_kind"] == "turn_failed"
assert unsafe["unsafe-stop"]["terminal_reason"] == unsafe_detail
assert unsafe["unsafe-error"]["terminal_reason"] == unsafe_detail
for control in ("\x1b", "\r", "\u202e"):
    if control in raw:
        raise SystemExit(f"contrôle brut dans le JSON sérialisé: U+{ord(control):04X}")
b = data["branch_backlog"]
assert b["state"] == "partial"
assert b["reason"] == "greffe sans etat exploitable"
assert any(x["ref"] == "origin/pending-lot" and x["age_secs"] == 7200 for x in b["lots"])
assert not any(x["ref"] == "origin/merged-lot" for x in b["lots"])
print("json_diagnostic_valide_et_inerte_et_backlog_branches: OK")
PY

echo "test-bridget-idle: checks OK (partition + bornes de tour + backlog Git + incertitudes + copies + cli)"
