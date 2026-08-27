#!/usr/bin/env bash
# One-shot référent : reclasse nature via `maicie registre requalifier`.
# Défaut = dry-run. --apply écrit réellement. JAMAIS lancé par un modèle.
set -euo pipefail
CONFIG="${MAICIE_CONFIG:?MAICIE_CONFIG absolu obligatoire}"
TABLE="${1:?table jsonl obligatoire}"
BIN="${MAICIE_BIN:-./target/release/maicie}"
APPLY=0
[[ "${2:-}" == "--apply" ]] && APPLY=1
DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
REF="${NATURE_RECLASS_REF:?NATURE_RECLASS_REF=sha:<hex> ou mesure:<N/M> obligatoire}"
n=0
while IFS= read -r line; do
  [[ -z "$line" || "$line" == \#* ]] && continue
  id=$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["id"])' "$line")
  de=$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["nature_de"])' "$line")
  vers=$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["nature_vers"])' "$line")
  raison=$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["raison"])' "$line")
  n=$((n+1))
  cmd=("$BIN" registre requalifier --config "$CONFIG" --constat "$id" \
    --nature-de "$de" --nature-vers "$vers" --raison "$raison" \
    --ref "$REF" --date "$DATE")
  if [[ "$APPLY" -eq 1 ]]; then
    "${cmd[@]}"
  else
    printf 'DRY-RUN %s\n' "${cmd[*]}"
  fi
done < "$TABLE"
echo "lignes=$n apply=$APPLY"
