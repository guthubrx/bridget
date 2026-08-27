#!/usr/bin/env bash
# One-shot référent : reclasse nature via `maicie registre requalifier`.
# Défaut = dry-run. --apply écrit réellement. JAMAIS lancé par un modèle.
#
# Rejouable : si la nature courante vaut déjà nature_vers, SKIP (pas de
# second append). Les lignes Blocker→règle DOIVENT porter severity_de/
# severity_vers : la garde « une règle ne peut pas porter blocker » reste ;
# la migration la franchit en requalifiant nature ET sévérité dans le même
# geste, avec trace au journal.
set -euo pipefail
CONFIG="${MAICIE_CONFIG:?MAICIE_CONFIG absolu obligatoire}"
TABLE="${1:?table jsonl obligatoire}"
BIN="${MAICIE_BIN:-./target/release/maicie}"
APPLY=0
[[ "${2:-}" == "--apply" ]] && APPLY=1
DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
REF="${NATURE_RECLASS_REF:?NATURE_RECLASS_REF=sha:<hex> ou mesure:<N/M> obligatoire}"

CATALOGUE_PATH="${CATALOGUE_PATH:-}"
if [[ -z "$CATALOGUE_PATH" ]]; then
  CATALOGUE_PATH="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("catalogue_path",""))' "$CONFIG")"
fi
[[ -n "$CATALOGUE_PATH" && -f "$CATALOGUE_PATH" ]] || {
  echo "catalogue introuvable (CATALOGUE_PATH ou config.catalogue_path)" >&2
  exit 2
}

export CONFIG TABLE BIN APPLY DATE REF CATALOGUE_PATH
python3 - <<'PY'
import json, os, subprocess, sys

config = os.environ["CONFIG"]
table = os.environ["TABLE"]
bin_path = os.environ["BIN"]
apply = os.environ["APPLY"] == "1"
date = os.environ["DATE"]
ref = os.environ["REF"]
catalogue = os.environ["CATALOGUE_PATH"]

def current_qualification(cid: str) -> tuple[str, str]:
    nature = "constat"
    severity = "minor"
    with open(catalogue, encoding="utf-8") as fh:
        for raw in fh:
            raw = raw.strip()
            if not raw.startswith("{"):
                continue
            try:
                entry = json.loads(raw)
            except json.JSONDecodeError:
                continue
            if entry.get("kind") == "add" and entry.get("id") == cid:
                nature = entry.get("nature", "constat")
                severity = entry.get("severity", "minor")
            if (
                entry.get("kind") == "transition"
                and entry.get("constat_id") == cid
                and entry.get("trigger") == "requalified"
            ):
                if entry.get("nature_to"):
                    nature = entry["nature_to"]
                if entry.get("severity_to"):
                    severity = entry["severity_to"]
    return nature, severity

n = skipped = applied = 0
with open(table, encoding="utf-8") as fh:
    for line in fh:
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        row = json.loads(line)
        n += 1
        cid = row["id"]
        de = row["nature_de"]
        vers = row["nature_vers"]
        raison = row["raison"]
        sev_de = row.get("severity_de") or ""
        sev_vers = row.get("severity_vers") or ""

        current_nature, current_severity = current_qualification(cid)
        if current_nature == vers and (not sev_vers or current_severity == sev_vers):
            print(f"SKIP_ALREADY id={cid} nature={vers} severity={current_severity}")
            skipped += 1
            continue

        if (sev_de and not sev_vers) or (sev_vers and not sev_de):
            print(
                f"ligne invalide {cid} : severity_de et severity_vers ensemble ou absents",
                file=sys.stderr,
            )
            sys.exit(2)

        cmd = [
            bin_path,
            "registre",
            "requalifier",
            "--config",
            config,
            "--constat",
            cid,
            "--nature-de",
            de,
            "--nature-vers",
            vers,
            "--raison",
            raison,
            "--ref",
            ref,
            "--date",
            date,
        ]
        if sev_de and sev_vers:
            cmd.extend(["--de", sev_de, "--vers", sev_vers])

        if apply:
            subprocess.run(cmd, check=True)
            applied += 1
        else:
            print("DRY-RUN " + " ".join(cmd))

print(f"lignes={n} skipped={skipped} applied={applied} apply={int(apply)}")
PY
