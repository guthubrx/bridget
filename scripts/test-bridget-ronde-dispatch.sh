#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dispatcher="${root_dir}/scripts/bridget-ronde-dispatch.py"
fixture_root="$(mktemp -d -t bridget-ronde-dispatch.XXXXXX)"
trap 'rm -rf -- "$fixture_root"' EXIT
log="${fixture_root}/commands.log"
fake="${fixture_root}/bridget"

cat >"$fake" <<'FAKE'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"${ROUND_COMMAND_LOG}"
printf '%s\n' '[]'
FAKE
chmod +x "$fake"

ROUND_COMMAND_LOG="$log" "$dispatcher" --bridget-bin "$fake" --now 1000 --json >/dev/null
ROUND_COMMAND_LOG="$log" "$dispatcher" --bridget-bin "$fake" --now 1001 >/dev/null
mapfile -t commands <"$log"
[[ "${#commands[@]}" -eq 2 ]]
[[ "${commands[0]}" == "project-round dispatch --occurrence 840 --json" ]]
[[ "${commands[1]}" == "${commands[0]}" ]]

if ROUND_COMMAND_LOG="$log" "$dispatcher" --bridget-bin "$fake" --now 1000 --unknown >/dev/null 2>&1; then
  echo "la grammaire du dispatcher a accepté une option inconnue" >&2
  exit 1
fi

echo "dispatcher de ronde: occurrence stable, zéro projet et grammaire fermée vérifiés"
