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
grep -q 'gate Maicie refusé avant installation' <<<"$refusal"
grep -q 'schéma incompatible (fixture)' <<<"$refusal"
grep -qx "preflight --config ${home}/.config/maicie/config.json --json" "$command_log"
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
grep -q 'gate Maicie accepté avant installation' <<<"$accepted"
grep -q "cible=${fake_target}" <<<"$accepted"
grep -qx "preflight --config ${home}/.config/maicie/config.json --json" "$command_log"
[[ -L "${home}/.local/bin/maicie" ]]

rm -f "${home}/.config/maicie/config.json"
mkdir -p "${home}/.cache/bridget/maicie-state"
: >"${home}/.cache/bridget/maicie-state/maicie.sqlite3"
: >"$command_log"
set +e
missing_config="$(
  HOME="$home" \
  K1_TEST_COMMAND_LOG="$command_log" \
  K1_TEST_PREFLIGHT_EXIT=0 \
  "$installer" --skip-services --skip-verify 2>&1
)"
missing_config_code=$?
set -e
[[ "$missing_config_code" == "1" ]]
grep -q 'config absente mais greffe par défaut présent' <<<"$missing_config"
[[ ! -s "$command_log" ]]

echo "gate installation Maicie: refus non contournable, cible de symlink et config absente vérifiés"
