# Bridget 0.3.0

Version construite depuis `24a6507a99520bf2245caf67fa69cbac40bd300f`.

## Coordination autonome

- `bridget delegate <MISSION>` permet à un agent Bridget authentifié de créer un enfant et de lui confier une mission en une commande.
- `bridget task` permet de lister, lire, attendre, continuer, répondre, annuler et accuser lecture depuis un terminal.
- Le parcours réutilise le moteur natif de Bridget. T3 reste compatible, mais il n’est pas requis.
- Les choix avancés restent facultatifs. Un choix explicite indisponible est refusé sans substitution silencieuse.
- La sortie `--json` fournit un objet stable pour les scripts.

## Sécurité

- La délégation exige une identité d’agent Bridget vivante.
- Un terminal humain sans identité reçoit `identity_not_found` avant tout effet métier.
- Les opérations de tâche restent limitées au coordinateur propriétaire. `task list` conserve la projection globale en lecture seule.

## Actifs et validation

- `bridget-0.3.0-linux-x86_64.tar.gz` contient le binaire Linux x86_64, la licence, la documentation et les sommes internes.
- SHA-256 de l’archive Linux : `3f4455dc380c855002178df49a4f0b6447bd7323d00f2acc090ca83eeb0296d8`.
- SHA-256 du binaire Linux : `989c27777d34c57fab9658f9aee2a05409be243bd567daec61597def305a8089`.
- SHA-256 du binaire macOS signé : `4783e8cb4e3389d71c40d6ecb5e0d89bb98f2d7af14e2275c7e48f6b92ed98cd`.
- Tests ciblés : 87 PASS et 2 ignored.
- Audit v14 : A, aucun finding.
- La suite globale conserve 90 échecs hors diff ou liés à l’environnement après correction du seul échec introduit par cette version. Aucun succès global n’est revendiqué.
