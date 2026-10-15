# Décision 167 — Coordination interagent depuis la CLI

## Statut

Acceptée le 15 octobre 2026.

## Contexte

Le moteur natif sait déjà choisir un profil, créer un agent et gérer sa tâche. MCP expose aussi l’état, l’attente, la continuation, les réponses, l’annulation et l’accusé de lecture. La CLI exposait seulement `auto-delegate`, avec plusieurs paramètres techniques.

Un utilisateur qui travaille sans T3 doit pouvoir demander à son agent coordinateur de créer et piloter des enfants. Bridget doit rester le moteur de communication et de coordination interagent. Il ne doit pas reproduire l’interface de T3.

## Décision

Bridget expose deux façades :

- `bridget delegate <MISSION>` démarre une admission automatique ;
- `bridget task <OPÉRATION>` pilote le cycle de vie natif.

La CLI appelle `delegation_mcp::execute`. Elle ne réimplémente aucune règle de sélection, de droit, de quota, de rejeu ou de stockage.

Le parent reste un agent Bridget authentifié. Un processus humain sans identité ne devient pas un faux agent. La commande refuse avant tout effet et indique les wrappers `bridget codex` ou `bridget claude`.

`bridget task list` est différent des opérations du propriétaire. Il réutilise la projection humaine globale et bornée de `control delegations`. Il donne de la visibilité. Il n’accorde aucun droit sur les tâches affichées.

## Conséquences

- T3 n’est pas requis pour déléguer ou suivre une tâche.
- Les identifiants et états ont le même sens en CLI et par MCP.
- Une clé générée est ajoutée au reçu affiché. Un script critique peut fournir `--request-id`.
- Les commandes historiques restent disponibles.
- Aucun protocole, service, stockage ou fournisseur n’est ajouté.

## Alternatives rejetées

- `spawn` puis `send` : deux effets sans reçu de tâche commun.
- principal humain synthétique : filiation et remise des résultats incorrectes.
- moteur CLI séparé : duplication des règles natives.
- copie du coordinateur T3 : hors du métier de Bridget.
