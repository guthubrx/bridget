#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fixture_root="$(mktemp -d -t bridget-k1-macos-releve.XXXXXX)"
trap 'rm -rf -- "$fixture_root"' EXIT

repo="${fixture_root}/repo"
home="${fixture_root}/home"
log="${fixture_root}/commands.log"
mkdir -p \
  "${repo}/scripts" \
  "${repo}/target/release" \
  "${home}/.cargo/bin" \
  "${home}/.config/maicie"

cp "${root_dir}/scripts/install-k1.sh" "${repo}/scripts/install-k1.sh"
chmod 0755 "${repo}/scripts/install-k1.sh"

for tool in cargo rustc; do
  cat >"${home}/.cargo/bin/${tool}" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
  chmod 0755 "${home}/.cargo/bin/${tool}"
done

cat >"${repo}/target/release/maicie" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'maicie %s\n' "$*" >>"${K1_TEST_COMMAND_LOG:?}"
if [[ "${1:-}" == "preflight" ]]; then
  printf '%s\n' '{"kind":"schema_preflight","state":"compatible","database_schema":16,"binary_schema":16,"write_schema_compatible":true,"bootstrap_required":false}'
fi
EOF
cat >"${repo}/target/release/bridget" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'bridget %s\n' "$*" >>"${K1_TEST_COMMAND_LOG:?}"
EOF
chmod 0755 "${repo}/target/release/maicie" "${repo}/target/release/bridget"

printf '{"version":1,"database_path":"%s"}\n' "${fixture_root}/maicie.sqlite3" \
  >"${home}/.config/maicie/config.json"

HOME="$home" \
K1_TEST_COMMAND_LOG="$log" \
"${repo}/scripts/install-k1.sh" --force --skip-services --skip-verify >/dev/null

releve="${home}/.local/bin/maicie-releve"
plist="${home}/Library/LaunchAgents/com.bridget.maicie.releve.plist"
[[ -x "$releve" ]]
grep -Fxq "\"${home}/.local/bin/maicie-suivi\"" "$releve"
grep -Fxq "exec \"${home}/.local/bin/bridget\" project-round dispatch" "$releve"
grep -Fq "<string>${releve}</string>" "$plist"
plutil -lint "$plist" >/dev/null

: >"$log"
K1_TEST_COMMAND_LOG="$log" "$releve" >/dev/null
grep -Fqx "maicie preflight --config ${home}/.config/maicie/config.json --json" "$log"
grep -Fqx "maicie status --config ${home}/.config/maicie/config.json --json" "$log"
grep -Fqx 'bridget project-round dispatch' "$log"

echo "relève macOS: Maicie puis dispatcher global des rondes vérifiés"
