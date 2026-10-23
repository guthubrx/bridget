# Session 174 — Audit des incidents et corrections

Statut : en cours. Autorisation : audit correctif demandé par l’utilisateur ; plafond porté à 128 et nettoyage organisé le 23 octobre 2026.

## Lot immédiat : capacité des worktrees

Le plafond de 32 refusait des projets avec des travaux conservés. Le porter à 128, en gardant la portée existante : dossiers Git enregistrés sous `.worktrees` du checkout cible, sous-dossiers inclus. Le maître compte aussi les réservations et chemins encore présents des tâches conservées. Une seule constante porte cette limite. Aucun retrait automatique ni lancement métier implicite. Ce lot ne crée pas de réglage par projet.

## Nettoyage

Conserver les worktrees actifs, modifiés, verrouillés, imbriqués ou portant des commits non intégrés. Conserver également les fichiers ignorés : un état Git propre ne prouve pas leur absence. Archiver les preuves avant retrait. Utiliser Git sans force ; garder les branches. Recontrôler l’activité et l’état au moment de chaque retrait. Aucun nettoyage périodique installé.

## Référence T3

Lecture de `apps/server/src/vcs/GitVcsDriverCore.ts`, fonction `createWorktree` : création Git à partir d’une référence explicite, délai borné ; aucun plafond de 32 dans ce chemin. Bridget garde une borne supplémentaire pour les créations distantes et les réservations concurrentes.

## Audit général

Les revues identité/permissions et cycle de vie sont reçues. Leurs observations restent des candidats, à confirmer ; elles ne constituent pas une validation générale. La qualification des diagnostics de base Git et les autres corrections suivent dans cette même session.
