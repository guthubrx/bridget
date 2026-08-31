PASS

J’ai relu uniquement les deux correctifs du diff actuel.

- `decommissioning_names` : je ne vois pas de faille `P0/P1/P2` restante.
- Pourquoi : le nom est pris avant l’arrêt/décommission (`insert`), la relance le refuse immédiatement via `contains`, puis le verrou est libéré sur toutes les issues opérationnelles après acquisition :
  `/home/moi/bridget-referent/.worktrees/session-075-cycle-vie-agents/crates/bridget-daemon/src/daemon.rs:8479`
  `/home/moi/bridget-referent/.worktrees/session-075-cycle-vie-agents/crates/bridget-daemon/src/daemon.rs:8512`
  `/home/moi/bridget-referent/.worktrees/session-075-cycle-vie-agents/crates/bridget-daemon/src/daemon.rs:8527`
  `/home/moi/bridget-referent/.worktrees/session-075-cycle-vie-agents/crates/bridget-daemon/src/daemon.rs:8534`
  `/home/moi/bridget-referent/.worktrees/session-075-cycle-vie-agents/crates/bridget-daemon/src/daemon.rs:8542`
  `/home/moi/bridget-referent/.worktrees/session-075-cycle-vie-agents/crates/bridget-daemon/src/daemon.rs:8550`
- Le refus de relance concurrente est bien couvert ici :
  `/home/moi/bridget-referent/.worktrees/session-075-cycle-vie-agents/crates/bridget-daemon/src/daemon.rs:8261`

- `buildAgentLifecycleUrl` : je ne vois pas non plus de faille `P0/P1/P2` restante.
- Pourquoi : la route est résolue depuis une table fermée, et toute action inconnue lève explicitement une erreur au lieu de retomber sur `stop` :
  `/home/moi/bridget-referent/.worktrees/session-075-cycle-vie-agents/crates/bridget-daemon/assets/ui/app.js:3125`
- Le test de non-régression existe bien :
  `/home/moi/bridget-referent/.worktrees/session-075-cycle-vie-agents/crates/bridget-daemon/assets/ui/app.js:284`

Verdict global : les deux correctifs ciblés me paraissent sains, sans faille restante de sévérité `P0/P1/P2`.