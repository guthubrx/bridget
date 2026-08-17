#!/usr/bin/env bash
# Enrôle une machine distante dans le daemon Bridget local via SSH.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: federate-ssh.sh install|status|remove NAME [--host HOST] [--user USER] [--port PORT] [--identity FILE] [--remote-socket PATH]

Le tunnel reverse rend le socket du daemon maître disponible sur la machine distante.
EOF
}

action=${1:-}
name=${2:-}
shift $(( $# >= 2 ? 2 : $# ))
[[ -n "$action" && -n "$name" ]] || { usage >&2; exit 2; }

host=""; user="${USER}"; port=22; identity=""; remote_socket=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --host) host=${2:?}; shift 2 ;;
    --user) user=${2:?}; shift 2 ;;
    --port) port=${2:?}; shift 2 ;;
    --identity) identity=${2:?}; shift 2 ;;
    --remote-socket) remote_socket=${2:?}; shift 2 ;;
    *) usage >&2; exit 2 ;;
  esac
done

[[ "$name" =~ ^[A-Za-z0-9_-]+$ ]] || { echo "nom d'enrôlement invalide" >&2; exit 2; }
config_dir="${XDG_CONFIG_HOME:-$HOME/.config}/bridget/federation"
plist="$HOME/Library/LaunchAgents/com.bridget.federation.${name}.plist"
config="$config_dir/${name}.env"
local_socket="$HOME/.cache/bridget/bridget.sock"
label="com.bridget.federation.${name}"

# Un tunnel durable ne doit pas réutiliser un ControlMaster interactif : celui-ci
# peut refuser un nouveau reverse forward sans que launchd puisse le rétablir.
ssh_args=(-p "$port" -o BatchMode=yes -o ControlMaster=no -o ControlPath=none -o ExitOnForwardFailure=yes -o ServerAliveInterval=20 -o ServerAliveCountMax=3)
[[ -n "$identity" ]] && ssh_args+=(-i "$identity")
target="${user}@${host}"

case "$action" in
  install)
    [[ -n "$host" ]] || { echo "--host est requis pour l'installation" >&2; exit 2; }
    [[ -S "$local_socket" ]] || { echo "daemon maître absent : $local_socket" >&2; exit 1; }
    if [[ -z "$remote_socket" ]]; then
      remote_home=$(ssh "${ssh_args[@]}" "$target" 'printf %s "$HOME"')
      [[ "$remote_home" == /* ]] || { echo "HOME distant invalide : $remote_home" >&2; exit 1; }
      remote_socket="$remote_home/.cache/bridget/bridget.sock"
    fi
    remote_dir=$(dirname "$remote_socket")
    ssh "${ssh_args[@]}" "$target" bash -s -- "$remote_dir" "$remote_socket" <<'REMOTE_SCRIPT'
mkdir -p "$1"
rm -f "$2"
mkdir -p "$HOME/.config/bridget"
printf 'transport=ssh-unix\n' > "$HOME/.config/bridget/federation.env"
REMOTE_SCRIPT
    mkdir -p "$config_dir"
    umask 077
    printf 'host=%q\nuser=%q\nport=%q\nidentity=%q\nremote_socket=%q\nlocal_socket=%q\n' "$host" "$user" "$port" "$identity" "$remote_socket" "$local_socket" > "$config"
    cat > "$plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict><key>Label</key><string>$label</string><key>ProgramArguments</key><array><string>/usr/bin/ssh</string><string>-N</string><string>-p</string><string>$port</string><string>-o</string><string>BatchMode=yes</string><string>-o</string><string>ControlMaster=no</string><string>-o</string><string>ControlPath=none</string><string>-o</string><string>ExitOnForwardFailure=yes</string><string>-o</string><string>ServerAliveInterval=20</string><string>-o</string><string>ServerAliveCountMax=3</string>$( [[ -n "$identity" ]] && printf '<string>-i</string><string>%s</string>' "$identity" )<string>-R</string><string>$remote_socket:$local_socket</string><string>$target</string></array><key>KeepAlive</key><true/><key>RunAtLoad</key><true/><key>StandardOutPath</key><string>$HOME/Library/Logs/Bridget/federation-$name.log</string><key>StandardErrorPath</key><string>$HOME/Library/Logs/Bridget/federation-$name.err</string></dict></plist>
EOF
    mkdir -p "$HOME/Library/Logs/Bridget"
    launchctl bootout "gui/$(id -u)/$label" 2>/dev/null || true
    launchctl bootstrap "gui/$(id -u)" "$plist"
    for _ in {1..5}; do
      ssh "${ssh_args[@]}" "$target" "test -S '$remote_socket'" && break
      sleep 1
    done
    if ! ssh "${ssh_args[@]}" "$target" "test -S '$remote_socket'"; then
      launchctl bootout "gui/$(id -u)/$label" 2>/dev/null || true
      echo "échec d'activation du tunnel ; vérifier AllowTcpForwarding remote (ou yes) dans sshd sur $host" >&2
      exit 1
    fi
    echo "enrôlé : $name ($target:$port)"
    ;;
  status)
    [[ -f "$plist" ]] || { echo "inconnu : $name" >&2; exit 1; }
    launchctl print "gui/$(id -u)/$label" | sed -n '1,32p'
    ;;
  remove)
    launchctl bootout "gui/$(id -u)/$label" 2>/dev/null || true
    rm -f "$plist" "$config"
    echo "retiré : $name"
    ;;
  *) usage >&2; exit 2 ;;
esac
