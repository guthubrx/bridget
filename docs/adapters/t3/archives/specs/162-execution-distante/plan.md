# Archives techniques de l’adaptateur T3

Document d’origine : [specs/162-execution-distante/plan.md](../../../../../../specs/162-execution-distante/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Audit de réemploi et contre-revue avant tasks.md ; Analyze ensuite. Tests ciblés sur les risques, au plus une suite Rust finale si requise, jamais une suite complète par tâche. T3 impose des tests ciblés, aucune suite globale. Pas de nouvelle API HTTP, de store T3 ou de moteur Git. La décision architecturale est consignée dans docs/decisions/162-execution-distante-sous-autorite-unique.md. Les primitives SpecKit absentes sont documentées plus bas.

<a id="extrait-2"></a>

## Extrait 2

last_contact vient d’une observation du transport, jamais de task.updated_at. Réemployer la trame Heartbeat existante, avec son traitement étendu à la connexion worker authentifiée. Heartbeat est unitaire : son époque vient de la connexion, pas d’un champ ajouté à la trame. Une trame toutes les 10 secondes ; dégradation après 30 secondes selon l’horloge du maître. Aucun nouveau type ping nécessaire. last_contact reste volatile. Notifier seulement les transitions de contact_state, sans incrément de génération métier à chaque trame. Les transitions de contact_state incrémentent seq et invalident le curseur via le mécanisme existant. last_contact_at est une observation propre à chaque page lors de sa lecture, sans gel global des données volatiles. Aucun polling de mission par T3.

<a id="extrait-3"></a>

## Extrait 3

| Surface | Modification |
|---|---|
| crates/bridget-daemon/src/registry.rs | Machines, mappings privés, validation et loaders. |
| crates/bridget-transport/src/protocol.rs, lineage_protocol.rs | Placement et handshake worker ; étendre la requête et la version Lineage existantes, sans type parallèle. |
| crates/bridget-daemon/src/native_permissions.rs | Origine distincte de la destination, projection restrictive, sources Linux. |
| crates/bridget-daemon/src/delegation.rs, delegation_lineage.rs | Placement durable et projections compatibles. |
| crates/bridget-daemon/src/fleet.rs, desired_state.rs, idempotency.rs | Placement dans le canon, réservation et reprise. |
| crates/bridget-daemon/src/daemon/native_delegation.rs, daemon.rs | Admission hors verrou, dispatch, résultat, journal et annulation. |
| crates/bridget-daemon/src/lifecycle.rs, managed_process.rs | Extraction minimale des primitives réemployées, sans second superviseur. |
| crates/bridget-daemon/src/cli.rs, cli_lineage.rs, lineage_client.rs, delegation_mcp.rs | Mode worker, choix explicite et lecture enrichie. |
| crates/bridget-daemon/src/wrapper.rs | Persistance finale du couloir distant avant réseau, ACK corrélé et relais worker après fin du wrapper. |
| crates/bridget-daemon/src/remote_worker.rs | Frontière technique distante et reçus, justifiée par l’ADR et l’audit. |
| T3 packages/contracts/src/bridgetLineage.ts | Schéma v2 et compatibilité v1. |
| T3 BridgetReader.ts, BridgetLineage.ts, Orchestrator.ts | Lecture négociée et publication atomique. |
| T3 ThreadRelationshipsControl.tsx, SubagentTooltipContent.tsx, BridgetTaskJournal.tsx | Machine et contact dans les vues existantes. |

<a id="extrait-4"></a>

## Extrait 4

1. Audit de réemploi et contre-revue sur ces artefacts, puis Tasks et Analyze.
2. Configuration, contrat, authentification et tests de refus/compatibilité.
3. Mappings, isolation 154 et permissions par hôte.
4. Réservation, reçus, bootstrap, permis, réconciliation et annulation.
5. Résultat durable, journal distant, demandes 155 et reprises 152/160.
6. Lineage v2 et tests ciblés de compatibilité et de rendu.
7. Recettes isolées puis mission réelle sur chaque route autorisée, avec coupures aux frontières définies.
8. Converge, contre-revue, audit, commit, intégration, push, build et installation avec rollback.
9. Activation contrôlée de Bridget et du worker, puis reçus et nettoyage gardé. T3 reste sans relance.

<a id="extrait-5"></a>

## Extrait 5

Le pipeline demandé autorise l’activation Bridget 162. L’interdiction de relancer le maître et le pont était propre au client seul 161. Avant activation 162 : sauvegarder, vérifier les missions actives et leur reprise 152–160 ; ne pas forcer un arrêt dans une situation inconnue. T3 ne doit jamais être relancé par ce lot.

<a id="extrait-6"></a>

## Extrait 6

Worktree T3 : /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant, branche session-162-lineage-distant, base d833387203e113ec2a24f947ba8b8064a1c2542b, cible fork/local/main-20261009. Aucun push upstream, aucune PR. Pas de navigateur ou de contrôle graphique sans accord exprès ; la recette de rendu repose sur les tests intégrés isolés. Installer les fichiers et distinguer le runtime encore chargé. Les 17 worktrees préexistants de ce dépôt partagé restent hors nettoyage ; seuls les deux worktrees 162 appartiennent au lot.
