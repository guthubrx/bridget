#!/bin/bash
# Déploie et compile bridget sur un serveur Linux distant.
set -e
REMOTE="${1:?Usage: deploy-remote.sh <host> [port]}"
PORT="${2:-22}"
REMOTE_DIR="~/bridget"

echo "=== Déploiement bridget vers $REMOTE:$PORT ==="

echo "→ Synchronisation du code source..."
rsync -az --delete --exclude 'target' --exclude '.git' --exclude '*.db' --exclude '*.sock' \
    "$(dirname "$0")/.." -e "ssh -p $PORT" "$REMOTE:$REMOTE_DIR/"

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
ssh -p $PORT "$REMOTE" 'bash -s' << 'REMOTE_SCRIPT'
source "$HOME/.cargo/env" 2>/dev/null || true
cd ~/bridget
cargo build --release 2>&1 | tail -5
echo "  Binaire: $(ls -la target/release/bridget 2>/dev/null | awk '{print $5}') bytes"
REMOTE_SCRIPT

echo "→ Installation..."
ssh -p $PORT "$REMOTE" 'bash -s' << 'REMOTE_SCRIPT'
source "$HOME/.cargo/env" 2>/dev/null || true
mkdir -p ~/.local/bin
ln -sf ~/bridget/target/release/bridget ~/.local/bin/bridget
echo "  $(~/.local/bin/bridget version)"
REMOTE_SCRIPT

echo "→ Configuration systemd..."
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
