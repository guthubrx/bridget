#!/usr/bin/env bash
# Ouvre la vue Bridget distante à travers deux connexions SSH locales :
# le relais HTTP et son jeton restent liés à 127.0.0.1 sur l'hôte distant,
# puis le forward local reste lié à 127.0.0.1 sur cette machine.
set -euo pipefail

usage() {
    cat <<'EOF'
Usage: open-remote-ui.sh utilisateur@hote --maicie-config CHEMIN_ABSOLU [--port PORT] [--identity FICHIER] [--local-port PORT]

Ouvre une vue Bridget strictement en lecture seule via un tunnel SSH local.
Le chemin --maicie-config est interprété sur l'hôte distant.
EOF
}

target=${1:-}
[[ -n "$target" && "$target" != -* ]] || { usage >&2; exit 2; }
shift

maicie_config=""
ssh_port=22
identity=""
local_port=18777
while [[ $# -gt 0 ]]; do
    case "$1" in
        --maicie-config) maicie_config=${2:-}; shift 2 ;;
        --port) ssh_port=${2:-}; shift 2 ;;
        --identity) identity=${2:-}; shift 2 ;;
        --local-port) local_port=${2:-}; shift 2 ;;
        *) usage >&2; exit 2 ;;
    esac
done

[[ "$maicie_config" == /* ]] || {
    echo "--maicie-config doit être un chemin absolu sur l'hôte distant" >&2
    exit 2
}
[[ "$ssh_port" =~ ^[1-9][0-9]{0,4}$ && "$ssh_port" -le 65535 ]] || {
    echo "port SSH invalide" >&2
    exit 2
}
[[ "$local_port" =~ ^[1-9][0-9]{0,4}$ && "$local_port" -le 65535 ]] || {
    echo "port local invalide" >&2
    exit 2
}

ssh_args=(-p "$ssh_port" -o BatchMode=yes -o ExitOnForwardFailure=yes)
[[ -n "$identity" ]] && ssh_args+=(-i "$identity")

umask 077
temporary_dir=$(mktemp -d -t bridget-ui-tunnel.XXXXXX)
relay_log="$temporary_dir/relay.log"
relay_pid=""
forward_pid=""

cleanup() {
    for pid in "$forward_pid" "$relay_pid"; do
        if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
            wait "$pid" 2>/dev/null || true
        fi
    done
    rm -rf "$temporary_dir"
}
trap cleanup EXIT INT TERM

# printf %q conserve le chemin comme un argument du shell distant ; aucun texte
# local ne peut devenir une sous-commande distante.
printf -v remote_command 'exec %q ui --maicie-config %q' bridget "$maicie_config"
ssh "${ssh_args[@]}" "$target" "$remote_command" >"$relay_log" 2>&1 &
relay_pid=$!

remote_url=""
for _ in $(seq 1 100); do
    remote_url=$(sed -n 's#^Bridget UI (lecture seule) : \(http://127\.0\.0\.1:[0-9][0-9]*/?token=[A-Za-z0-9_-][A-Za-z0-9_-]*\)$#\1#p' "$relay_log" | head -1)
    [[ -n "$remote_url" ]] && break
    if ! kill -0 "$relay_pid" 2>/dev/null; then
        sed -n '1,80p' "$relay_log" >&2
        echo "le relais UI distant s'est arrêté avant de publier son URL" >&2
        exit 1
    fi
    sleep 0.1
done
[[ -n "$remote_url" ]] || {
    echo "délai dépassé en attente de l'URL du relais UI distant" >&2
    exit 1
}

remote_port=${remote_url#http://127.0.0.1:}
remote_port=${remote_port%%/*}
token=${remote_url#*'?token='}
[[ "$remote_port" =~ ^[1-9][0-9]{0,4}$ && "$remote_port" -le 65535 ]] || {
    echo "port du relais distant invalide" >&2
    exit 1
}

ssh "${ssh_args[@]}" -N -L "127.0.0.1:${local_port}:127.0.0.1:${remote_port}" "$target" &
forward_pid=$!
sleep 0.2
if ! kill -0 "$forward_pid" 2>/dev/null; then
    echo "le tunnel SSH local n'a pas pu démarrer" >&2
    exit 1
fi

local_url="http://127.0.0.1:${local_port}/?token=${token}"
printf 'Vue distante Bridget (lecture seule) : %s\n' "$local_url"
printf 'Le tunnel reste attaché à ce terminal ; Ctrl-C le ferme.\n'
if command -v open >/dev/null 2>&1; then
    open "$local_url"
elif command -v xdg-open >/dev/null 2>&1; then
    xdg-open "$local_url" >/dev/null 2>&1 || true
fi

while kill -0 "$relay_pid" 2>/dev/null && kill -0 "$forward_pid" 2>/dev/null; do
    sleep 1
done

echo "le tunnel UI a été fermé" >&2
exit 1
