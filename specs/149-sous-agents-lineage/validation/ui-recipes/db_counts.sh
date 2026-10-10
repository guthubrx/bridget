#!/bin/zsh
# Recette 149 - compteurs lecture seule de la DB PRIVÉE de recette (jamais la production).
# Un compteur de run/tour/session fournisseur qui bouge = exécution T3 produite par la vue.
DB="${RECIPE149_CACHE:-/Users/moi/.cache/bridget149-ui}/t3home/userdata/statev2.sqlite"
[[ "$DB" == *bridget149-ui* ]] || { echo "refus : DB hors cache de recette" >&2; exit 2; }
for t in orchestration_v2_projection_threads orchestration_v2_projection_runs orchestration_v2_projection_run_attempts orchestration_v2_projection_provider_turns orchestration_v2_projection_provider_sessions orchestration_v2_projection_provider_threads orchestration_v2_projection_provider_session_bindings orchestration_v2_projection_runtime_requests orchestration_v2_effect_outbox orchestration_v2_thread_launch_workflows projection_thread_messages; do
  printf '%s=%s\n' "${t#orchestration_v2_}" "$(sqlite3 -readonly "file:$DB?mode=ro" "select count(*) from $t")"
done | paste -sd' ' -
