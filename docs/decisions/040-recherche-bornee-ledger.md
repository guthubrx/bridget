# Recherche reprenable dans les sources existantes

Date : 2026-09-16. Statut : Proposé, préparation documentaire sans implémentation.

## Contexte

L'utilisateur veut des fonctions utiles aux agents sans orchestrateur bloquant.
Voir /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/specs/104-recherche-echanges/research.md pour les preuves et alternatives.

## Décision

Deux index d'accès ledger, deux opérations de protocole et actions de lecture/recherche dans les modules existants ; aucune nouvelle table/dépendance/service.

## Conséquences

Positives : conservation des transports existants, pas de fournisseur/modèle/service ajouté,
interfaces bornées et tests décrits pour un mainteneur sans le contexte de cette discussion.
Négatives : conservation limitée du ledger, accès globaux historiques inchangés ; toute
référence reste soumise à ses droits et à la disponibilité de sa source.
Le statut ne sera accepté définitivement qu'après implémentation et validation des gates.
