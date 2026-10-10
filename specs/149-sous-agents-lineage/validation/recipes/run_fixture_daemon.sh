#!/bin/zsh
# run_fixture_daemon.sh — Lance le daemon fixture 149 (binaire compilé) hors prod.
# BRIDGET_HOME et BRIDGET_SOCKET privés (0700) ; le socket vit directement dans
# BRIDGET_HOME (exigence de Namespace::from_environment). Aucun vecteur T3 n'est
# exporté : la voie reste NativeDelegation hors T3 (T038).
# Arrêt : SIGTERM au PID enregistré, jamais -9 (cf. protection processus).
set -u

RECIPE_DIR="${0:A:h}"
FIXTURE_ROOT="${BRIDGET_149_FIXTURE_ROOT:-/Users/moi/.cache/bridget149-recipe}"
BIN149="${BRIDGET_149_BIN:?exporter BRIDGET_149_BIN (binaire 149 compilé)}"

umask 077
HOME_FIXTURE="$FIXTURE_ROOT/home"
mkdir -p "$HOME_FIXTURE" "$FIXTURE_ROOT/logs"
chmod 700 "$HOME_FIXTURE" "$FIXTURE_ROOT/logs" 2>/dev/null

# Refus de recouvrir la prod ou l'autre registre.
[[ "$HOME_FIXTURE" != "/Users/moi/.cache/bridget-core" ]] || { print -u2 "refus : fixture = prod"; exit 2; }
[[ ! -e "$HOME_FIXTURE/bridget.sock" ]] || {
  print -u2 "refus : socket fixture déjà présent ($HOME_FIXTURE/bridget.sock) — daemon déjà lancé ?"
  exit 2
}

export BRIDGET_HOME="$HOME_FIXTURE"
export BRIDGET_SOCKET="$HOME_FIXTURE/bridget.sock"
unset BRIDGET_T3_MCP_ENDPOINT BRIDGET_T3_MCP_AUTHORIZATION

PIDFILE="$FIXTURE_ROOT/state/daemon.pid"
LOG="$FIXTURE_ROOT/logs/daemon-stderr.log"
"$BIN149" daemon </dev/null >>"$LOG" 2>&1 &
PID=$!
print "$PID" > "$PIDFILE"

# Attente bornée du socket (10 s max), sans boucle infinie.
for i in {1..20}; do
  [[ -S "$BRIDGET_SOCKET" ]] && break
  kill -0 "$PID" 2>/dev/null || { print -u2 "daemon mort ; voir $LOG"; exit 1; }
  sleep 0.5
done
[[ -S "$BRIDGET_SOCKET" ]] || { print -u2 "socket absent après 10 s ; voir $LOG"; exit 1; }

print "OK daemon fixture pid=$PID socket=$BRIDGET_SOCKET log=$LOG"
print "Arrêt : kill \$(cat $PIDFILE) puis attendre la libération du socket."
