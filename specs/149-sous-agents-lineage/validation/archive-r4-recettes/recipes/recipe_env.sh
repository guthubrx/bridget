#!/bin/zsh
# recipe_env.sh — Prépare et vérifie l'environnement de recette native T037/T038 (session 149, r2).
# Aucun daemon, aucun modèle, aucun cargo : ce script ne fait que vérifier et créer des répertoires.
# Sécurité : ne lit et n'écrit JAMAIS dans BRIDGET_HOME prod (/Users/moi/.cache/bridget-core)
# ni dans /Users/moi/.config/bridget. Fixture privée sous /Users/moi/.cache (0700, APFS POSIX).
#
# r2 — corrections fondées sur la source Sol et la doc primaire des permissions :
#  - le mapping claude→claude accepte les règles SETTINGS préexistantes
#    (native_permissions.rs:227-252 rechecke les sources ; claude_inputs_without_rules
#    ne concerne QUE claude→codex, l.185-260). Le refus global des règles est retiré ;
#  - fixture POSITIVE : settings projet avec allow/deny réels. Syntaxe vérifiée contre
#    https://code.claude.com/docs/en/permissions : les chemins ne sont consultés que pour
#    Edit(path) et Read(path) (une règle Write(path) serait ignorée) ; "/path" est relatif
#    à la source settings. La règle deny Bash est un refus d'OUTIL, pas un confinement OS ;
#  - baseline filesystem avant exécution (constats avant/après).
set -u

RECIPE_DIR="${0:A:h}"
WORKTREE="${RECIPE_DIR%/specs/149-sous-agents-lineage/validation/recipes}"
FIXTURE_ROOT="${BRIDGET_149_FIXTURE_ROOT:-/Users/moi/.cache/bridget149-recipe}"
BIN149="${BRIDGET_149_BIN:-$WORKTREE/target/release/bridget}"

# Constantes de transparence : sha256 des lanceurs attestés (contrat permissions.md).
TRANSPARENT_GCLAUDE="sha256:dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e"
GCLAUDE="/Users/moi/.local/bin/gclaude"
CODEXPRO="/Users/moi/.local/bin/codex-pro"
GLM_PROFILE="/Users/moi/.claude-glm"

fail() { print -u2 "ECHEC: $1"; exit 1; }

# --- 1. Prérequis binaire 149 -------------------------------------------------
[[ -x "$BIN149" ]] || fail "binaire 149 absent ou non exécutable: $BIN149 (exporter BRIDGET_149_BIN)"
BIN_DIGEST="$(shasum -a 256 "$BIN149" | awk '{print $1}')"

# --- 2. Lanceurs : hash réel == constantes (composition attestée) -------------
[[ -x "$GCLAUDE" ]] || fail "lanceur gclaude absent: $GCLAUDE"
ACTUAL="$(shasum -a 256 "$GCLAUDE" | awk '{print $1}')"
[[ "sha256:$ACTUAL" == "$TRANSPARENT_GCLAUDE" ]] \
  || fail "hash gclaude $ACTUAL != constante contrat $TRANSPARENT_GCLAUDE"
[[ -x "$CODEXPRO" ]] || fail "lanceur codex-pro absent: $CODEXPRO"

# Résolution du CLI claude derrière le lanceur (symlink canonisé, sans secrets).
CLI_RESOLVED="$(command -v claude || true)"
[[ -n "$CLI_RESOLVED" ]] || CLI_RESOLVED="$("$GCLAUDE" --version >/dev/null 2>&1; command -v claude || true)"
[[ -n "$CLI_RESOLVED" ]] || fail "CLI claude introuvable sous PATH"
CLI_RESOLVED="$(python3 -c 'import os,sys;print(os.path.realpath(sys.argv[1]))' "$CLI_RESOLVED")"
CLI_DIGEST="$(shasum -a 256 "$CLI_RESOLVED" | awk '{print $1}')"

# --- 3. Profil GLM : présent, métadonnées seulement (aucune valeur copiée) ----
[[ -d "$GLM_PROFILE" ]] || fail "profil GLM absent: $GLM_PROFILE"

# --- 4. Arborescence fixture 0700 sous /Users/moi/.cache ----------------------
umask 077
PARENT_PROJ="$FIXTURE_ROOT/project/parent-proj"
for dir in "$FIXTURE_ROOT" "$FIXTURE_ROOT/home" "$FIXTURE_ROOT/logs" \
           "$PARENT_PROJ" "$PARENT_PROJ/allowed" "$PARENT_PROJ/forbidden" \
           "$PARENT_PROJ/.claude" "$FIXTURE_ROOT/outside" "$FIXTURE_ROOT/state"; do
  mkdir -p "$dir" || fail "mkdir $dir"
  chmod 700 "$dir"
done
# .git de bornage pour la capture des ancêtres settings (aucun contenu projet).
[[ -d "$PARENT_PROJ/.git" ]] || mkdir "$PARENT_PROJ/.git" || fail "mkdir .git"

# --- 5. Fixture POSITIVE : règles settings préexistantes ----------------------
# r1 refusait toute règle (hypothèse de harnais fausse) : le mapping claude→claude
# clone+rechecke ces sources (l.227-252) et les conserve dans l'enfant.
# Edit(path) couvre aussi l'outil Write (doc permissions, v2.1.210+) ; Write(path)
# ne serait jamais consulté. "/x" = relatif à la source settings (= cwd projet ici).
if [[ ! -f "$PARENT_PROJ/.claude/settings.json" ]]; then
  cat > "$PARENT_PROJ/.claude/settings.json" <<'JSON'
{
  "permissions": {
    "allow": [
      "Read",
      "Glob",
      "Grep",
      "Edit(/allowed/**)",
      "mcp__bridget__bridget_delegate",
      "mcp__bridget__bridget_task_status",
      "mcp__bridget__bridget_task_cancel"
    ],
    "deny": [
      "Bash",
      "Edit(/forbidden/**)"
    ]
  }
}
JSON
  chmod 600 "$PARENT_PROJ/.claude/settings.json"
fi
# oracle filesystem : rien ne doit préexister dans allowed/ ni forbidden/
find "$PARENT_PROJ/allowed" "$PARENT_PROJ/forbidden" -type f -print -quit | grep -q . \
  && fail "allowed/ ou forbidden/ contient déjà des fichiers (baseline invalide)"

# --- 6. Registre fixture (clone contrôlé, sans secrets) -----------------------
python3 "$RECIPE_DIR/make_fixture_registry.py" --home "$FIXTURE_ROOT/home" --force \
  || fail "génération registre fixture"

# --- 7. Empreintes figées pour verify_oracles.py ------------------------------
python3 - "$FIXTURE_ROOT/state/expected.json" "$BIN149" "$GCLAUDE" "$CODEXPRO" "$CLI_RESOLVED" <<'PY'
import hashlib, json, os, sys
out = sys.argv[1]
def digest(path):
    h = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()
expected = {
    "bin149_sha256": digest(sys.argv[2]),
    "launcher_gclaude_sha256": digest(sys.argv[3]),
    "launcher_codexpro_sha256": digest(sys.argv[4]),
    "resolved_claude_path": sys.argv[5],
    "resolved_claude_sha256": digest(sys.argv[5]),
}
with open(os.open(out, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600), "w") as handle:
    json.dump(expected, handle, indent=1, sort_keys=True)
PY
[[ -f "$FIXTURE_ROOT/state/expected.json" ]] || fail "empreintes non écrites"

# --- 7b. Empreintes des sources sélectionnées (profil glm + fixture projet) ---
python3 - "$FIXTURE_ROOT/state/sources-snapshot.json" "$FIXTURE_ROOT" <<'PY'
import hashlib, json, os, sys
out, fixture = sys.argv[1], sys.argv[2]
sources = [
    "/Users/moi/.claude-glm/settings.json",
    "/Library/Application Support/ClaudeCode/managed-settings.json",
    "/Users/moi/.claude-glm/plugins/cache/humanizer/humanizer/3.1.0/.claude-plugin/plugin.json",
    "/Users/moi/.claude-glm/plugins/cache/typesafe-ai/typesafe/0.5.7/.claude-plugin/plugin.json",
    os.path.join(fixture, "project/parent-proj/.claude/settings.json"),
]
def digest(path):
    h = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()
snapshot = {path: digest(path) for path in sources if os.path.exists(path)}
with open(os.open(out, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600), "w") as handle:
    json.dump(snapshot, handle, indent=1, sort_keys=True)
PY
[[ -f "$FIXTURE_ROOT/state/sources-snapshot.json" ]] || fail "snapshot sources non écrit"

# --- 7c. Baseline filesystem (constat avant exécution) ------------------------
python3 - "$FIXTURE_ROOT/state/fs-baseline.json" "$FIXTURE_ROOT" <<'PY'
import hashlib, json, os, sys
out, fixture = sys.argv[1], sys.argv[2]
def digest(path):
    h = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()
baseline = {}
for root, dirs, files in os.walk(fixture):
    dirs[:] = [d for d in dirs if d not in ("state", "logs", "home")]
    for name in files:
        path = os.path.join(root, name)
        baseline[os.path.relpath(path, fixture)] = digest(path)
with open(os.open(out, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600), "w") as handle:
    json.dump(baseline, handle, indent=1, sort_keys=True)
PY
[[ -f "$FIXTURE_ROOT/state/fs-baseline.json" ]] || fail "baseline filesystem non écrite"

# --- 8. Absence de vecteur T3 dans l'environnement de recette -----------------
if [[ -n "${BRIDGET_T3_MCP_ENDPOINT:-}" || -n "${BRIDGET_T3_MCP_AUTHORIZATION:-}" ]]; then
  fail "BRIDGET_T3_MCP_ENDPOINT/AUTHORIZATION présents : voie T3 interdite pour T038 (déséxporter)"
fi

print "OK environnement recette r2:"
print "  fixture      = $FIXTURE_ROOT (0700, settings projet positif allow/deny)"
print "  binaire149   = $BIN149 (sha256 $BIN_DIGEST)"
print "  gclaude      = $GCLAUDE (hash == constante contrat)"
print "  codex-pro    = $CODEXPRO (hash figé dans expected.json)"
print "  claude réel  = $CLI_RESOLVED (sha256 $CLI_DIGEST)"
print "  registre     = $FIXTURE_ROOT/home/agents.json (glm flash + codex, sans secrets)"
print "Étape suivante : run_fixture_daemon.sh (ne pas lancer pendant la préparation)."
