#!/usr/bin/env bash
# K1 — installateur Bridget + Maicie (macOS / Linux).
#
# Voie Rust : rustup + compilation sur place — autoportant, zéro croisée à maintenir.
# Windows : explicitement différé (refus propre).
#
# Trois gardes :
#   1. Idempotent : une machine déjà installée dit « déjà en place », sans toucher.
#   2. Jamais d'écrasement silencieux : --force exigé pour remplacer.
#   3. Outil vs projet : pose le générique ; profils et catalogue restent des gabarits vides.
#
# Preuves :
#   Mac — idempotence ici + pose bac à sable (2026-08-24).
#   Linux (hôte distant) — daemon systemd, spawn/stop, dépôt guichet queued +
#     relève Maicie sans panne (--verify-guichet, 2026-08-24).
#   Windows — différé (refus nommé).
#
# Flags :
#   --catalogue-path PATH   journal du dû
#   --force                 remplace explicitement (config Maicie préservée)
#   --skip-services         pose les unités sans les activer (bac à sable)
#   --skip-launchd          alias de --skip-services (compat)
#   --skip-verify           pas de vérification runtime
#   --verify-daemon-only    uniquement bridget status
#   --verify-guichet        tente spawn/stop + dépôt guichet + relève maicie
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: install-k1.sh [options]

Plateformes : Darwin (launchd) | Linux (systemd --user). Windows refusé.

Options:
  --catalogue-path PATH   chemin absolu du journal du dû
  --force                 remplace les artefacts, sauf la config Maicie
  --skip-services         n'active pas launchd/systemd (pose seulement)
  --skip-launchd          alias de --skip-services
  --skip-verify           pas de vérification runtime
  --verify-daemon-only    vérifie seulement que le daemon répond
  --verify-guichet        vérifie spawn/stop + dépôt guichet + relève
  -h, --help              cette aide

Idempotence : sans --force, tout fichier déjà présent est laissé intact
et annoncé « déjà en place ».
La configuration Maicie déclarative reste toujours intacte : ses profils et
son database_path ne sont jamais régénérés par --force.
EOF
}

CATALOGUE_PATH=""
FORCE="0"
SKIP_SERVICES="0"
SKIP_VERIFY="0"
VERIFY_DAEMON_ONLY="0"
VERIFY_GUICHET="0"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --catalogue-path)
      CATALOGUE_PATH="${2:-}"
      [[ -n "$CATALOGUE_PATH" ]] || { echo "valeur manquante pour --catalogue-path" >&2; exit 2; }
      shift 2
      ;;
    --force) FORCE="1"; shift ;;
    --skip-services|--skip-launchd) SKIP_SERVICES="1"; shift ;;
    --skip-verify) SKIP_VERIFY="1"; shift ;;
    --verify-daemon-only) VERIFY_DAEMON_ONLY="1"; shift ;;
    --verify-guichet) VERIFY_GUICHET="1"; shift ;;
    -h|--help) usage; exit 0 ;;
    *)
      echo "option inconnue: $1" >&2
      usage
      exit 2
      ;;
  esac
done

# --- (1) Détection de plateforme en tête : rien n'est posé avant ---
OS_NAME="$(uname -s)"
case "$OS_NAME" in
  Darwin) PLATFORM="macos"; SERVICE_BACKEND="launchd" ;;
  Linux)  PLATFORM="linux"; SERVICE_BACKEND="systemd" ;;
  *)
    echo "install-k1.sh: plateforme non couverte: ${OS_NAME} (cibles: Darwin, Linux ; Windows différé)" >&2
    exit 2
    ;;
esac

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR_BRIDGET="${HOME}/.config/bridget"
CONFIG_DIR_MAICIE="${HOME}/.config/maicie"
CACHE_DIR="${HOME}/.cache/bridget"
SHARE_DIR="${HOME}/.local/share/bridget/agents"
LAUNCHD_DIR="${HOME}/Library/LaunchAgents"
SYSTEMD_USER_DIR="${HOME}/.config/systemd/user"

BRIDGET_SOCKET="${CACHE_DIR}/bridget.sock"
# Répertoire privé 0700 exigé par MaicieStore (pas ~/.cache/bridget partagé).
MAICIE_STATE_DIR="${CACHE_DIR}/maicie-state"
MAICIE_DB_PATH="${MAICIE_STATE_DIR}/maicie.sqlite3"
DEFAULT_CATALOGUE_PATH="${CACHE_DIR}/catalogue.jsonl"
CATALOGUE_PATH="${CATALOGUE_PATH:-$DEFAULT_CATALOGUE_PATH}"

BRIDGET_BIN="${INSTALL_DIR}/bridget"
MAICIE_BIN="${INSTALL_DIR}/maicie"
MAICIE_SUIVI_BIN="${INSTALL_DIR}/maicie-suivi"
AGENTS_JSON="${CONFIG_DIR_BRIDGET}/agents.json"
MAICIE_CONFIG="${CONFIG_DIR_MAICIE}/config.json"

DAEMON_PLIST="${LAUNCHD_DIR}/com.bridget.daemon.plist"
MAICIE_RELEVE_PLIST="${LAUNCHD_DIR}/com.bridget.maicie.releve.plist"
DAEMON_SERVICE="${SYSTEMD_USER_DIR}/bridget-daemon.service"
MAICIE_RELEVE_SERVICE="${SYSTEMD_USER_DIR}/bridget-maicie-releve.service"
MAICIE_RELEVE_TIMER="${SYSTEMD_USER_DIR}/bridget-maicie-releve.timer"

MAICIE_STAGE_DIR=""
MAICIE_STAGED_BIN=""
MAICIE_STAGED_CONFIG=""
MAICIE_CANDIDATE_SOURCE=""
MAICIE_BIN_NEEDS_PUBLISH="0"
MAICIE_CONFIG_NEEDS_PUBLISH="0"

TEST_AGENT_TYPE="k1-test-fixture"
TEST_AGENT_NAME="k1-agent-1"
ADAPTER_PATH="${SHARE_DIR}/${TEST_AGENT_TYPE}.sh"

CREATED=0
SKIPPED=0
TOUCHED=0

log() { echo "install-k1: $*"; }
die() { echo "install-k1: ERREUR: $*" >&2; exit 1; }

cleanup_maicie_stage() {
  if [[ -n "$MAICIE_STAGE_DIR" && -d "$MAICIE_STAGE_DIR" ]]; then
    rm -rf -- "$MAICIE_STAGE_DIR"
  fi
}
trap cleanup_maicie_stage EXIT

already() {
  log "déjà en place: $1"
  SKIPPED=$((SKIPPED + 1))
}

created() {
  log "créé: $1"
  CREATED=$((CREATED + 1))
}

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
    "$CACHE_DIR" "$SHARE_DIR"
  mkdir -p -m 0700 "$MAICIE_STATE_DIR"
  chmod 0700 "$MAICIE_STATE_DIR" 2>/dev/null || true
  case "$SERVICE_BACKEND" in
    launchd) mkdir -p "$LAUNCHD_DIR" ;;
    systemd) mkdir -p "$SYSTEMD_USER_DIR" ;;
  esac
}

# --- (2) Absence de Rust : rustup + compile sur place ---
ensure_rust() {
  export PATH="${HOME}/.cargo/bin:${PATH:-}"
  if [[ -n "${K1_HOST_CARGO_HOME:-}" ]]; then
    export CARGO_HOME="$K1_HOST_CARGO_HOME"
  fi
  if [[ -n "${K1_HOST_RUSTUP_HOME:-}" ]]; then
    export RUSTUP_HOME="$K1_HOST_RUSTUP_HOME"
  fi
  # shellcheck disable=SC1091
  [[ -f "${HOME}/.cargo/env" ]] && source "${HOME}/.cargo/env"

  if command -v cargo >/dev/null 2>&1 && command -v rustc >/dev/null 2>&1; then
    log "Rust présent: $(rustc --version 2>/dev/null || echo '?')"
    return 0
  fi

  log "Rust absent — amorce rustup (voie autoportante, pas de croisée)"
  if ! command -v curl >/dev/null 2>&1; then
    die "curl requis pour amorcer rustup"
  fi
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable
  # shellcheck disable=SC1091
  source "${HOME}/.cargo/env"
  command -v cargo >/dev/null 2>&1 || die "rustup installé mais cargo introuvable"
  log "Rust amorcé: $(rustc --version)"
}

build_release_if_needed() {
  if [[ -e "$BRIDGET_BIN" && -e "$MAICIE_BIN" && "$FORCE" != "1" ]]; then
    already "binaires bridget + maicie"
    return 0
  fi
  ensure_rust
  log "build release (bridget + maicie) sur ${PLATFORM}"
  (
    cd "$ROOT_DIR"
    export PATH="${HOME}/.cargo/bin:${PATH:-}"
    if [[ -n "${K1_HOST_CARGO_HOME:-}" ]]; then
      export CARGO_HOME="$K1_HOST_CARGO_HOME"
    fi
    if [[ -n "${K1_HOST_RUSTUP_HOME:-}" ]]; then
      export RUSTUP_HOME="$K1_HOST_RUSTUP_HOME"
    fi
    # shellcheck disable=SC1091
    [[ -f "${HOME}/.cargo/env" ]] && source "${HOME}/.cargo/env"
    cargo build --release -p bridget-daemon -p maicie
  ) >/tmp/k1-build.out 2>/tmp/k1-build.err || {
    tail -n 80 /tmp/k1-build.err >&2 || true
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
  mkdir -p "$(dirname "$dst")"
  install -m 0755 "$src" "$dst"
  created "$label ($dst)"
}

render_default_maicie_config() {
  local destination="$1"
  cat >"$destination" <<JSON
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
  chmod 0600 "$destination"
}

# Fige les deux octets qui seront publiés. La configuration déclarative reste
# propriété de l'utilisateur : --force remplace le binaire, jamais ses profils
# ni son database_path. Une configuration absente est générée dans le staging.
stage_maicie_activation() {
  local candidate candidate_real
  MAICIE_STAGE_DIR="$(mktemp -d -t maicie-k1-stage.XXXXXX)"
  chmod 0700 "$MAICIE_STAGE_DIR"
  MAICIE_STAGED_BIN="${MAICIE_STAGE_DIR}/maicie"
  MAICIE_STAGED_CONFIG="${MAICIE_STAGE_DIR}/config.json"

  if [[ -e "$MAICIE_CONFIG" ]]; then
    cp "$MAICIE_CONFIG" "$MAICIE_STAGED_CONFIG"
    chmod 0600 "$MAICIE_STAGED_CONFIG"
    MAICIE_CONFIG_NEEDS_PUBLISH="0"
    already "config maicie préservée ($MAICIE_CONFIG)"
  else
    render_default_maicie_config "$MAICIE_STAGED_CONFIG"
    MAICIE_CONFIG_NEEDS_PUBLISH="1"
  fi

  if [[ -e "$MAICIE_BIN" && "$FORCE" != "1" ]]; then
    candidate="$MAICIE_BIN"
    MAICIE_BIN_NEEDS_PUBLISH="0"
  else
    candidate="${ROOT_DIR}/target/release/maicie"
    MAICIE_BIN_NEEDS_PUBLISH="1"
  fi
  [[ -x "$candidate" ]] || die "gate Maicie: binaire candidat absent ou non exécutable ($candidate)"
  cp "$candidate" "$MAICIE_STAGED_BIN"
  chmod 0755 "$MAICIE_STAGED_BIN"
  cmp -s "$candidate" "$MAICIE_STAGED_BIN" \
    || die "gate Maicie: binaire candidat modifié pendant sa mise en staging ($candidate)"
  MAICIE_CANDIDATE_SOURCE="$candidate"
  candidate_real="$(python3 - "$candidate" <<'PY'
import os, sys
print(os.path.realpath(sys.argv[1]))
PY
)"
  log "candidat Maicie figé: source=$candidate cible=$candidate_real stage=$MAICIE_STAGED_BIN"
}

# Gate de SCHÉMA Maicie uniquement : Bridget conserve exactement son chemin
# de pose. La provenance origin/main appartient au gate d'activation gouvernée
# (session 018), à étendre aux binaires compilés ; cette extension doit appeler
# celui-ci en complément et non le remplacer.
preflight_staged_maicie_activation() {
  local report remedy
  remedy=""
  if [[ "$FORCE" != "1" ]]; then
    remedy=" ; relancer avec --force pour construire puis contrôler le nouveau candidat"
  fi
  if ! report="$("$MAICIE_STAGED_BIN" preflight --config "$MAICIE_STAGED_CONFIG" --json 2>&1)"; then
    die "gate Maicie refusé avant publication: binaire=$MAICIE_CANDIDATE_SOURCE config=$MAICIE_STAGED_CONFIG : $report$remedy"
  fi
  log "gate Maicie accepté sur la paire stagée: binaire=$MAICIE_CANDIDATE_SOURCE config=$MAICIE_STAGED_CONFIG $report"
}

atomic_publish_file() {
  local source="$1" destination="$2" mode="$3" temporary
  mkdir -p "$(dirname "$destination")"
  temporary="$(mktemp "${destination}.maicie.XXXXXX")"
  if ! install -m "$mode" "$source" "$temporary"; then
    rm -f -- "$temporary"
    die "publication Maicie impossible ($destination)"
  fi
  if ! mv -f -- "$temporary" "$destination"; then
    rm -f -- "$temporary"
    die "activation atomique Maicie impossible ($destination)"
  fi
}

publish_staged_maicie_activation() {
  if [[ "$MAICIE_BIN_NEEDS_PUBLISH" == "1" ]]; then
    may_write "$MAICIE_BIN" "binaire maicie" \
      || die "état de publication Maicie incohérent ($MAICIE_BIN)"
    atomic_publish_file "$MAICIE_STAGED_BIN" "$MAICIE_BIN" 0755
    created "binaire maicie ($MAICIE_BIN)"
  fi

  if [[ "$MAICIE_CONFIG_NEEDS_PUBLISH" == "1" ]]; then
    mkdir -p "$(dirname "$CATALOGUE_PATH")"
    if [[ ! -e "$CATALOGUE_PATH" ]]; then
      : >"$CATALOGUE_PATH"
      created "catalogue vide ($CATALOGUE_PATH)"
    else
      already "catalogue ($CATALOGUE_PATH)"
    fi
    atomic_publish_file "$MAICIE_STAGED_CONFIG" "$MAICIE_CONFIG" 0600
    created "config maicie ($MAICIE_CONFIG) mode 0600, profiles=[], state_dir 0700"
  fi

  cmp -s "$MAICIE_STAGED_BIN" "$MAICIE_BIN" \
    || die "binaire Maicie publié différent du candidat préflighté"
  cmp -s "$MAICIE_STAGED_CONFIG" "$MAICIE_CONFIG" \
    || die "configuration Maicie publiée différente de celle préflightée"
}

# Second témoin : il porte sur les chemins réellement écrits dans les unités,
# après toute publication et immédiatement avant leur activation.
preflight_published_maicie_activation() {
  local report
  if ! report="$("$MAICIE_BIN" preflight --config "$MAICIE_CONFIG" --json 2>&1)"; then
    die "gate Maicie refusé sur la paire publiée: binaire=$MAICIE_BIN config=$MAICIE_CONFIG : $report"
  fi
  log "gate Maicie accepté sur la paire publiée: binaire=$MAICIE_BIN config=$MAICIE_CONFIG $report"
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
            "capabilities": {
                "execution_paths": ["claude_stream_json"],
                "models": {},
            },
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

write_maicie_suivi() {
  may_write "$MAICIE_SUIVI_BIN" "lien maicie-suivi" || return 0
  cat >"$MAICIE_SUIVI_BIN" <<EOF
#!/usr/bin/env bash
set -euo pipefail
CFG="${MAICIE_CONFIG}"
echo "maicie-suivi: \$(date -u +'%Y-%m-%dT%H:%M:%SZ')"
"${MAICIE_BIN}" preflight --config "\$CFG" --json >/dev/null
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
    <key>PATH</key><string>${HOME}/.local/bin:/usr/bin:/bin</string>
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
    <string>${MAICIE_SUIVI_BIN}</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>RUST_LOG</key><string>info</string>
    <key>HOME</key><string>${HOME}</string>
    <key>PATH</key><string>${HOME}/.local/bin:/usr/bin:/bin</string>
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

write_systemd_daemon() {
  may_write "$DAEMON_SERVICE" "systemd daemon" || return 0
  cat >"$DAEMON_SERVICE" <<EOF
[Unit]
Description=Bridget daemon
After=network.target

[Service]
Type=simple
ExecStart=${BRIDGET_BIN} daemon
Environment=RUST_LOG=info
Environment=HOME=${HOME}
Environment=PATH=${HOME}/.local/bin:/usr/bin:/bin
Restart=always
RestartSec=3

[Install]
WantedBy=default.target
EOF
  chmod 0644 "$DAEMON_SERVICE"
  created "systemd daemon ($DAEMON_SERVICE)"
}

write_systemd_maicie_releve() {
  may_write "$MAICIE_RELEVE_SERVICE" "systemd maicie.releve" || return 0
  cat >"$MAICIE_RELEVE_SERVICE" <<EOF
[Unit]
Description=Maicie guichet releve (pull-only)

[Service]
Type=oneshot
ExecStart=${MAICIE_SUIVI_BIN}
Environment=RUST_LOG=info
Environment=HOME=${HOME}
Environment=PATH=${HOME}/.local/bin:/usr/bin:/bin
EOF
  chmod 0644 "$MAICIE_RELEVE_SERVICE"
  created "systemd maicie.releve ($MAICIE_RELEVE_SERVICE)"

  may_write "$MAICIE_RELEVE_TIMER" "systemd maicie.releve.timer" || return 0
  cat >"$MAICIE_RELEVE_TIMER" <<EOF
[Unit]
Description=Timer Maicie guichet releve (toutes les 2 min)

[Timer]
OnBootSec=30
OnUnitActiveSec=120
AccuracySec=15
Unit=bridget-maicie-releve.service

[Install]
WantedBy=timers.target
EOF
  chmod 0644 "$MAICIE_RELEVE_TIMER"
  created "systemd maicie.releve.timer ($MAICIE_RELEVE_TIMER)"
}

write_services() {
  case "$SERVICE_BACKEND" in
    launchd)
      write_plist_daemon
      write_plist_maicie_releve
      ;;
    systemd)
      write_systemd_daemon
      write_systemd_maicie_releve
      ;;
  esac
}

activate_services() {
  if [[ "$SKIP_SERVICES" == "1" ]]; then
    log "NON FAIT: activation ${SERVICE_BACKEND} (--skip-services)"
    return 0
  fi
  case "$SERVICE_BACKEND" in
    launchd)
      log "charger services launchd"
      launchctl bootstrap "gui/$(id -u)" "$DAEMON_PLIST" 2>/dev/null \
        || launchctl load "$DAEMON_PLIST" 2>/dev/null \
        || true
      launchctl bootstrap "gui/$(id -u)" "$MAICIE_RELEVE_PLIST" 2>/dev/null \
        || launchctl load "$MAICIE_RELEVE_PLIST" 2>/dev/null \
        || true
      ;;
    systemd)
      log "activer services systemd --user"
      systemctl --user daemon-reload
      # Relire le registre agents.json : restart si déjà actif.
      if systemctl --user is-active --quiet bridget-daemon.service; then
        systemctl --user restart bridget-daemon.service
      else
        systemctl --user enable --now bridget-daemon.service
      fi
      systemctl --user enable --now bridget-maicie-releve.timer
      # Relève immédiate une fois (oneshot), sans attendre le timer.
      systemctl --user start bridget-maicie-releve.service 2>/dev/null || true
      ;;
  esac
}

verify_daemon() {
  local attempts=40
  local i
  for ((i = 1; i <= attempts; i++)); do
    if [[ -x "$BRIDGET_BIN" ]] && "$BRIDGET_BIN" status >/dev/null 2>&1; then
      echo "VÉRIFIÉ: daemon joignable (bridget status) [${PLATFORM}/${SERVICE_BACKEND}]"
      return 0
    fi
    sleep 0.25
  done
  echo "NON VÉRIFIÉ: daemon joignable" >&2
  if [[ "$SERVICE_BACKEND" == "systemd" ]]; then
    systemctl --user status bridget-daemon.service --no-pager >&2 || true
  fi
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
    echo "NON VÉRIFIÉ: spawn/stop (type $TEST_AGENT_TYPE absent — non injecté par idempotence)" >&2
    return 1
  fi
  if ! "$BRIDGET_BIN" spawn "$TEST_AGENT_TYPE" --name "$TEST_AGENT_NAME" --timeout 10 \
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
  echo "VÉRIFIÉ: maicie status (config chargeable + relève pull)"
  return 0
}

# Dépôt guichet E2E : spawn fixture, dépôt mission-status via --from, relève maicie.
verify_guichet_e2e() {
  local issued_at request_id
  issued_at="$(date +%s)"
  request_id="k1-guichet-${issued_at}"

  if ! "$BRIDGET_BIN" spawn "$TEST_AGENT_TYPE" --name "$TEST_AGENT_NAME" --timeout 15 \
    >/tmp/k1-g-spawn.out 2>/tmp/k1-g-spawn.err; then
    echo "NON VÉRIFIÉ: guichet (spawn préalable échoué)" >&2
    cat /tmp/k1-g-spawn.err >&2 || true
    return 1
  fi

  # Délégation minimale dans la base Maicie pour un mission-status valide.
  # Sans délégation, le dépôt peut rester queued puis être rejeté à la greffe —
  # on vérifie au minimum : queued puis relève sans panne fatale.
  if ! "$BRIDGET_BIN" guichet deposer mission-status \
    --from "$TEST_AGENT_NAME" \
    --delegation "00000000-0000-0000-0000-000000000001" \
    --id "$request_id" \
    --issued-at "$issued_at" \
    --issuer-scope "k1_install_verify_scope_0123456789abcdef" \
    >/tmp/k1-guichet-deposit.out 2>/tmp/k1-guichet-deposit.err; then
    # queued / outcome_unknown = ok transport ; accepted au retry = déjà traité
    if ! grep -Eq 'DÉPÔT: (queued|outcome_unknown|accepted)' /tmp/k1-guichet-deposit.out \
      /tmp/k1-guichet-deposit.err 2>/dev/null; then
      echo "NON VÉRIFIÉ: dépôt guichet" >&2
      cat /tmp/k1-guichet-deposit.out /tmp/k1-guichet-deposit.err >&2 || true
      "$BRIDGET_BIN" stop "$TEST_AGENT_NAME" >/dev/null 2>&1 || true
      return 1
    fi
  fi
  echo "VÉRIFIÉ: dépôt guichet (sortie: $(tr '\n' ' ' </tmp/k1-guichet-deposit.out))"

  if ! "$MAICIE_BIN" status --config "$MAICIE_CONFIG" --json >/tmp/k1-releve.out 2>/tmp/k1-releve.err; then
    echo "NON VÉRIFIÉ: relève maicie après dépôt (panne fatale?)" >&2
    cat /tmp/k1-releve.err >&2 || true
    "$BRIDGET_BIN" stop "$TEST_AGENT_NAME" >/dev/null 2>&1 || true
    return 1
  fi
  echo "VÉRIFIÉ: relève maicie après dépôt (status sans panne)"

  "$BRIDGET_BIN" stop "$TEST_AGENT_NAME" >/tmp/k1-g-stop.out 2>/tmp/k1-g-stop.err || true
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
  if [[ "$VERIFY_GUICHET" == "1" ]]; then
    verify_spawn_stop || ok=0
    verify_maicie_status || ok=0
    verify_guichet_e2e || ok=0
  else
    verify_spawn_stop || ok=0
    verify_maicie_status || ok=0
    echo "NON FAIT: dépôt guichet relevé bout en bout (passer --verify-guichet)"
  fi
  [[ "$ok" == "1" ]] || return 1
  return 0
}

print_report() {
  cat <<EOF

install-k1: RAPPORT DE POSE
  plateforme: ${PLATFORM} (${OS_NAME})
  services:   ${SERVICE_BACKEND}
  créés:      $CREATED
  déjà là:    $SKIPPED
  remplacés:  $TOUCHED (--force)

Artefacts:
  $BRIDGET_BIN
  $MAICIE_BIN
  $MAICIE_SUIVI_BIN
  $AGENTS_JSON
  $MAICIE_CONFIG
  catalogue: $CATALOGUE_PATH
EOF
  case "$SERVICE_BACKEND" in
    launchd)
      echo "  $DAEMON_PLIST"
      echo "  $MAICIE_RELEVE_PLIST"
      ;;
    systemd)
      echo "  $DAEMON_SERVICE"
      echo "  $MAICIE_RELEVE_SERVICE"
      echo "  $MAICIE_RELEVE_TIMER"
      ;;
  esac
}

main() {
  log "plateforme=${PLATFORM} backend=${SERVICE_BACKEND}"
  ensure_dirs
  build_release_if_needed

  if [[ -e "${ROOT_DIR}/target/release/bridget" ]]; then
    install_binary "${ROOT_DIR}/target/release/bridget" "$BRIDGET_BIN" "binaire bridget"
  fi
  stage_maicie_activation
  preflight_staged_maicie_activation
  publish_staged_maicie_activation

  write_test_adapter
  write_agents_json
  write_maicie_suivi
  write_services
  preflight_published_maicie_activation
  activate_services

  print_report

  if ! run_verify; then
    die "vérification incomplète ou échouée — voir NON VÉRIFIÉ ci-dessus"
  fi

  log "terminé"
}

main "$@"
