# Plan 136 — Étendre les fils existants

## Contexte technique

Rust, serde, SQLite/rusqlite. Pas de nouvelle dépendance. Réutiliser fils 102 :
entrées immuables, dépôts idempotents, sollicitations coalescées, reçus de lecture.
Le ledger et les envois directs restent inchangés.

## Architecture et réutilisation

1. Étendre ThreadAction::Post (crates/bridget-transport/src/protocol.rs) avec kind
   optionnel history/action/blocker/decision et supersedes_seq optionnel.
   Omission des deux: canon et comportement legacy strictement identiques.
2. threads.rs: history impose notify:[] et aucun remplacement ; autres classes
   body ≤ 2048 octets ; remplacement exige classe déclarée.
3. store/threads.rs: deux colonnes nullable discussion_entries, index unique
   partiel (thread_id,supersedes_seq). Pas de table supplémentaire. Migration transactionnelle.
4. thread_post valide sous transaction: cible antérieure, même auteur et même
   audience effective, pas history, pas déjà remplacée. Refus avant tout ACK.
   all et targets sont équivalents si leurs ensembles effectifs sont identiques.
   La source est notify_json.targets déjà figée au dépôt ; pas les wakes ni
   les membres recalculés. Les membres d'un fil102 sont immuables.
5. read_range: history garde les corps ; read projette une référence sans body
   pour history et pour ordre remplacé au snapshot_seq du reçu. Rejeu stable
   après nouveau dépôt. La confirmation porte sur cette projection, pas la mission.
   Relire une référence avec l'action history(thread_id,from_seq,to_seq),
   selon history_ref, sans déplacer le repère ; droits/pages102 conservés.
6. CLI thread post --kind/--supersedes, schéma MCP identique. Décrire la règle
   dans send/thread, enveloppe de sollicitation et documentation du projet.
7. Recette vrai daemon isolé ; transmettre la règle aux coordinateurs.
   Aucun tri ni conversion automatique des 64 anciens messages libres.

Read: O(log E + P log E), P ≤ 200, jointure indexée sur successeur unique,
sans scan des corps ni requête par entrée. Post: O(log E + M), M ≤ 16.

## Constitution Check

PASS: session approuvée, worktree privé, tests avant code, accès membre,
pas d'appel payant, état dérivé calculé, aucune duplication du contrôleur 135.
Enum partagé et deux colonnes justifiés par CLI/MCP/store ; pas de framework.
Charge future réduite: projection explicable plutôt que classement IA.

## Tests et livraison

Gherkin 136 avant code, tests dans modules existants. RED réel puis GREEN:
silence, histoire exacte, remplacements, refus atomiques, canon legacy,
frontière UTF-8 2048, snapshot/rejeu, pagination, migration/restart, CLI/MCP,
coalescence. Suite Rust, fmt, clippy -D warnings, release isolé.
Contre-revue plan et implementation, Analyze, Converge, audit v14.
Livraison: backup base/binaire/services, recette, commit/fusion/push périmètre136,
remplacement atomique binaire, redémarrage contrôlé daemon/pont, contrôle version
et identités, règle communiquée. Préserver worktree135 ; rollback documenté.

## Adaptateurs absents

Templates/scripts SpecKit officiels absents, vérifiés par lecture. Skills
utilisateur disponibles: application directe de leurs protocoles, pas de faux
runtime. Sync exécuté 2026-10-06 ; sorties générées hors besoin exclues du commit.
