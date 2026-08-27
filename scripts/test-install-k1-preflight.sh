#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
installer="${root_dir}/scripts/install-k1.sh"
fixture_root="$(mktemp -d -t maicie-install-preflight.XXXXXX)"
cleanup() {
  rm -rf "$fixture_root"
}
trap cleanup EXIT

session049_passed=0
session049_failed=0
session049_record() {
  local name="$1"
  shift
  if "$@"; then
    printf 'session-049 %s ... ok\n' "$name"
    session049_passed=$((session049_passed + 1))
  else
    printf 'session-049 %s ... FAILED\n' "$name" >&2
    session049_failed=$((session049_failed + 1))
  fi
}

database_from_config() {
  python3 - "$1" <<'PY'
import json, sys
with open(sys.argv[1], encoding="utf-8") as stream:
    print(json.load(stream)["database_path"])
PY
}

directory_mode() {
  python3 - "$1" <<'PY'
import os, stat, sys
print(oct(stat.S_IMODE(os.stat(sys.argv[1]).st_mode))[2:])
PY
}

printf '%s\n' \
  'univers session-049 (5 scénarios): défaut durable, XDG absolu, XDG relatif, cache préservé averti, état préservé silencieux'

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

default_state_is_durable() {
  local expected="${home}/.local/state/maicie/maicie.sqlite3"
  local actual
  actual="$(database_from_config "${home}/.config/maicie/config.json")"
  [[ "$actual" == "$expected" ]] \
    && [[ "$actual" != "${home}/.cache/"* ]] \
    && [[ -d "${home}/.local/state/maicie" ]] \
    && [[ "$(directory_mode "${home}/.local/state/maicie")" == "700" ]]
}
session049_record default_hors_cache default_state_is_durable

xdg_home="${fixture_root}/xdg-home"
xdg_state="${fixture_root}/xdg-state"
mkdir -p "${xdg_home}/.local/bin" "${xdg_home}/.config/maicie"
cp "${home}/.local/bin/bridget" "${xdg_home}/.local/bin/bridget"
ln -s "$fake_target" "${xdg_home}/.local/bin/maicie"
: >"$command_log"
xdg_output="$(
  HOME="$xdg_home" \
  XDG_STATE_HOME="$xdg_state" \
  K1_TEST_COMMAND_LOG="$command_log" \
  K1_TEST_PREFLIGHT_EXIT=0 \
  "$installer" --skip-services --skip-verify 2>&1
)"
xdg_absolute_is_honoured() {
  [[ "$(database_from_config "${xdg_home}/.config/maicie/config.json")" \
    == "${xdg_state}/maicie/maicie.sqlite3" ]] \
    && [[ -d "${xdg_state}/maicie" ]] \
    && [[ "$(directory_mode "${xdg_state}/maicie")" == "700" ]] \
    && ! grep -Fq 'XDG_STATE_HOME relatif ignoré' <<<"$xdg_output"
}
session049_record xdg_absolu_honore xdg_absolute_is_honoured

relative_home="${fixture_root}/relative-home"
mkdir -p "${relative_home}/.local/bin" "${relative_home}/.config/maicie"
cp "${home}/.local/bin/bridget" "${relative_home}/.local/bin/bridget"
ln -s "$fake_target" "${relative_home}/.local/bin/maicie"
: >"$command_log"
relative_output="$(
  HOME="$relative_home" \
  XDG_STATE_HOME='relative/state' \
  K1_TEST_COMMAND_LOG="$command_log" \
  K1_TEST_PREFLIGHT_EXIT=0 \
  "$installer" --skip-services --skip-verify 2>&1
)"
xdg_relative_falls_back() {
  [[ "$(database_from_config "${relative_home}/.config/maicie/config.json")" \
    == "${relative_home}/.local/state/maicie/maicie.sqlite3" ]] \
    && grep -Fq 'XDG_STATE_HOME relatif ignoré' <<<"$relative_output"
}
session049_record xdg_relatif_replie xdg_relative_falls_back

cache_database="${home}/.cache/bridget/maicie-state/maicie.sqlite3"
cat >"${home}/.config/maicie/config.json" <<EOF
{"version":1,"database_path":"${cache_database}"}
EOF
cp "${home}/.config/maicie/config.json" "${fixture_root}/cache-config.before.json"
: >"$command_log"
cache_output="$(
  HOME="$home" \
  K1_TEST_COMMAND_LOG="$command_log" \
  K1_TEST_PREFLIGHT_EXIT=0 \
  "$installer" --skip-services --skip-verify 2>&1
)"
existing_cache_is_preserved_and_warned() {
  cmp -s "${fixture_root}/cache-config.before.json" \
    "${home}/.config/maicie/config.json" \
    && grep -Fq 'AVERTISSEMENT: base Maicie durable située sous un cache' \
      <<<"$cache_output" \
    && grep -Fq "$cache_database" <<<"$cache_output" \
    && grep -Fq "${home}/.local/state/maicie/maicie.sqlite3" <<<"$cache_output"
}
session049_record cache_explicite_preserve_averti \
  existing_cache_is_preserved_and_warned

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

force_output="$(
  HOME="$force_home" \
  K1_TEST_COMMAND_LOG="$force_log" \
  "${force_repo}/scripts/install-k1.sh" --force --skip-services --skip-verify 2>&1
)"

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

existing_state_is_preserved_without_warning() {
  cmp -s "${force_root}/config.before.json" \
    "${force_home}/.config/maicie/config.json" \
    && ! grep -Fq 'AVERTISSEMENT: base Maicie durable située sous un cache' \
      <<<"$force_output"
}
session049_record etat_explicite_preserve_silencieux \
  existing_state_is_preserved_without_warning

systemctl_state="${force_root}/systemctl.state"
systemctl_log="${force_root}/systemctl.log"
printf '%s\n' \
  'active bridget-daemon.service' \
  'active bridget-maicie-releve.timer' \
  'active bridget-maicie-releve.service' >"$systemctl_state"
cat >"${force_home}/.cargo/bin/systemctl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
state="${K1_TEST_SYSTEMCTL_STATE:?}"
log="${K1_TEST_SYSTEMCTL_LOG:?}"
args=("$@")
[[ "${args[0]:-}" == "--user" ]] && args=("${args[@]:1}")
printf '%s\n' "${args[*]}" >>"$log"
command="${args[0]:-}"
unit="${args[${#args[@]}-1]:-}"
case "$command" in
  is-active)
    grep -Fqx "active $unit" "$state"
    ;;
  stop)
    grep -Fv "active $unit" "$state" >"${state}.tmp"
    mv "${state}.tmp" "$state"
    ;;
  daemon-reload)
    ;;
  restart|start|enable)
    grep -Fv "active $unit" "$state" >"${state}.tmp" || true
    printf 'active %s\n' "$unit" >>"${state}.tmp"
    mv "${state}.tmp" "$state"
    ;;
  *)
    echo "commande systemctl fixture inconnue: ${args[*]}" >&2
    exit 99
    ;;
esac
EOF
chmod 0755 "${force_home}/.cargo/bin/systemctl"
: >"$systemctl_log"
live_output="$(
  HOME="$force_home" \
  K1_TEST_COMMAND_LOG="$force_log" \
  K1_TEST_SYSTEMCTL_STATE="$systemctl_state" \
  K1_TEST_SYSTEMCTL_LOG="$systemctl_log" \
  "${force_repo}/scripts/install-k1.sh" --force --skip-verify 2>&1
)"
grep -q 'relève systemd arrêtée avant publication' <<<"$live_output"
grep -q 'gate Maicie accepté sur la paire publiée' <<<"$live_output"
quiesce_line="$(grep -n 'relève systemd arrêtée avant publication' <<<"$live_output" | cut -d: -f1)"
published_gate_line="$(grep -n 'gate Maicie accepté sur la paire publiée' <<<"$live_output" | cut -d: -f1)"
[[ "$quiesce_line" -lt "$published_gate_line" ]]
grep -Fxq 'stop bridget-maicie-releve.timer' "$systemctl_log"
grep -Fxq 'stop bridget-maicie-releve.service' "$systemctl_log"
grep -Fxq 'enable --now bridget-maicie-releve.timer' "$systemctl_log"
grep -Fxq 'start bridget-maicie-releve.service' "$systemctl_log"

printf 'session-049 result: %s passed / %s failed / 0 ignored\n' \
  "$session049_passed" "$session049_failed"
[[ "$session049_failed" == "0" ]]
echo "gate installation Maicie: refus, paire finale --force et relève gardée vérifiés"
