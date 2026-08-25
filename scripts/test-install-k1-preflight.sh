#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
installer="${root_dir}/scripts/install-k1.sh"
fixture_root="$(mktemp -d -t maicie-install-preflight.XXXXXX)"
cleanup() {
  rm -rf "$fixture_root"
}
trap cleanup EXIT

home="${fixture_root}/home"
fake_target="${fixture_root}/maicie-candidate"
command_log="${fixture_root}/commands.log"
mkdir -p "${home}/.local/bin" "${home}/.config/maicie"

cat >"${home}/.local/bin/bridget" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
cat >"$fake_target" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >>"$K1_TEST_COMMAND_LOG"
if [[ "$1" == "preflight" ]]; then
  if [[ "$K1_TEST_PREFLIGHT_EXIT" == "0" ]]; then
    printf '%s\n' '{"kind":"schema_preflight","state":"compatible","database_schema":16,"binary_schema":16,"write_schema_compatible":true,"bootstrap_required":false}'
    exit 0
  fi
  printf '%s\n' '{"error":{"code":"store","message":"schéma incompatible (fixture)"}}' >&2
  exit "$K1_TEST_PREFLIGHT_EXIT"
fi
exit 99
EOF
chmod 0755 "${home}/.local/bin/bridget" "$fake_target"
ln -s "$fake_target" "${home}/.local/bin/maicie"
cat >"${home}/.config/maicie/config.json" <<EOF
{"version":1,"database_path":"${fixture_root}/maicie.sqlite3"}
EOF

set +e
refusal="$(
  HOME="$home" \
  K1_TEST_COMMAND_LOG="$command_log" \
  K1_TEST_PREFLIGHT_EXIT=6 \
  "$installer" --skip-services --skip-verify 2>&1
)"
refusal_code=$?
set -e
[[ "$refusal_code" == "1" ]]
grep -q 'gate Maicie refusé avant publication' <<<"$refusal"
grep -q 'schéma incompatible (fixture)' <<<"$refusal"
grep -Eq '^preflight --config .*/maicie-k1-stage\.[^/]+/config.json --json$' "$command_log"
[[ -L "${home}/.local/bin/maicie" ]]
[[ "$(readlink "${home}/.local/bin/maicie")" == "$fake_target" ]]
[[ ! -e "${home}/.config/bridget/agents.json" ]]

: >"$command_log"
accepted="$(
  HOME="$home" \
  K1_TEST_COMMAND_LOG="$command_log" \
  K1_TEST_PREFLIGHT_EXIT=0 \
  "$installer" --skip-services --skip-verify 2>&1
)"
grep -q 'gate Maicie accepté sur la paire stagée' <<<"$accepted"
grep -q "cible=${fake_target}" <<<"$accepted"
[[ "$(wc -l <"$command_log")" == "2" ]]
grep -Eq '^preflight --config .*/maicie-k1-stage\.[^/]+/config.json --json$' "$command_log"
grep -qx "preflight --config ${home}/.config/maicie/config.json --json" "$command_log"
[[ -L "${home}/.local/bin/maicie" ]]

rm -f "${home}/.config/maicie/config.json"
mkdir -p "${home}/.cache/bridget/maicie-state"
: >"${home}/.cache/bridget/maicie-state/maicie.sqlite3"
: >"$command_log"
missing_config="$(
  HOME="$home" \
  K1_TEST_COMMAND_LOG="$command_log" \
  K1_TEST_PREFLIGHT_EXIT=0 \
  "$installer" --skip-services --skip-verify 2>&1
)"
grep -q 'gate Maicie accepté sur la paire publiée' <<<"$missing_config"
[[ -f "${home}/.config/maicie/config.json" ]]
[[ "$(wc -l <"$command_log")" == "2" ]]

force_root="${fixture_root}/force"
force_repo="${force_root}/repo"
force_home="${force_root}/home"
force_log="${force_root}/commands.log"
mkdir -p \
  "${force_repo}/scripts" \
  "${force_repo}/target/release" \
  "${force_home}/.cargo/bin" \
  "${force_home}/.config/maicie" \
  "${force_home}/.cache/bridget/maicie-state"
cp "$installer" "${force_repo}/scripts/install-k1.sh"
chmod 0755 "${force_repo}/scripts/install-k1.sh"
cat >"${force_home}/.cargo/bin/cargo" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
cat >"${force_home}/.cargo/bin/rustc" <<'EOF'
#!/usr/bin/env bash
echo 'rustc fixture'
EOF
cat >"${force_repo}/target/release/bridget" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
cat >"${force_repo}/target/release/maicie" <<'EOF'
#!/usr/bin/env bash
config=""
for ((index = 1; index <= $#; index++)); do
  if [[ "${!index}" == "--config" ]]; then
    next=$((index + 1))
    config="${!next}"
  fi
done
database="$(python3 - "$config" <<'PY'
import json, sys
with open(sys.argv[1], encoding="utf-8") as stream:
    print(json.load(stream)["database_path"])
PY
)"
printf '%s|%s\n' "$config" "$database" >>"$K1_TEST_COMMAND_LOG"
version="$(<"$database")"
if [[ "$1" == "preflight" && "$version" == "16" ]]; then
  printf '%s\n' '{"kind":"schema_preflight","state":"compatible","database_schema":16,"binary_schema":16,"write_schema_compatible":true,"bootstrap_required":false}'
  exit 0
fi
printf '%s\n' "{\"error\":{\"code\":\"store\",\"message\":\"schéma SQLite ${version} non supporté (maximum 16)\"}}" >&2
exit 6
EOF
chmod 0755 \
  "${force_home}/.cargo/bin/cargo" \
  "${force_home}/.cargo/bin/rustc" \
  "${force_repo}/target/release/bridget" \
  "${force_repo}/target/release/maicie"
compatible_database="${force_root}/compatible.sqlite3"
default_database="${force_home}/.cache/bridget/maicie-state/maicie.sqlite3"
printf '16' >"$compatible_database"
printf '17' >"$default_database"
cat >"${force_home}/.config/maicie/config.json" <<EOF
{"version":1,"database_path":"${compatible_database}"}
EOF
cp "${force_home}/.config/maicie/config.json" "${force_root}/config.before.json"

HOME="$force_home" \
K1_TEST_COMMAND_LOG="$force_log" \
"${force_repo}/scripts/install-k1.sh" --force --skip-services --skip-verify >/dev/null

if ! cmp -s "${force_root}/config.before.json" "${force_home}/.config/maicie/config.json"; then
  echo "--force a remplacé la configuration préflightée avant activation" >&2
  exit 1
fi
cmp -s "${force_repo}/target/release/maicie" "${force_home}/.local/bin/maicie"
HOME="$force_home" \
K1_TEST_COMMAND_LOG="$force_log" \
"${force_home}/.local/bin/maicie" preflight \
  --config "${force_home}/.config/maicie/config.json" --json >/dev/null
if grep -Fq "|${default_database}" "$force_log"; then
  echo "le gate a contrôlé la base par défaut 17 au lieu de préserver la paire finale" >&2
  exit 1
fi
grep -Fq "\"${force_home}/.local/bin/maicie\" preflight" \
  "${force_home}/.local/bin/maicie-suivi"
grep -Fxq "ExecStart=${force_home}/.local/bin/maicie-suivi" \
  "${force_home}/.config/systemd/user/bridget-maicie-releve.service"

echo "gate installation Maicie: refus, paire finale --force et relève gardée vérifiés"
