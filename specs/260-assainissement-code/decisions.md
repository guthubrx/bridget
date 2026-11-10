# Décisions 260 - portes réservées à François

Ce fichier est lu par l'implémenteur au début de chaque lot. François le remplit.

Mode d'emploi pour François : écrire la réponse dans la colonne « Réponse », passer le statut à `tranchée`, committer sur `main`.

Mode d'emploi pour l'implémenteur :

1. Lire la version de `origin/main` : `git fetch origin && git show origin/main:specs/260-assainissement-code/decisions.md`.
2. Statut `tranchée` : appliquer la réponse.
3. Statut `en attente` et porte non bloquante : appliquer l'action par défaut. L'écrire dans `implementation.md`.
4. Statut `en attente` et porte bloquante : ne pas faire les tâches liées. Passer au lot suivant. L'écrire dans `implementation.md`.
5. Ne jamais modifier la colonne « Réponse ».

| N° | Question | Action par défaut | Bloquante | Lots | Statut | Réponse |
|----|----------|-------------------|-----------|------|--------|---------|
| D1 | Qui active les versions exécutables produites par les lots ? Options : (a) l'implémenteur intègre et pousse les sources, le mainteneur active par vagues ; (b) l'implémenteur active à chaque lot selon `AGENTS.md` ; (c) branche d'intégration longue fusionnée par le mainteneur. | (a). Chaque lot poussé porte la mention « activation en attente » dans `implementation.md`. Le mot « livré » est interdit sans activation. | Oui, avant la première tâche qui modifie du code Rust exécutable (T011). Les lots 2, 3, 12, 14, 15 n'attendent pas. | 1, 4 à 11, 17 | en attente | |
| D2 | Dans `native_lineage.rs`, la boucle `while receiver.recv().is_ok()` sort toujours au premier tour. L'intention était-elle de reboucler après une invalidation de garde ? | Correction à comportement constant : `while` devient `if`. Aucun rebouclage ajouté. | Non | 1 | en attente | |
| D3 | 345 fichiers sont suivis par Git alors que `.gitignore` les exclut (`audits/` 317, journaux de la session 159 : 26, `AGENTS.md`, `.specify/feature.json`). Faut-il cesser de les suivre ? | Ne rien retirer. Ajouter un commentaire dans `.gitignore` qui liste ces 4 exceptions historiques. | Non | 2 | en attente | |
| D4 | Une adresse IP publique figure dans `specs/233-placement-opus2d-mac/validation.md` (dépôt public). Faut-il la masquer dans le fichier courant ? L'historique resterait inchangé. | Aucune action. La question est seulement posée. | Non | 2 | en attente | |
| D5 | Les fichiers résiduels de la racine (`.wip-reprise-preflight.txt`, `watch_20260823-175033`, `.wip-mesures/`) et deux scripts sans référence peuvent-ils être archivés ? | Oui : `git mv` vers un dossier d'archives après contrôle par `rg`. Aucune suppression. | Non | 2 | en attente | |
| D6 | Faut-il plafonner le nombre de connexions simultanées au daemon ? | Non. Le lot 7 ajoute seulement le compteur et son maximum. | Non | 7 | en attente | |
| D7 | Après mesure de la boucle périodique, faut-il regrouper ses requêtes (une requête par élément aujourd'hui) ? | Non. La tâche T106 reste fermée tant que la mesure et la réponse manquent. | Oui pour T106 seulement | 7 | en attente | |
| D8 | Faut-il signer les fichiers publiés par une signature détachée ? Avec quel outil et qui garde la clé ? | Aucune mise en place. Le lot 15 écrit seulement la proposition. | Non | 15 | en attente | |
| D9 | À quel moment appliquer le formatage de masse (120 fichiers) ? Il gêne toute branche ouverte. | Attendre. Le lot 10 ne démarre pas sans réponse. | Oui pour le lot 10 | 10 | en attente | |
| D10 | À quel moment renommer les 18 modules numérotés ? Même gêne que D9. | Attendre. Le lot 11 ne démarre pas sans réponse. | Oui pour le lot 11 | 11 | en attente | |
| D11 | `scripts/sentinel.py` exécute par `exec` un code fourni par le daemon lui-même. Garder ce mécanisme ? | Oui : commentaire d'invariant et `# noqa: S102` justifié. Aucun changement de mécanisme. | Non | 13 | en attente | |
| D12 | Le code réservé à la « boucle T806b » dans `attach.rs` est inutilisé. Le supprimer ? | Non : le conserver avec une raison écrite. | Non | 17 | en attente | |
| D13 | Quelles erreurs comparées par texte doivent devenir des types (niveau 2) ? La liste proposée sort de l'inventaire du lot 8. | Aucune. La tâche T114 reste fermée sans liste validée. | Oui pour T114 seulement | 8 | en attente | |
| D14 | Faut-il réactiver certains des 8 tests ignorés sans motif ? | Non : ajouter seulement un motif. Ne pas exécuter ces tests. | Non | 14 | en attente | |
| D15 | Jusqu'où découper `MobileViewModel.kt` (une classe, 95 fonctions, état partagé) ? | Extraire seulement les fonctions pures. Documenter le reste. | Non | 16 | en attente | |
| D16 | Quand rendre bloquant le contrôle distant des tests ? | Après une ligne de base Linux stable sur 3 exécutions. D'ici là : observation. | Oui pour T024 seulement | 1 | en attente | |

## Recommandations de l'auteur des spécifications

- D1 : option (a). Les lots 4, 5, 6, 9, 10, 11 ne changent pas le comportement. Les activer un par un coûte sept coupures pour aucun effet visible. Une activation par vague suffit.
- D2 : garder la correction constante. Un rebouclage serait un changement de comportement, donc une autre session.
- D9 et D10 : choisir un moment sans autre branche ouverte, par exemple juste après une livraison.
- D8 : voir la proposition du lot 15 avant de répondre.
