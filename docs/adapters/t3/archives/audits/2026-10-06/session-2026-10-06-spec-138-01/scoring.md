# Archives techniques de l’adaptateur T3

Document d’origine : [audits/2026-10-06/session-2026-10-06-spec-138-01/scoring.md](../../../../../../../audits/2026-10-06/session-2026-10-06-spec-138-01/scoring.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Fichier : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/t3code_contract.rs:271

<a id="extrait-2"></a>

## Extrait 2

- 03 — Le projet source est issu du fait de connexion ; from et domain ne font pas autorité (daemon.rs / communication.rs).
- 04 — parse_snapshot indexe les projets et consulte chaque thread par HashMap ; index attendu O(P+T) (t3code_contract.rs:279).
- 05 — La copie Claude délègue au moteur canonique au lieu de recopier le moteur (scope externe claude/.claude/skills/agent-loop/scripts/agent_loop.py).
- 06 — Le contrôle conserve les warnings dans le résultat durable ; un replay accepté ne réexécute pas l'effet (execution_store.rs / spec138_project_test.rs).
- 06 — CAS, archive et publication partagent le verrou de tâche ; les tests couvrent course et pannes E/S ordinaires (agent_loop.py:638).
- 07 — Aucun bloc de 100 lignes dans les deux rapports JSCPD ; les taux sont des mesures de contexte, pas du diff.
- 12 — Cargo.toml/Cargo.lock et les manifests des crates restent inchangés ; aucun nouveau service, registre ou framework.
