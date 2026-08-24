#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ronde="${root_dir}/scripts/bridget-ronde.py"
installer="${root_dir}/scripts/install-bridget-ronde.sh"
fixture_root="$(mktemp -d -t bridget-ronde-test.XXXXXX)"
cleanup() { rm -rf "$fixture_root"; }
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
python3 - "$config" "$database" <<'PY'
import json, sqlite3, sys
config, database = sys.argv[1:]
db = sqlite3.connect(database)
db.execute("CREATE TABLE objectives(id TEXT PRIMARY KEY, state TEXT NOT NULL)")
db.execute("CREATE TABLE delegations(id TEXT PRIMARY KEY, objective_id TEXT NOT NULL, payload_json TEXT NOT NULL)")
db.execute("INSERT INTO objectives VALUES ('evaluate', 'a_evaluer')")
db.execute("INSERT INTO objectives VALUES ('active', 'en_coordination')")
db.execute("INSERT INTO delegations VALUES ('d-active', 'active', '{\"participant\":\"alice\"}')")
db.commit()
db.close()
with open(config, "w", encoding="utf-8") as stream:
    json.dump({"database_path": database}, stream)
PY

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
grep -qx 'bridget agents --json' "$command_log"
grep -qx 'bridget requests --all --json' "$command_log"
grep -qx 'maicie registre list --config .*/maicie.json --attente' "$command_log"
if rg -q '(send|spawn|stop|approve|status)' "$command_log"; then
  echo "la ronde ne doit ni agir ni appeler maicie status" >&2
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
