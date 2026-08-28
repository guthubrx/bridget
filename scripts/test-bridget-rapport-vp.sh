#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ronde="${root_dir}/scripts/bridget-ronde.py"
config="${BRIDGET_VP_REAL_CONFIG:-${HOME}/.config/maicie/config.json}"
python_bin="${PYTHON_BIN:-/usr/bin/python3}"

[[ -f "$config" ]] || {
  echo "base réelle Maicie introuvable via la configuration: $config" >&2
  exit 2
}

fixture_root="$(mktemp -d -t bridget-vp-real.XXXXXX)"
cleanup() {
  rm -rf "$fixture_root"
}
trap cleanup EXIT

fake_bridget="${fixture_root}/bridget"
fake_maicie="${fixture_root}/maicie"
cat >"$fake_bridget" <<'EOF'
#!/usr/bin/env bash
case "$*" in
  "agents --json"|"requests --all --json") printf '%s\n' '[]' ;;
  *) exit 9 ;;
esac
EOF
cat >"$fake_maicie" <<'EOF'
#!/usr/bin/env bash
[[ "$1" == registre ]] || exit 9
printf 'REGISTRE\n'
EOF
chmod 0755 "$fake_bridget" "$fake_maicie"

# Cette borne reproduit exactement la population humaine gelée. La ronde lit
# la vraie base par sa copie jetable ; le harnais ne reconstruit pas les 169
# objectifs depuis des littéraux.
output="$($python_bin "$ronde" \
  --json \
  --config "$config" \
  --bridget-bin "$fake_bridget" \
  --maicie-bin "$fake_maicie" \
  --now 1787874934)"

"$python_bin" - "$output" <<'PY'
import json
import sys

report = json.loads(sys.argv[1])
measure = report.get("verification_production")
assert measure is not None, "la ronde réelle ne publie pas encore la mesure V/P"
assert measure["state"] == "available"
assert measure["target"] is None
assert measure["origin_used"] is False

baseline = measure["populations"]["human_baseline_2026_08_27"]
rolling = measure["populations"]["rolling_24h"]
for population in (baseline, rolling):
    assert population["population"] == {
        "count": 169,
        "id_bytes": 6253,
        "sha256": "e2a279624ecd4fbd9ca7f8f97effa45777c15b6b3b9574f3d43a9d90298d5a55",
    }
    assert len(population["manifest"]["entries"]) == 169
    assert population["coverage"]["classification"] == {
        "classified": 5,
        "total": 169,
        "ratio": 5 / 169,
    }
    assert population["coverage"]["root"] == {
        "rooted": 5,
        "total": 169,
        "ratio": 5 / 169,
    }
    assert population["objectives"]["counts"] == {
        "production": 1,
        "verification": 4,
        "instruction": 0,
        "repair": 0,
        "indeterminate": 164,
    }
    assert population["roots"]["counts"]["production"] == 1
    assert population["roots"]["counts"]["verification"] == 1
    assert population["roots"]["expansion_factor"]["verification"] == 4.0
    assert population["objectives"]["ratios"]["verification_per_production"]["state"] == "unavailable"
    assert population["roots"]["ratios"]["verification_per_production"]["state"] == "unavailable"

entries = {entry["objective_id"]: entry for entry in baseline["manifest"]["entries"]}
root = "lot:maicie-republish-before-migrate"
assert entries["17441d4f-244b-41dc-8c0f-a26110a55276"]["class"] == "production"
for objective_id in (
    "d105d9c4-8f67-482f-8f53-5f6725229f74",
    "3344a60b-6f3f-4cd3-ac07-84ab40483cef",
    "f11f889b-4f22-482a-9960-fb45348bdd01",
    "83240941-9a69-4879-8bd7-95b5534c01c3",
):
    assert entries[objective_id]["class"] == "verification"
    assert entries[objective_id]["root_id"] == root
assert entries["17441d4f-244b-41dc-8c0f-a26110a55276"]["root_id"] == root

# Contre-épreuve du découpage : quatre objectifs de vérification réels sous
# une même racine ne produisent qu'une racine vérifiée. Le mutant qui remplace
# root_id par objective_id rendrait 4 ici et meurt après lecture réussie.
assert baseline["objectives"]["counts"]["verification"] == 4
assert baseline["roots"]["counts"]["verification"] == 1
PY

echo "rapport V/P réel : baseline, couverture et invariance de racine vérifiées"
