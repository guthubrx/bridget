# Session 174 — Audit des incidents et corrections

Statut : implémenté et livré le 24 octobre 2026. Autorisation : audit correctif demandé par l’utilisateur ; plafond porté à 128 et nettoyage organisé le 23 octobre 2026.

## Lot immédiat : capacité des worktrees

Le plafond de 32 refusait des projets avec des travaux conservés. Le porter à 128, en gardant la portée existante : dossiers Git enregistrés sous `.worktrees` du checkout cible, sous-dossiers inclus. Le maître compte aussi les réservations et chemins encore présents des tâches conservées. Une seule constante porte cette limite. Aucun retrait automatique ni lancement métier implicite. Ce lot ne crée pas de réglage par projet.

## Nettoyage

Conserver les worktrees actifs, modifiés, verrouillés, imbriqués ou portant des commits non intégrés. Conserver également les fichiers ignorés : un état Git propre ne prouve pas leur absence. Archiver les preuves avant retrait. Utiliser Git sans force. Après la demande utilisateur de retour à un seul main, retirer les branches dont tous les patches sont intégrés, avec une archive Git vérifiée. Recontrôler l’activité et l’état au moment de chaque retrait. Aucun nettoyage périodique installé.

## Référence T3

Lecture de `apps/server/src/vcs/GitVcsDriverCore.ts`, fonction `createWorktree` : création Git à partir d’une référence explicite, délai borné ; aucun plafond de 32 dans ce chemin. Bridget garde une borne supplémentaire pour les créations distantes et les réservations concurrentes.

## Audit général

Les revues identité/permissions et cycle de vie sont reçues. Les neuf candidats restants sont qualifiés : quatre dépendances remplacées, quatre hypothèses de permissions réfutées dans le périmètre lu, gigue de reconnexion corrigée. Le délai de réservation est sorti de la maintenance partagée. Les diagnostics associés et les variantes confirmées sont corrigés. Le rapport ciblé du 24 octobre conclut pass ; il ne constitue pas une certification générale. Voir validation/remaining-final.md.
