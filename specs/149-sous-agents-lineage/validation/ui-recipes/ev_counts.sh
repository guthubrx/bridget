#!/bin/zsh
# Recette 149 r3 - événements d'orchestration par type depuis un numéro de séquence (DB PRIVÉE, lecture seule).
# Usage : ev_counts.sh [seq_depart]   -> "max=N types: thread.created=a subagent.updated=b thread.metadata-updated=c ..."
DB="${RECIPE149_CACHE:-/Users/moi/.cache/bridget149-ui}/t3home/userdata/statev2.sqlite"
[[ "$DB" == *bridget149-ui* ]] || { echo "refus : DB hors cache de recette" >&2; exit 2; }
SINCE="${1:-0}"
MAX="$(sqlite3 -readonly "file:$DB?mode=ro" "select coalesce(max(sequence),0) from orchestration_events")"
BODY="$(sqlite3 -readonly "file:$DB?mode=ro" "select event_type||'='||count(*) from orchestration_events where sequence > $SINCE group by event_type order by event_type" | paste -sd' ' -)"
echo "max=$MAX delta_depuis_$SINCE: ${BODY:-aucun}"
