# Passation structurée dans Send

Date : 2026-09-16. Statut : Proposé, préparation documentaire sans implémentation.

## Contexte

L'utilisateur veut des fonctions utiles aux agents sans orchestrateur bloquant.
Voir /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/specs/103-dossier-passation/research.md pour les preuves et alternatives.

## Décision

Un module de validation/rendu handoff.rs, un outil métier bridget_handoff et une sous-commande handoff. Aucun nouveau stockage/service/dépendance.

## Conséquences

Positives : conservation des transports existants, pas de fournisseur/modèle/service ajouté,
interfaces bornées et tests décrits pour un mainteneur sans le contexte de cette discussion.
Négatives : conservation limitée du ledger, accès globaux historiques inchangés ; toute
référence reste soumise à ses droits et à la disponibilité de sa source.
Le statut ne sera accepté définitivement qu'après implémentation et validation des gates.
