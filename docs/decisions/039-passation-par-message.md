# Passation structurée dans Send

Date : 2026-09-16. Statut : Accepté — implémenté le 2026-09-17 dans la session 103 (worktree, non fusionné, non déployé).

## Contexte

L'utilisateur veut des fonctions utiles aux agents sans orchestrateur bloquant.
Voir /Users/user/projets/bridget/.worktrees/103-dossier-passation/specs/103-dossier-passation/research.md pour les preuves et alternatives.

## Décision

Un module de validation/rendu handoff.rs, un outil métier bridget_handoff et une sous-commande handoff. Aucun nouveau stockage/service/dépendance.

## Conséquences

Positives : conservation des transports existants, pas de fournisseur/modèle/service ajouté,
interfaces bornées et tests décrits pour un mainteneur sans le contexte de cette discussion.
Négatives : conservation limitée du ledger, accès globaux historiques inchangés ; toute
référence reste soumise à ses droits et à la disponibilité de sa source.
Le statut ne sera accepté définitivement qu'après implémentation et validation des gates.

## Mise en œuvre (2026-09-17)

Réalisée telle que décrite : module `handoff.rs` (validation stricte, rendu v1
déterministe borné à 16 Kio, requête commune MCP/CLI), outil `bridget_handoff`
(preview local, send délégué à l'envoi idempotent 099 sans champ protocolaire ni
table nouvelle), commande `bridget handoff --json-stdin`. Fichier doré du rendu,
tests unitaires et d'intégration, recettes et exercices de reprise dans la skill.
