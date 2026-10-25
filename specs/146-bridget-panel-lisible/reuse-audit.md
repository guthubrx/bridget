# Audit de réutilisation — SPEC146

Date : 2026-10-08. Gate de conception : PASS. Cet audit ne prouve pas une implémentation.

## Socle identifié

La session145 installée reste non committée. Son import dans les deux worktrees146 est identifié par `/Users/moi/.cache/bridget-workspace146.nnyH61/baseline145-source.json` et le manifeste d'héritage associé. Les changements146 restent séparés. Aucun fichier des racines principales ou des worktrees145 n'est modifié par cet audit.

## Inventaire et décisions

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/reuse-audit.md#extrait-1).

## Tests existants à étendre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/reuse-audit.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/reuse-audit.md#extrait-3).

`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/daemon.rs:11501` couvre corps exacts, instantané et non-mutation jusqu'à11632. `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/threads.rs:1012` couvre projection, capacité et révocation jusqu'à1114. Ajouter variantes récentes sans supprimer ces garanties145.

## Arbitrages

CRÉER : deux actions et réponses humaines récentes, une capacité négociée et un résumé récent avec activité requise. Ces différences empêchent une compatibilité silencieuse fausse.

RÉUTILISER : stockage autorisé, helper de lecture bornée, projection des noms, snapshot, reader CLI, RPC, cache natif, panneau, Button/Input/ScrollArea et copie. Une petite extraction de message est admise si elle réunit le comportement sans créer une abstraction parallèle.

NE PAS CRÉER : nouvelle dépendance, table ou migration, journal, transport, polling, résumé généré, réglage global ou framework UI. Aucune nouvelle logique agent, ACK ou mission.

## Gate avant tâches

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/reuse-audit.md#extrait-4).

Le principal relit ce gate et la contre-revue avant de donner l'ordre des tâches. Les références désignent le socle importé, avant tout déplacement de lignes par l'implémentation.
