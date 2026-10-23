# Livraison176 — observation bornée des worktrees

Livrée le 23 octobre 2026. Source 98e187e264d7c9136896ed0803ba2b1d4327ea9b intégrée et poussée sur main, puis synchronisée sur Mac et Cartae. Neuf tâches réalisées. Cette livraison modifie les scripts Python et les guides, pas le binaire Rust.

## Effet livré
- Captures des outils et attentes bornées ; les commandes d'une observation partagent 30 secondes. Les appels filesystem Python restent hors délai dur.
- Observation des fichiers ouverts qualifiée sur Mac et Cartae. Une observation incomplète conserve les worktrees.
- Base Git et faits de commits réutilisés, recherche des imbrications indexée. La fixture128 passe de 513 à 154 appels Git ; aucun gain global de temps n'est revendiqué.
- Aucun nouveau droit, service, moteur de retrait ou rejeu métier. Plafond128 inchangé.

## Publication et lecture réelle
L'empreinte SHA-256 du script publié sur les deux hôtes est e2cf11323540ab6841269a8756d3a19be0b9ca3923ad02d63ac991fcf2d9abb0.
La commande publiée d'inventaire a rendu une observation complète sur Mac (0,953 s, 8 appels Git) et Cartae (10,501 s, 123 appels Git, une exemption de montage étranger contrôlée).
Ces passages sont des observations ponctuelles, pas une garantie de disponibilité permanente.
Le daemon conservé annonce 175-10a375e3 et running=true ; aucun redémarrage ni remplacement du binaire optimisé.

## Vérification et audit
Les résultats, sauts et erreurs de préparation des fixtures restent décrits dans verification.md.
Deux passages de 60 tests : 59 réussis et 1 ignoré sur chaque hôte. Puis 16 tests affectés réussis sur chaque hôte, sur le code final.
Audit ciblé avec la grille175 : tests75, performances75, complexité75, minimalisme75 ; huit autres axes75. Aucun score global de qualité n'est calculé. Les limites restent publiées dans audit.md.

## Notifications
Messages d'information remis au pont pour monuments et BUS, sans demande de nouvelle mission. Reçus mcp-290-6adb9c51-1 et mcp-290-6adb9c51-2 : in_flight au point de livraison, effet destinataire non confirmé. Deux tentatives initiales ont été rejetées avant envoi pour paramètres id/issued_at incomplets ; les envois valides ont utilisé les identifiants attribués par le pont.

## Nettoyage et conservations
Les 128 worktrees du seul dépôt d'essai créé pour176 ont été retirés par Git, sans force, après observation complète, contrôle de propreté et d'intégration. Aucun worktree applicatif supprimé.
Le dépôt principal de cette fixture, les preuves privées et le worktree source176 sont conservés : le manifeste d'audit référence ce dernier chemin. Les caches partagés et les travaux étrangers sont préservés.
La modification préexistante du fichier AGENTS.md sur Mac est inchangée et exclue des commits.
