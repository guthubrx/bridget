#!/usr/bin/env bash
# Relance propre du service Bridget, du relais d'interface et du référent.
#
#   bridget-relance.sh            relance complète
#   bridget-relance.sh --etat     diagnostic seul, n'écrit rien
#   bridget-relance.sh --sans-referent   service et relais uniquement
#
# Chaque étape est vérifiée par son EFFET, jamais par la présence d'un
# processus. Écrit le 28/08/2026 après une relance qui a demandé sept
# tentatives ; chaque garde ci-dessous correspond à un piège rencontré ce
# jour-là, et non à un risque imaginé.

set -uo pipefail

BIN="${BRIDGET_BIN:-$HOME/.local/bin/bridget}"
SOURCE="${BRIDGET_SOURCE:-$HOME/bridget-referent/bridget}"
CFG_MAICIE="${MAICIE_CONFIG:-$HOME/.config/maicie/config.json}"
CFG_AGENTS="${BRIDGET_AGENTS:-$HOME/.config/bridget/agents.json}"
CACHE="${BRIDGET_CACHE:-$HOME/.cache/bridget}"
LOGS="$HOME/.local/share"
REFERENT="${BRIDGET_REFERENT_NAME:-bridget}"
PORT_UI=17888

etat_seul=0; sans_referent=0
for a in "$@"; do
  case "$a" in
    --etat) etat_seul=1 ;;
    --sans-referent) sans_referent=1 ;;
    *) echo "option inconnue : $a" >&2; exit 2 ;;
  esac
done

titre() { printf '\n=== %s\n' "$1"; }
ok()    { printf '  OK    %s\n' "$1"; }
ko()    { printf '  ECHEC %s\n' "$1"; }
info()  { printf '        %s\n' "$1"; }

service_repond() { timeout 10 "$BIN" who >/dev/null 2>&1; }

# Liste les daemons par leur binaire réel. `pgrep -f` est proscrit ici : le
# motif attrape la commande elle-même et fait tuer son propre shell — c'est
# arrivé le 28/08.
daemons() {
  for p in $(pgrep -x bridget 2>/dev/null); do
    local exe; exe=$(readlink "/proc/$p/exe" 2>/dev/null) || continue
    [[ "$(tr -d '\0' < "/proc/$p/cmdline" 2>/dev/null)" == *daemon* ]] && echo "$p|$exe"
  done
}

# ---------------------------------------------------------------- 1. état
titre "1. ÉTAT INITIAL"
service_repond && ok "le service répond" || ko "le service ne répond pas"
mapfile -t D < <(daemons)
if ((${#D[@]})); then
  for e in "${D[@]}"; do
    p="${e%%|*}"; exe="${e#*|}"
    if [[ "$exe" == /tmp/* || "$exe" == *"/debug/"* ]]; then
      info "pid $p : $exe   <-- DAEMON DE TEST, il squatte la place"
    else
      info "pid $p : $exe"
    fi
  done
else
  info "aucun daemon"
fi

# Piège mesuré : un daemon de test compilé par un agent dans /tmp a occupé la
# socket de production pendant DIX-SEPT HEURES. Toutes les relances échouaient
# en silence, et la version affichée ne changeait jamais quoi qu'on installe.
titre "2. BINAIRE"
if [[ -x "$SOURCE/target/release/bridget" ]]; then
  a=$(stat -c %Y "$BIN" 2>/dev/null || echo 0)
  b=$(stat -c %Y "$SOURCE/target/release/bridget" 2>/dev/null || echo 0)
  if (( b > a )); then
    info "un binaire plus récent existe : $SOURCE/target/release/bridget"
    info "compilé le $(date -d "@$b" '+%d/%m %H:%M' 2>/dev/null)"
  else
    ok "le binaire installé est le plus récent"
  fi
fi

# Piège : l'entrée d'un agent sans `capabilities` fait refuser tout lancement
# avec « capacité manquante », sur les binaires postérieurs au 28/08. Une
# entrée écrite à la main pendant une bascule omet facilement ce champ.
titre "3. CONFIGURATION DES AGENTS"
python3 - "$CFG_AGENTS" <<'PY' 2>/dev/null || info "configuration illisible"
import json, sys
d = json.load(open(sys.argv[1])).get('agents', {})
for nom, a in d.items():
    proto = a.get('protocol')
    caps = (a.get('capabilities') or {}).get('execution_paths') or []
    etat = "OK   " if proto in caps else "ECHEC"
    print(f"  {etat} {nom} : protocole {proto}, capacités {caps or 'ABSENTES'}")
PY

if (( etat_seul )); then
  titre "DIAGNOSTIC SEUL — rien n'a été modifié"
  exit 0
fi

# ---------------------------------------------------------------- arrêt
titre "4. ARRÊT"
mapfile -t D < <(daemons)
for e in "${D[@]}"; do
  p="${e%%|*}"
  kill "$p" 2>/dev/null
done
for i in 1 2 3 4; do
  sleep 3
  mapfile -t D < <(daemons)
  ((${#D[@]})) || { ok "arrêté après $((i*3)) s"; break; }
done
mapfile -t D < <(daemons)
if ((${#D[@]})); then
  # Défaut connu : les binaires antérieurs au correctif de terminaison ne
  # traitent pas le signal poli. Le signal fort est alors le seul recours,
  # et il est délibéré — pas un réflexe.
  info "résiste au signal poli après 12 s — signal fort"
  for e in "${D[@]}"; do kill -9 "${e%%|*}" 2>/dev/null; done
  sleep 3
fi
mapfile -t D < <(daemons)
((${#D[@]})) && { ko "un daemon survit, abandon"; exit 1; } || ok "plus aucun daemon"

# Le verrou survit au processus et fait échouer la relance avec « un daemon
# tourne déjà », alors que plus rien ne tourne.
rm -f "$CACHE/bridget.sock" "$CACHE/bridget.pid" 2>/dev/null
ok "socket et verrou nettoyés"

# ---------------------------------------------------------------- binaire
titre "5. INSTALLATION DU BINAIRE"
if [[ -x "$SOURCE/target/release/bridget" ]]; then
  a=$(stat -c %Y "$BIN" 2>/dev/null || echo 0)
  b=$(stat -c %Y "$SOURCE/target/release/bridget" 2>/dev/null || echo 0)
  if (( b > a )); then
    # `cp` sur un binaire encore mappé par un autre processus rend
    # « Text file busy » ; le déplacer d'abord contourne sans gêner personne.
    mv "$BIN" "$BIN.avant-$(date +%d%b)" 2>/dev/null
    cp "$SOURCE/target/release/bridget" "$BIN" && chmod +x "$BIN" \
      && ok "binaire mis à jour" || { ko "copie impossible"; exit 1; }
  else
    ok "binaire déjà à jour"
  fi
fi

# ---------------------------------------------------------------- service
titre "6. SERVICE"
setsid nohup "$BIN" daemon > "$LOGS/bridget-daemon.log" 2>&1 < /dev/null &
for i in 1 2 3 4 5; do
  sleep 3
  service_repond && break
done
if service_repond; then
  ok "le service répond"
  p=$(daemons | head -1); info "binaire réel : ${p#*|}"
  info "$(timeout 10 "$BIN" who 2>/dev/null | grep -i 'build-id' || true)"
else
  ko "le service ne démarre pas"
  tail -3 "$LOGS/bridget-daemon.log" 2>/dev/null | sed 's/^/        /'
  exit 1
fi

# ---------------------------------------------------------------- relais
titre "7. RELAIS D'INTERFACE"
# Le relais meurt avec le service et ne se relance pas seul. Il doit être
# détaché de la session : rattaché, il survit mais perd sa liaison, et
# affiche alors « daemon indisponible » alors que tout va bien.
if ss -ltn 2>/dev/null | grep -q ":$PORT_UI"; then
  ok "déjà en écoute"
else
  setsid nohup "$BIN" ui --maicie-config "$CFG_MAICIE" > "$LOGS/bridget-ui.log" 2>&1 < /dev/null &
  sleep 6
  ss -ltn 2>/dev/null | grep -q ":$PORT_UI" && ok "relancé" || ko "n'écoute pas"
fi
grep -o 'http://[^ ]*' "$LOGS/bridget-ui.log" 2>/dev/null | tail -1 | sed 's/^/        /'

# ---------------------------------------------------------------- référent
if (( ! sans_referent )); then
  titre "8. RÉFÉRENT"
  if timeout 10 "$BIN" who 2>/dev/null | grep -qE "^ *$REFERENT "; then
    ok "déjà connecté"
  else
    setsid nohup "$BIN" spawn claude --name "$REFERENT" --persistent \
      > "$LOGS/bridget-referent.log" 2>&1 < /dev/null &
    sleep 18
    if timeout 10 "$BIN" who 2>/dev/null | grep -qE "^ *$REFERENT "; then
      ok "connecté"
    else
      ko "non connecté"
      tail -2 "$LOGS/bridget-referent.log" 2>/dev/null | sed 's/^/        /'
    fi
  fi
fi

# ---------------------------------------------------------------- bilan
titre "9. BILAN"
n=$(timeout 10 "$BIN" who 2>/dev/null | tail -n +2 | grep -c . || echo 0)
info "$n interlocuteur(s) à l'annuaire"
# Défaut ouvert au 28/08 : les agents lancés avant l'arrêt gardent leur
# contexte et peuvent émettre, mais ne se réinscrivent pas à l'annuaire.
# Ils redeviennent injoignables sans être perdus. Les relancer coûterait
# plusieurs heures de contexte accumulé — ne pas le faire par réflexe.
if (( n < 3 )); then
  info "les agents lancés avant l'arrêt ne se réinscrivent pas :"
  info "ils travaillent et peuvent émettre, mais ne sont pas adressables."
  info "voir priorite-humaine/un-agent-peut-emettre-sans-exister-a-l-annuaire"
fi
