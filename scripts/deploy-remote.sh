#!/bin/bash
# Déploie Bridget sur un hôte Linux distant.
# Le mode "client-only" est destiné à un hôte fédéré via federate-ssh.sh :
# il installe le client sans lancer de daemon concurrent.
set -e
REMOTE="${1:?Usage: deploy-remote.sh <utilisateur@hôte> [port] [daemon|client-only]}"
PORT="${2:-22}"
MODE="${3:-daemon}"
REMOTE_DIR="~/bridget"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CODEX_SKILL="$HOME/.codex/skills/bridget/SKILL.md"
CLAUDE_SKILL="$HOME/.claude/skills/bridget/SKILL.md"

case "$MODE" in
    daemon|client-only) ;;
    *) echo "Mode invalide : $MODE (daemon ou client-only)" >&2; exit 2 ;;
esac

echo "=== Déploiement bridget vers $REMOTE:$PORT ==="

# Le rsync exclut .git : sans cette variable, build.rs ne trouve aucun dépôt et
# retombe sur "unknown", ce qui fait afficher « daemon périmé » à CHAQUE commande
# sur toute machine déployée — en accusant le daemon alors que c'est le client
# qui n'est pas identifiable. `build.rs` lit BRIDGET_BUILD_ID en priorité.
BUILD_ID="$(git -C "$PROJECT_DIR" rev-parse --short=12 HEAD 2>/dev/null || true)"
if [[ -n "$BUILD_ID" ]] \
    && [[ -n "$(git -C "$PROJECT_DIR" status --porcelain --untracked-files=no 2>/dev/null)" ]]; then
    BUILD_ID="${BUILD_ID}-dirty"
fi
: "${BUILD_ID:=unknown}"
echo "→ Identité de compilation : $BUILD_ID"

echo "→ Synchronisation du code source..."
rsync -az --delete --exclude 'target' --exclude '.git' --exclude '*.db' --exclude '*.sock' \
    "$PROJECT_DIR/" -e "ssh -p $PORT" "$REMOTE:$REMOTE_DIR/"

echo "→ Vérification de Rust..."
ssh -p $PORT "$REMOTE" 'bash -s' << 'REMOTE_SCRIPT'
if ! command -v cargo &>/dev/null; then
    echo "  Installation de Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi
echo "  Rust: $(rustc --version)"
REMOTE_SCRIPT

echo "→ Compilation..."
ssh -p $PORT "$REMOTE" 'bash -s' -- "$BUILD_ID" << 'REMOTE_SCRIPT'
source "$HOME/.cargo/env" 2>/dev/null || true
cd ~/bridget
export BRIDGET_BUILD_ID="$1"
cargo build --release 2>&1 | tail -5
echo "  Binaire: $(ls -la target/release/bridget 2>/dev/null | awk '{print $5}') bytes"
echo "  Build-id: $BRIDGET_BUILD_ID"
REMOTE_SCRIPT

echo "→ Installation..."
ssh -p $PORT "$REMOTE" 'bash -s' << 'REMOTE_SCRIPT'
source "$HOME/.cargo/env" 2>/dev/null || true
mkdir -p ~/.local/bin
ln -sf ~/bridget/target/release/bridget ~/.local/bin/bridget
for shell_file in ~/.profile ~/.bashrc; do
    touch "$shell_file"
    grep -qxF 'export PATH="$HOME/.local/bin:$PATH"' "$shell_file" || \
        printf '\n# Bridget CLI\nexport PATH="$HOME/.local/bin:$PATH"\n' >> "$shell_file"
done
echo "  $(~/.local/bin/bridget version)"
REMOTE_SCRIPT

echo "→ Installation des skills Bridget pour les agents..."
ssh -p $PORT "$REMOTE" 'mkdir -p ~/.codex/skills/bridget ~/.claude/skills/bridget'
for skill in "$CODEX_SKILL" "$CLAUDE_SKILL"; do
    if [[ ! -f "$skill" ]]; then
        echo "Skill locale absente : $skill" >&2
        exit 1
    fi
done
rsync -az -e "ssh -p $PORT" "$CODEX_SKILL" "$REMOTE:~/.codex/skills/bridget/SKILL.md"
rsync -az -e "ssh -p $PORT" "$CLAUDE_SKILL" "$REMOTE:~/.claude/skills/bridget/SKILL.md"
echo "  Codex : ~/.codex/skills/bridget/SKILL.md"
echo "  Claude : ~/.claude/skills/bridget/SKILL.md"

if [[ "$MODE" == "client-only" ]]; then
    echo "=== Client Bridget installé (mode fédéré, aucun daemon distant) ==="
    exit 0
fi

echo "→ Configuration systemd utilisateur Linux..."
ssh -p $PORT "$REMOTE" 'bash -s' << 'REMOTE_SCRIPT'
mkdir -p ~/.config/systemd/user
cat > ~/.config/systemd/user/bridget-daemon.service << 'SERVICE'
[Unit]
Description=Bridget daemon
After=network.target
[Service]
Type=simple
ExecStart=%h/.local/bin/bridget daemon
Environment=RUST_LOG=info
Restart=always
RestartSec=3
[Install]
WantedBy=default.target
SERVICE
systemctl --user daemon-reload
systemctl --user enable bridget-daemon 2>/dev/null
systemctl --user start bridget-daemon 2>/dev/null || true
echo "  systemd: $(systemctl --user is-active bridget-daemon 2>/dev/null || echo 'n/a')"
REMOTE_SCRIPT

echo "=== Déploiement terminé ==="
echo "  Test: ssh -p $PORT $REMOTE 'bridget status'"
