# Décision 167 — Coordination interagent depuis la CLI

## Statut

Acceptée le 15 octobre 2026.

## Contexte

Le moteur natif sait déjà choisir un profil, créer un agent et gérer sa tâche. MCP expose aussi l’état, l’attente, la continuation, les réponses, l’annulation et l’accusé de lecture. La CLI exposait seulement `auto-delegate`, avec plusieurs paramètres techniques.

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/167-coordination-cli-autonome.md#extrait-1).

## Décision

Bridget expose deux façades :

- `bridget delegate <MISSION>` démarre une admission automatique ;
- `bridget task <OPÉRATION>` pilote le cycle de vie natif.

La CLI appelle `delegation_mcp::execute`. Elle ne réimplémente aucune règle de sélection, de droit, de quota, de rejeu ou de stockage.

Le parent reste un agent Bridget authentifié. Un processus humain sans identité ne devient pas un faux agent. La commande refuse avant tout effet et indique les wrappers `bridget codex` ou `bridget claude`.

`bridget task list` est différent des opérations du propriétaire. Il réutilise la projection humaine globale et bornée de `control delegations`. Il donne de la visibilité. Il n’accorde aucun droit sur les tâches affichées.

## Conséquences

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/167-coordination-cli-autonome.md#extrait-2).

## Alternatives rejetées

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/167-coordination-cli-autonome.md#extrait-3).
