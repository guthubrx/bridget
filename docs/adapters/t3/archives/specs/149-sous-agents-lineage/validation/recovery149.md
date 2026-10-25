# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/recovery149.md](../../../../../../../specs/149-sous-agents-lineage/validation/recovery149.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Binaire release r8 IMMUABLE : `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-0a29ad9b2cdb`. SHA-256 `0a29ad9b2cdb88b1c19f95d9a9bfd1cd89292e269a92fa440864a25bdfa5dde6`, recalculé au début et à la fin de CHAQUE recette (`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results-r3/shacheck.log`) : toujours égal au reçu `native149-release-receipt-r8.json`.
- Empreinte de production au début : `3943009ca82db59d...` (égale au reçu r8). **Dérive de source en cours de ronde, hors de ma propriété** : `crates/bridget-daemon/src/wrapper.rs` a changé à 19:19:03 (digest `49f3a304d9504fe6`, reçu r8 : `28e92599eee1bb03`). L'empreinte de production devient `4fa56fcac7b37220...`. Aucun autre fichier. Je n'ai pas modifié ce fichier. Conséquence : toutes les preuves de ce rapport valent pour le binaire r8 seulement. Elles ne couvrent pas le nouveau `wrapper.rs`. Il faudra un nouveau binaire et une nouvelle ronde pour le couvrir.
- Sources T3 : `git diff HEAD` = `59a54267ce7c1d2a` avant et après toute la ronde (inchangé). HEAD `33f6d04e11`.
- Fichier d'empreintes : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results-r3/fingerprints.json`.

<a id="extrait-2"></a>

## Extrait 2

| Contrôle | Niveau | Résultat |
|---|---|---|
| E2.a : tâche `queued` EXTERNE (propriétaire = fil T3 A reconnecté) | SIMULÉ | **PASS** : elle repart, termine, remise UNE fois (`xDelivered` 1) ; la tâche terminée d'origine n'est pas rouverte (2 tours `x1` : l'original et le clone) |
| E2.b : tâche `queued` d'un propriétaire natif mort, racine `waiting_for_children` | SIMULÉ | **FAIL (BLOCK-E2)** : après 45 s la racine est toujours `waiting_for_children`, la tâche toujours `queued`, le descendant réel `failed` |
| E2c : même clone, mais le propriétaire est en plein tour (racine `working`) à la coupure | SIMULÉ | **PASS** : racine `failed`, le nettoyage ANNULE la tâche `queued`, 1 notice d'échec, aucun lancement |
| E.rel : issue de secours | RÉEL | PASS : seul `cancel` du parent libère la racine bloquée (`cancelled`) ; le résultat retenu est perdu |
