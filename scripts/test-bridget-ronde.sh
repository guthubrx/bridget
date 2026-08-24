#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ronde="${root_dir}/scripts/bridget-ronde.py"
installer="${root_dir}/scripts/install-bridget-ronde.sh"
fixture_root="$(mktemp -d -t bridget-ronde-test.XXXXXX)"
writer_pid=""
cleanup() {
  [[ -n "$writer_pid" ]] && touch "${fixture_root}/writer-release"
  [[ -n "$writer_pid" ]] && wait "$writer_pid" 2>/dev/null || true
  rm -rf "$fixture_root"
}
trap cleanup EXIT

fake_bridget="${fixture_root}/bridget"
fake_maicie="${fixture_root}/maicie"
cat >"$fake_bridget" <<'EOF'
#!/usr/bin/env bash
printf 'bridget %s\n' "$*" >>"$RONDE_COMMAND_LOG"
case "$*" in
  "agents --json") printf '%s\n' '[{"name":"alice","state":"connected","domain":"bridget","last_seen_secs":12},{"name":"bob","state":"connected","domain":"bridget","last_seen_secs":1901},{"name":"bridget","state":"connected","last_seen_secs":1}]' ;;
  "requests --all --json") printf '%s\n' '[{"id":"expired","sender":"alice","target":"bob","deadline_at":99},{"id":"live","sender":"alice","target":"bob","deadline_at":101}]' ;;
  *) exit 9 ;;
esac
EOF
cat >"$fake_maicie" <<'EOF'
#!/usr/bin/env bash
printf 'maicie %s\n' "$*" >>"$RONDE_COMMAND_LOG"
case "$1" in
  registre) printf 'REGISTRE\nopen=1\n' ;;
  *) exit 9 ;;
esac
EOF
chmod 0755 "$fake_bridget" "$fake_maicie"

config="${fixture_root}/maicie.json"
database="${fixture_root}/maicie.sqlite3"
# Ce processus garde une vraie base WAL ouverte : l'oracle vérifie les
# sidecars de la SOURCE, pas ceux de la copie temporaire de la ronde.
python3 - "$config" "$database" "${fixture_root}/writer-ready" "${fixture_root}/writer-release" <<'PY' &
import json, pathlib, sqlite3, sys, time
config, database, ready, release = map(pathlib.Path, sys.argv[1:])
db = sqlite3.connect(database)
db.execute("PRAGMA journal_mode=WAL")
db.execute("CREATE TABLE objectives(id TEXT PRIMARY KEY, state TEXT NOT NULL)")
db.execute("CREATE TABLE delegations(id TEXT PRIMARY KEY, objective_id TEXT NOT NULL, payload_json TEXT NOT NULL)")
db.execute("INSERT INTO objectives VALUES ('evaluate', 'a_evaluer')")
db.execute("INSERT INTO objectives VALUES ('active', 'en_coordination')")
db.execute("INSERT INTO delegations VALUES ('d-active', 'active', '{\"participant\":\"alice\"}')")
db.commit()
db.execute("SELECT COUNT(*) FROM objectives").fetchone()
assert pathlib.Path(f"{database}-wal").exists()
assert pathlib.Path(f"{database}-shm").exists()
with config.open("w", encoding="utf-8") as stream:
    json.dump({"database_path": str(database)}, stream)
pathlib.Path(ready).touch()
while not pathlib.Path(release).exists():
    time.sleep(0.01)
db.close()
PY
writer_pid=$!
for _ in $(seq 1 100); do [[ -f "${fixture_root}/writer-ready" ]] && break; sleep 0.01; done
[[ -f "${fixture_root}/writer-ready" ]]

sidecars_before="$(python3 - "$database" <<'PY'
import json, os, sys
database = sys.argv[1]
print(json.dumps({suffix: [os.stat(database + suffix).st_size, os.stat(database + suffix).st_mtime_ns] for suffix in ('-wal', '-shm')}))
PY
)"

report_dir="${fixture_root}/reports"
command_log="${fixture_root}/commands.log"
output="$(RONDE_COMMAND_LOG="$command_log" python3 "$ronde" --bridget-bin "$fake_bridget" --maicie-bin "$fake_maicie" --config "$config" --now 100 --report-dir "$report_dir")"
grep -q 'Constat seulement : aucune décision ni aucun envoi.' <<<"$output"
json="$(tail -n 1 <<<"$output")"
python3 - "$json" <<'PY'
import json, sys
report = json.loads(sys.argv[1])
assert report["decision"] == "none" and report["delivery"] == "none"
assert report["maicie"]["objectives_to_evaluate"] == [{"objective_id": "evaluate", "state": "a_evaluer"}]
assert [a["name"] for a in report["agents"]["unassigned_connected"]] == ["bob"]
assert [r["id"] for r in report["requests"]["expired"]] == ["expired"]
assert report["registry"]["view"] == "REGISTRE\nopen=1\n"
PY
[[ -f "${report_dir}/ronde-1970-01-01T00-01-40Z.txt" ]]
[[ -f "${report_dir}/ronde-1970-01-01T00-01-40Z.json" ]]
sidecars_after="$(python3 - "$database" <<'PY'
import json, os, sys
database = sys.argv[1]
print(json.dumps({suffix: [os.stat(database + suffix).st_size, os.stat(database + suffix).st_mtime_ns] for suffix in ('-wal', '-shm')}))
PY
)"
[[ "$sidecars_before" == "$sidecars_after" ]] || {
  echo "la ronde a modifié les sidecars de la source SQLite" >&2
  exit 1
}
grep -qx 'bridget agents --json' "$command_log"
grep -qx 'bridget requests --all --json' "$command_log"
grep -qx 'maicie registre list --config .*/maicie.json --attente' "$command_log"
assert_passive_log() {
  if rg -q '(send|spawn|stop|approve|status)' "$1"; then
    return 1
  fi
}
if ! assert_passive_log "$command_log"; then
  echo "la ronde ne doit ni agir ni appeler maicie status" >&2
  exit 1
fi
# Mutation d'oracle : l'action n'est jamais exécutée ; injecter son nom dans
# le journal DOIT néanmoins faire rougir le détecteur de passivité.
printf 'bridget send --to alice interdit\n' >>"$command_log"
if assert_passive_log "$command_log"; then
  echo "mutation de passivité non détectée" >&2
  exit 1
fi

degraded="$(RONDE_COMMAND_LOG="$command_log" python3 "$ronde" --json --bridget-bin "$fake_bridget" --maicie-bin "$fake_maicie" --config "${fixture_root}/missing.json" --now 100)"
python3 - "$degraded" <<'PY'
import json, sys
report = json.loads(sys.argv[1])
assert report["maicie"]["state"] == "unavailable"
assert report["agents"]["state"] == "available"
assert report["agents"]["unassigned_connected"] is None
assert report["decision"] == "none" and report["delivery"] == "none"
PY

home="${fixture_root}/home"
HOME="$home" "$installer" --config /tmp/maicie.json --report-dir "${fixture_root}/archive" --skip-activate
[[ -x "${home}/.local/bin/bridget-ronde" ]]
unit="${home}/Library/LaunchAgents/com.bridget.ronde.plist"
[[ -f "$unit" ]] || unit="${home}/.config/systemd/user/bridget-ronde.service"
[[ -f "$unit" ]]
if rg -q '(send|spawn|stop|approve)' "$unit"; then
  echo "une unité de ronde ne peut embarquer aucune action" >&2
  exit 1
fi
echo "ronde portable : rapport normal, dégradation Maicie et unité passive vérifiés"
