#!/usr/bin/env bash
# K1 — installateur Bridget + Maicie (macOS / launchd).
#
# Trois gardes :
#   1. Idempotent : une machine déjà installée dit « déjà en place », sans toucher.
#   2. Jamais d'écrasement silencieux : --force exigé pour remplacer.
#   3. Outil vs projet : pose le générique ; profils et catalogue restent des gabarits vides.
#
# Preuves runtime (2026-08-24) — trois niveaux :
#   PROUVÉ ICI (machine réelle) : second passage créés=0 / remplacés=0 ;
#     checksums des artefacts préexistants inchangés ; daemon joignable
#     (--verify-daemon-only). Premier passage a seulement comblé les trous
#     (maicie manquant, adapter test) sans toucher agents.json ni configs.
#   PROUVÉ BAC À SABLE (HOME jetable, --skip-launchd) : pose des binaires,
#     registres en 0600, plists, gabarits profiles=[] / type test seul ;
#     second passage créés=0.
#   NON PROUVÉ : activation launchd sur machine vierge ; spawn/stop agent
#     test + dépôt guichet relevé bout en bout (déclarés NON FAIT / NON
#     VÉRIFIÉ dans le rapport du script).
#
# Flags :
#   --catalogue-path PATH   journal du dû (défaut: $HOME/.cache/bridget/catalogue.jsonl)
#   --force                 autorise le remplacement explicite
#   --skip-launchd          pose les plists sans bootstrap/load (bac à sable HOME)
#   --skip-verify           pose seulement ; pas de preuve runtime
#   --verify-daemon-only    après pose : uniquement bridget status
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: install-k1.sh [options]

Options:
  --catalogue-path PATH   chemin absolu du journal du dû
  --force                 remplace les artefacts existants (explicite)
  --skip-launchd          n'active pas launchd (pose des plists seulement)
  --skip-verify           pas de vérification runtime
  --verify-daemon-only    vérifie seulement que le daemon répond
  -h, --help              cette aide

Idempotence : sans --force, tout fichier déjà présent est laissé intact
et annoncé « déjà en place ».
EOF
}

CATALOGUE_PATH=""
FORCE="0"
SKIP_LAUNCHD="0"
SKIP_VERIFY="0"
VERIFY_DAEMON_ONLY="0"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --catalogue-path)
      CATALOGUE_PATH="${2:-}"
      [[ -n "$CATALOGUE_PATH" ]] || { echo "valeur manquante pour --catalogue-path" >&2; exit 2; }
      shift 2
      ;;
    --force) FORCE="1"; shift ;;
    --skip-launchd) SKIP_LAUNCHD="1"; shift ;;
    --skip-verify) SKIP_VERIFY="1"; shift ;;
    --verify-daemon-only) VERIFY_DAEMON_ONLY="1"; shift ;;
    -h|--help) usage; exit 0 ;;
    *)
      echo "option inconnue: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "install-k1.sh: OS non supporté (attendu Darwin/launchd)" >&2
  exit 2
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR_BRIDGET="${HOME}/.config/bridget"
CONFIG_DIR_MAICIE="${HOME}/.config/maicie"
LAUNCHD_DIR="${HOME}/Library/LaunchAgents"
CACHE_DIR="${HOME}/.cache/bridget"
SHARE_DIR="${HOME}/.local/share/bridget/agents"

BRIDGET_SOCKET="${CACHE_DIR}/bridget.sock"
MAICIE_DB_PATH="${CACHE_DIR}/maicie.sqlite3"
DEFAULT_CATALOGUE_PATH="${CACHE_DIR}/catalogue.jsonl"
CATALOGUE_PATH="${CATALOGUE_PATH:-$DEFAULT_CATALOGUE_PATH}"

BRIDGET_BIN="${INSTALL_DIR}/bridget"
MAICIE_BIN="${INSTALL_DIR}/maicie"
MAICIE_SUIVI_BIN="${INSTALL_DIR}/maicie-suivi"
AGENTS_JSON="${CONFIG_DIR_BRIDGET}/agents.json"
MAICIE_CONFIG="${CONFIG_DIR_MAICIE}/config.json"
DAEMON_PLIST="${LAUNCHD_DIR}/com.bridget.daemon.plist"
MAICIE_RELEVE_PLIST="${LAUNCHD_DIR}/com.bridget.maicie.releve.plist"

TEST_AGENT_TYPE="k1-test-fixture"
TEST_AGENT_NAME="k1-agent-1"
ADAPTER_PATH="${SHARE_DIR}/${TEST_AGENT_TYPE}.sh"

CREATED=0
SKIPPED=0
TOUCHED=0

log() { echo "install-k1: $*"; }
die() { echo "install-k1: ERREUR: $*" >&2; exit 1; }

already() {
  log "déjà en place: $1"
  SKIPPED=$((SKIPPED + 1))
}

created() {
  log "créé: $1"
  CREATED=$((CREATED + 1))
}

# Refuse d'écraser sans --force. Retourne 0 si on peut écrire, 1 si skip.
may_write() {
  local path="$1"
  local label="$2"
  if [[ -e "$path" && "$FORCE" != "1" ]]; then
    already "$label ($path)"
    return 1
  fi
  if [[ -e "$path" && "$FORCE" == "1" ]]; then
    log "remplacement explicite (--force): $label ($path)"
    TOUCHED=$((TOUCHED + 1))
  fi
  return 0
}

ensure_dirs() {
  mkdir -p "$INSTALL_DIR" "$CONFIG_DIR_BRIDGET" "$CONFIG_DIR_MAICIE" \
    "$LAUNCHD_DIR" "$CACHE_DIR" "$SHARE_DIR"
}

build_release_if_needed() {
  # Ne compile que si au moins un binaire cible manque, ou si --force.
  if [[ -e "$BRIDGET_BIN" && -e "$MAICIE_BIN" && "$FORCE" != "1" ]]; then
    already "binaires bridget + maicie"
    return 0
  fi
  log "build release (bridget + maicie)"
  (
    cd "$ROOT_DIR"
    export PATH="${HOME}/.cargo/bin:${PATH:-}"
    # Dans un HOME jetable, cargo/rustc restent ceux de la machine hôte.
    if [[ -n "${K1_HOST_CARGO_HOME:-}" ]]; then
      export CARGO_HOME="$K1_HOST_CARGO_HOME"
    fi
    if [[ -n "${K1_HOST_RUSTUP_HOME:-}" ]]; then
      export RUSTUP_HOME="$K1_HOST_RUSTUP_HOME"
    fi
    cargo build --release -p bridget-daemon -p maicie
  ) >/tmp/k1-build.out 2>/tmp/k1-build.err || {
    cat /tmp/k1-build.err >&2
    die "build release a échoué"
  }
  [[ -x "${ROOT_DIR}/target/release/bridget" ]] || die "binaire bridget absent après build"
  [[ -x "${ROOT_DIR}/target/release/maicie" ]] || die "binaire maicie absent après build"
}

install_binary() {
  local src="$1"
  local dst="$2"
  local label="$3"
  may_write "$dst" "$label" || return 0
  install -d "$(dirname "$dst")"
  install -m 0755 "$src" "$dst"
  created "$label ($dst)"
}

write_test_adapter() {
  may_write "$ADAPTER_PATH" "adapter test" || return 0
  cat >"$ADAPTER_PATH" <<'EOF'
#!/bin/sh
# Fixture minimale claude_stream_json — spawn/stop seulement, pas un LLM.
while IFS= read -r line; do
  printf '%s\n' '{"type":"system","subtype":"init","model":"k1-test-model"}'
  printf '%s\n' '{"type":"stream_event","event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"k1-test: ok"}}}'
  printf '%s\n' '{"type":"result","is_error":false,"terminal_reason":"completed","result":"k1-test: ok"}'
done
EOF
  chmod 0700 "$ADAPTER_PATH"
  created "adapter test ($ADAPTER_PATH)"
}

write_agents_json() {
  # Gabarit outil : un seul type de test. Profils projet = hors périmètre.
  # Si le fichier existe déjà, on ne le touche JAMAIS (même pour ajouter le type test).
  if [[ -e "$AGENTS_JSON" && "$FORCE" != "1" ]]; then
    already "registre agents.json ($AGENTS_JSON)"
    return 0
  fi
  if [[ -e "$AGENTS_JSON" && "$FORCE" == "1" ]]; then
    log "remplacement explicite (--force): registre agents.json ($AGENTS_JSON)"
    TOUCHED=$((TOUCHED + 1))
  fi
  python3 - "$AGENTS_JSON" "$ADAPTER_PATH" "$TEST_AGENT_TYPE" <<'PY'
import json, os, sys
path, adapter, agent_type = sys.argv[1], sys.argv[2], sys.argv[3]
data = {
    "agents": {
        agent_type: {
            "command": adapter,
            "args": [],
            "protocol": "claude_stream_json",
            "permissions": "allow",
            "queue_capacity": 1,
            "notify_timeout_secs": 2,
            "pass_env": [],
            "forbidden_env": [],
        }
    }
}
os.makedirs(os.path.dirname(path), exist_ok=True)
tmp = path + ".tmp"
with open(tmp, "w", encoding="utf-8") as f:
    json.dump(data, f, ensure_ascii=False, indent=2)
    f.write("\n")
os.replace(tmp, path)
os.chmod(path, 0o600)
PY
  created "registre agents.json ($AGENTS_JSON) mode 0600"
}

write_maicie_config() {
  if [[ -e "$MAICIE_CONFIG" && "$FORCE" != "1" ]]; then
    already "config maicie ($MAICIE_CONFIG)"
    return 0
  fi
  if [[ -e "$MAICIE_CONFIG" && "$FORCE" == "1" ]]; then
    log "remplacement explicite (--force): config maicie ($MAICIE_CONFIG)"
    TOUCHED=$((TOUCHED + 1))
  fi
  mkdir -p "$(dirname "$MAICIE_DB_PATH")" "$(dirname "$CATALOGUE_PATH")"
  if [[ ! -e "$CATALOGUE_PATH" ]]; then
    : >"$CATALOGUE_PATH"
    created "catalogue vide ($CATALOGUE_PATH)"
  else
    already "catalogue ($CATALOGUE_PATH)"
  fi
  cat >"$MAICIE_CONFIG" <<JSON
{
  "version": 1,
  "bridget_socket": "$BRIDGET_SOCKET",
  "database_path": "$MAICIE_DB_PATH",
  "durations": {
    "short_secs": 30,
    "normal_secs": 300,
    "long_secs": 3600
  },
  "status_capture_budget_ms": 250,
  "catalogue_path": "$CATALOGUE_PATH",
  "profiles": []
}
JSON
  chmod 0600 "$MAICIE_CONFIG"
  created "config maicie ($MAICIE_CONFIG) mode 0600, profiles=[]"
}

write_maicie_suivi() {
  may_write "$MAICIE_SUIVI_BIN" "lien maicie-suivi" || return 0
  cat >"$MAICIE_SUIVI_BIN" <<EOF
#!/usr/bin/env bash
set -euo pipefail
CFG="${MAICIE_CONFIG}"
echo "maicie-suivi: \$(date -u +'%Y-%m-%dT%H:%M:%SZ')"
exec "${MAICIE_BIN}" status --config "\$CFG" --json
EOF
  chmod 0755 "$MAICIE_SUIVI_BIN"
  created "lien maicie-suivi ($MAICIE_SUIVI_BIN)"
}

write_plist_daemon() {
  may_write "$DAEMON_PLIST" "plist daemon" || return 0
  cat >"$DAEMON_PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>com.bridget.daemon</string>
  <key>ProgramArguments</key>
  <array>
    <string>${BRIDGET_BIN}</string>
    <string>daemon</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>RUST_LOG</key><string>info</string>
    <key>HOME</key><string>${HOME}</string>
  </dict>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>StandardOutPath</key><string>${CACHE_DIR}/daemon-stdout.log</string>
  <key>StandardErrorPath</key><string>${CACHE_DIR}/daemon-stderr.log</string>
</dict>
</plist>
EOF
  chmod 0644 "$DAEMON_PLIST"
  created "plist daemon ($DAEMON_PLIST)"
}

write_plist_maicie_releve() {
  may_write "$MAICIE_RELEVE_PLIST" "plist maicie.releve" || return 0
  cat >"$MAICIE_RELEVE_PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>com.bridget.maicie.releve</string>
  <key>ProgramArguments</key>
  <array>
    <string>${MAICIE_BIN}</string>
    <string>status</string>
    <string>--config</string>
    <string>${MAICIE_CONFIG}</string>
    <string>--json</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>RUST_LOG</key><string>info</string>
    <key>HOME</key><string>${HOME}</string>
  </dict>
  <key>RunAtLoad</key><true/>
  <key>StartInterval</key><integer>120</integer>
  <key>StandardOutPath</key><string>${CACHE_DIR}/maicie-releve-stdout.log</string>
  <key>StandardErrorPath</key><string>${CACHE_DIR}/maicie-releve-stderr.log</string>
</dict>
</plist>
EOF
  chmod 0644 "$MAICIE_RELEVE_PLIST"
  created "plist maicie.releve ($MAICIE_RELEVE_PLIST)"
}

activate_launchd() {
  if [[ "$SKIP_LAUNCHD" == "1" ]]; then
    log "NON FAIT: activation launchd (--skip-launchd)"
    return 0
  fi
  log "charger services launchd (idempotent si déjà chargés)"
  launchctl bootstrap "gui/$(id -u)" "$DAEMON_PLIST" 2>/dev/null \
    || launchctl load "$DAEMON_PLIST" 2>/dev/null \
    || true
  launchctl bootstrap "gui/$(id -u)" "$MAICIE_RELEVE_PLIST" 2>/dev/null \
    || launchctl load "$MAICIE_RELEVE_PLIST" 2>/dev/null \
    || true
}

verify_daemon() {
  local attempts=20
  local i
  for ((i = 1; i <= attempts; i++)); do
    if [[ -x "$BRIDGET_BIN" ]] && "$BRIDGET_BIN" status >/dev/null 2>&1; then
      echo "VÉRIFIÉ: daemon joignable (bridget status)"
      return 0
    fi
    sleep 0.25
  done
  echo "NON VÉRIFIÉ: daemon joignable" >&2
  return 1
}

verify_spawn_stop() {
  if [[ ! -e "$AGENTS_JSON" ]]; then
    echo "NON VÉRIFIÉ: spawn/stop (agents.json absent)" >&2
    return 1
  fi
  if ! python3 - "$AGENTS_JSON" "$TEST_AGENT_TYPE" <<'PY'
import json, sys
path, agent_type = sys.argv[1], sys.argv[2]
with open(path, encoding="utf-8") as f:
    data = json.load(f)
sys.exit(0 if agent_type in data.get("agents", {}) else 1)
PY
  then
    echo "NON VÉRIFIÉ: spawn/stop (type $TEST_AGENT_TYPE absent du registre existant — non injecté par respect d'idempotence)" >&2
    return 1
  fi
  if ! "$BRIDGET_BIN" spawn "$TEST_AGENT_TYPE" --name "$TEST_AGENT_NAME" --timeout 5 \
    >/tmp/k1-spawn.out 2>/tmp/k1-spawn.err; then
    echo "NON VÉRIFIÉ: spawn agent test" >&2
    cat /tmp/k1-spawn.err >&2 || true
    return 1
  fi
  if ! "$BRIDGET_BIN" stop "$TEST_AGENT_NAME" >/tmp/k1-stop.out 2>/tmp/k1-stop.err; then
    echo "NON VÉRIFIÉ: stop agent test" >&2
    cat /tmp/k1-stop.err >&2 || true
    return 1
  fi
  echo "VÉRIFIÉ: spawn/stop agent test ($TEST_AGENT_NAME)"
  return 0
}

verify_maicie_status() {
  if [[ ! -e "$MAICIE_CONFIG" ]]; then
    echo "NON VÉRIFIÉ: maicie status (config absente)" >&2
    return 1
  fi
  if ! "$MAICIE_BIN" status --config "$MAICIE_CONFIG" --json >/tmp/k1-maicie-status.out 2>/tmp/k1-maicie-status.err; then
    echo "NON VÉRIFIÉ: maicie status" >&2
    cat /tmp/k1-maicie-status.err >&2 || true
    return 1
  fi
  echo "VÉRIFIÉ: maicie status (config chargeable)"
  return 0
}

run_verify() {
  if [[ "$SKIP_VERIFY" == "1" ]]; then
    log "NON FAIT: vérifications runtime (--skip-verify)"
    return 0
  fi
  local ok=1
  verify_daemon || ok=0
  if [[ "$VERIFY_DAEMON_ONLY" == "1" ]]; then
    echo "NON FAIT: spawn/stop agent test (--verify-daemon-only)"
    echo "NON FAIT: dépôt guichet relevé bout en bout (--verify-daemon-only)"
    [[ "$ok" == "1" ]] || return 1
    return 0
  fi
  verify_spawn_stop || ok=0
  verify_maicie_status || ok=0
  echo "NON FAIT: dépôt guichet relevé bout en bout (hors preuve K1 locale ; gate G1504 séparé)"
  [[ "$ok" == "1" ]] || return 1
  return 0
}

print_report() {
  cat <<EOF

install-k1: RAPPORT DE POSE
  créés:     $CREATED
  déjà là:   $SKIPPED
  remplacés: $TOUCHED (--force)

Artefacts:
  $BRIDGET_BIN
  $MAICIE_BIN
  $MAICIE_SUIVI_BIN
  $AGENTS_JSON
  $MAICIE_CONFIG
  $DAEMON_PLIST
  $MAICIE_RELEVE_PLIST
  catalogue: $CATALOGUE_PATH
EOF
}

main() {
  ensure_dirs
  build_release_if_needed

  if [[ -e "${ROOT_DIR}/target/release/bridget" ]]; then
    install_binary "${ROOT_DIR}/target/release/bridget" "$BRIDGET_BIN" "binaire bridget"
  fi
  if [[ -e "${ROOT_DIR}/target/release/maicie" ]]; then
    install_binary "${ROOT_DIR}/target/release/maicie" "$MAICIE_BIN" "binaire maicie"
  fi

  write_test_adapter
  write_agents_json
  write_maicie_config
  write_maicie_suivi
  write_plist_daemon
  write_plist_maicie_releve
  activate_launchd

  print_report

  if ! run_verify; then
    die "vérification incomplète ou échouée — voir NON VÉRIFIÉ ci-dessus"
  fi

  log "terminé"
}

main "$@"
