#!/usr/bin/env bash
set -euo pipefail

root=$(mktemp -d)
trap 'find "$root" -depth -delete' EXIT
mkdir -p "$root/bin" "$root/config" "$root/table"
log="$root/calls"
cat >"$root/bin/maicie" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >>"$MAICIE_CALL_LOG"
exit 0
EOF
chmod +x "$root/bin/maicie"
cat >"$root/catalogue.jsonl" <<'EOF'
{"kind":"add","id":"c-script","nature":"resultat","severity":"minor"}
EOF
cat >"$root/table/reclassement.jsonl" <<'EOF'
{"id":"c-script","nature_de":"resultat","nature_vers":"regle","severity_de":"minor","severity_vers":"major","raison":"test"}
EOF
printf '{"catalogue_path":"%s"}\n' "$root/catalogue.jsonl" >"$root/config.json"
MAICIE_CONFIG="$root/config.json" NATURE_RECLASS_REF='sha:test' MAICIE_BIN="$root/bin/maicie" \
  MAICIE_CALL_LOG="$log" PATH="$root/bin:$PATH" \
  scripts/registre-nature-reclasser.sh "$root/table/reclassement.jsonl" --apply >"$root/out"
grep -F -- '--nature-de resultat --nature-vers regle' "$log" \
  >/dev/null || { echo 'temoin_script_transmet_la_nature: arguments absents'; exit 1; }
grep -F -- '--de minor --vers major' "$log" \
  >/dev/null || { echo 'temoin_script_transmet_la_severite: arguments absents'; exit 1; }
grep -F 'applied=1' "$root/out" \
  >/dev/null || { echo 'temoin_script_applique: application absente'; exit 1; }
echo '3 passed / 0 failed / 0 ignored'
