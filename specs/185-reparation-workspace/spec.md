# Session 185 — Réparation native du dossier d'une tâche
Statut : autorisée le 27 octobre 2026 par « fais ce qui est nécessaire » ; qualification en cours ; aucun runtime185 activé.

## Résultat attendu
Un agent propriétaire peut demander à Bridget de rattacher sa même tâche distante à son dossier déplacé. Bridget conserve identité de tâche, historique, résultats, commits et fichiers. La réparation ne crée aucune mission métier. Le prochain tour utilise le dossier réel validé.

## Cas réel
La tâche SYS 58459353-0d42-4fc2-acc4-692949616648 a rendu son résultat. Son enfant a déplacé son dossier vivant vers /mnt/storage/opus2d/worktrees/58459353 et laissé un lien au chemin historique. La reprise refuse à juste titre le placement non attesté. Les données sont conservées. L'autorisation de réparation ne permet aucun forçage, changement direct de base ou redémarrage de service.

## Contrat
- Réparation explicite, idempotente par tâche et request_id. Un rejeu divergent refuse.
- Propriétaire et parent attestés ; tâche à repos après résultat remis ; aucune admission ou ronde supplémentaire pendant réparation.
- Destination absolue canonique, dépôt et branche attendus, HEAD explicite, base conservée, fichiers préservés. Un chemin arbitraire, lien substitué ou autre dépôt refuse.
- Droits relus et projetés par les mécanismes existants ; aucune extension implicite.
- Historique du changement et point de reprise durables avant tout effet.
- Même session fournisseur attestée ; le processus technique peut être remplacé à repos. Aucun repli silencieux sur conversation neuve.
- Aucune suppression des données pendant cette opération.
- Interruption, réponse perdue, annulation concurrente : état explicite et réparation conservée, aucune remise métier incertaine.
- Ancien pair ou fournisseur incompatible : refus précis avec action humaine utile.
- Installation et activation uniquement par maintenance native admise.

## Hors périmètre
Migration de maître, changement de fournisseur/modèle/compte, déplacement de projets complets, contournement des droits, redémarrage global, suppression automatique de l'ancien dossier.
