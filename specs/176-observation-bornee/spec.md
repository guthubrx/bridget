# Session 176 — Observation fiable et bornée des worktrees
Statut : In Progress

## Demande et cadrage
Demande utilisateur : « go, fais tout », après cinq axes proposés pour améliorer le lot175.
Résultat : une observation locale exploitable sur Cartae, sans confondre une information absente avec un espace libre ; un inventaire dont temps, mémoire et parcours sont bornés ; une mesure avant/après et des scores réévalués sur preuves. Réutiliser T3 quand le mécanisme existe.
Portée : scripts/build.py, ses tests, guides FR/EN, preuves d'audit et livraison. Aucun nouveau daemon, droit, format durable de tâche ou exécution métier. La suppression existante reste sans force et interdit tout espace dont l'inactivité n'est pas prouvée. Les inconnus conservent le travail.

## Exigences
- FR01 : détecter un fichier ouvert depuis un autre cwd, y compris un chemin avec espaces et un fichier supprimé encore ouvert.
- FR02 : toute erreur d'observation, sortie tronquée, accès processus refusé ou montage pertinent illisible rend la portée inconnue. Une observation partielle ne vaut jamais absence d'usage.
- FR03 : permettre une observation Cartae complète de la portée demandée avec les accès existants, sans modifier sudoers, montages, droits ou services. Aucun prompt de mot de passe.
- FR04 : borner chaque capture et l'ensemble de l'inventaire ; signaler les entrées non examinées et arrêter les processus créés lors d'un dépassement.
- FR05 : garder les comptes Git et le plafond128 ; ne jamais publier un compte tronqué comme exact. Maximum d'entrées examiné explicite.
- FR06 : éviter les lectures Git communes répétées et les comparaisons de chaque chemin avec tous les autres ; conserver les mêmes motifs et gardes.
- FR07 : qualifier temps et mémoire à32 et128 worktrees et sur le dépôt réel ; aucune promesse chiffrée avant mesure.
- FR08 : mettre à jour EN/FR, les preuves d'activation175 et les scores avec le même barème, sans modifier la grille pour gagner des points.
- FR09 : une commande d'inventaire ne retire rien ; l'état Git reste inchangé. Les essais de retrait sont limités aux fixtures privées.
- FR10 : intégrer/pousser les corrections et livrer les scripts Mac/Cartae ; aucun redémarrage de daemon n'est nécessaire si Rust reste inchangé.

## Succès
Processus de fixture réellement vu puis libéré sur Cartae et Mac ; inconnus protégés ; dépassements de temps et volume vérifiés ; mesure128 publiée ; toutes les tâches closes ou blocage nommé. Les travaux des autres projets restent intacts.
