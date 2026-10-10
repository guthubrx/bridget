#!/bin/zsh
# Recette UI 149 — lance le serveur T3 réel sur base privée + daemon CLI fixture.
# Aucune app T3 Desktop, aucune base/config/secret de production.
# Usage : zsh run_recipe_server.sh start|stop|status
set -euo pipefail

WT="/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage"
NODE_BIN="/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin"
CACHE="${RECIPE149_CACHE:-/Users/moi/.cache/bridget149-ui}"
HERE="$(cd "$(dirname "$0")" && pwd)"
PIDFILE="$CACHE/server.pid"
LOG="$CACHE/logs/server.log"

export PATH="$NODE_BIN:$PATH"
export T3CODE_PORT="${RECIPE149_T3_PORT:-14773}"
export T3CODE_HOME="$CACHE/t3home"
export T3CODE_MODE=web
export T3CODE_NO_BROWSER=1
# jeton de recette local uniquement (fichier 0600 du cache, jamais logué)
if [[ ! -f "$CACHE/dev-auth-token" ]]; then
  dd if=/dev/urandom bs=32 count=1 2>/dev/null | od -An -tx1 | tr -d ' \n' > "$CACHE/dev-auth-token"
  chmod 600 "$CACHE/dev-auth-token"
fi
export T3CODE_DEV_AUTH_TOKEN="$(cat "$CACHE/dev-auth-token")"
export T3CODE_DEV_ALLOWED_ORIGINS="http://localhost:${RECIPE149_WEB_PORT:-15733}"
# daemon fixture : CLI Node du magasin privé (seule couche simulée, nommée)
export T3CODE_BRIDGET_EXECUTABLE="$HERE/bridget_fixture.mjs"
export BRIDGET149_UI_STORE="$CACHE/store.json"

case "${1:-start}" in
  start)
    if [[ -f "$PIDFILE" ]] && kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
      echo "serveur déjà lancé (pid $(cat "$PIDFILE"))"
      exit 0
    fi
    node "$WT/apps/server/src/bin.ts" >> "$LOG" 2>&1 &
    echo $! > "$PIDFILE"
    echo "serveur T3 recette lancé : pid $(cat "$PIDFILE"), port $T3CODE_PORT, log $LOG"
    ;;
  stop)
    if [[ -f "$PIDFILE" ]]; then
      PID="$(cat "$PIDFILE")"
      if kill -0 "$PID" 2>/dev/null; then
        # un seul PID owned, SIGTERM sans -9, vérification après 3s
        kill "$PID"
        sleep 3
        if kill -0 "$PID" 2>/dev/null; then
          echo "SIGTERM sans effet après 3s : pid $PID toujours vivant — intervention humaine requise"
          exit 1
        fi
      fi
      rm -f "$PIDFILE"
      echo "serveur arrêté"
    else
      echo "aucun pidfile"
    fi
    ;;
  status)
    if [[ -f "$PIDFILE" ]] && kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
      echo "actif (pid $(cat "$PIDFILE"))"
    else
      echo "inactif"
    fi
    ;;
  *)
    echo "usage: run_recipe_server.sh start|stop|status" >&2
    exit 2
    ;;
esac
