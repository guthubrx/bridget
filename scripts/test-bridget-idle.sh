#!/usr/bin/env bash
# Harnais bridget-idle : oracle de partition + contrôles positifs (omission + bloqués).
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
idle="${root_dir}/scripts/bridget-idle.py"
system_python="/usr/bin/python3"
fixture_root="$(mktemp -d -t bridget-idle-test.XXXXXX)"
cleanup() { rm -rf "$fixture_root"; }
trap cleanup EXIT

[[ -f "$idle" ]] || { echo "absent: $idle" >&2; exit 1; }

"$system_python" - "$idle" "$fixture_root" <<'PY'
import importlib.util, json, pathlib, sys, os, sqlite3

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
out="$("$system_python" "$idle" --agents-json "$agents_json" --occupied-json "$occupied_json" --backlog-json "$backlog_json")"
grep -q 'BLOQUES' <<<"$out"
grep -q 'frozen-consumer' <<<"$out"
grep -q 'cursor10-like' <<<"$out"
json_out="$("$system_python" "$idle" --agents-json "$agents_json" --occupied-json "$occupied_json" --backlog-json "$backlog_json" --json)"
"$system_python" -c 'import json,sys; d=json.loads(sys.argv[1]); assert d["daemon_count"]==8; assert any(x["name"]=="frozen-consumer" for x in d["bloques"]); assert not any(x["name"]=="healthy-consumer" for x in d["bloques"])' "$json_out"

echo "test-bridget-idle: checks OK (omission + partition + bloques 2 sens + seuil + copies + cli)"
