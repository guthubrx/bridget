# Archives techniques de l’adaptateur T3

Document d’origine : [specs/171-corrections-usage/implementation.md](../../../../../../specs/171-corrections-usage/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

### 1. Worktree opus2D jamais créé (branche orpheline)
- **Constat** : demande `opus2d-worktreeconfig-verification-20261017-73738533-r1`
  restée `chosen`, branche créée, aucun worktree.
- **Cause mesurée** : `git worktree add` du dépôt opus2D (13 656 fichiers,
  6,1 Go) dure 47,45 s ; Bridget arrêtait Git à 20 s. Git retire alors le
  dossier mais garde la branche, et la reprise refusait `workspace_branch_exists`.
- **Choix de conception Bridget
  T3 ne retire pas la branche après échec ; Bridget le fait car sa reprise
  refuse une branche existante.
- **Correction** : délai `WORKTREE_ADD_TIMEOUT` de 300 s (local et worker
  distant) ; après une extraction échouée, retrait comparer-et-supprimer
  (`update-ref -d ref base`) de la branche neuve si aucun worktree ne l'utilise ;
  un rejeu Auto pendant l'extraction reçoit `admission_pending`.
- **Test** : `worktree171_failed_extraction_releases_only_its_unused_branch`.

<a id="extrait-2"></a>

## Extrait 2

### 2 et 3. Réponse d'agent non relayée, échéance « avant démarrage »
- **Constat** : réponse opus2D à 15:32:27, avant l'échéance 15:34:38, jamais
  relayée ; demande retirée « avant démarrage ».
- **Cause relevée** : 64 211 avertissements « champ response limit » depuis le
  15 octobre. État du fil : `seeded=false`. Le pont élargissait la page bornée
  vers le fil complet T3 v2 (> 16 Mio), échouait, et abandonnait la page bornée
  déjà lue : jamais amorcé, aucun tour ni réponse observés.
- **Choix de conception Bridget
  T3 ne lit jamais le fil entier pour un client.
- **Correction** : un élargissement échoué conserve la page précédente et plafonne
  ce fil à cette taille (`detail_page_cap`).
- **Défaut 3** : la seconde demande (échéance 2 min) visait un fil occupé de
  15:36:11 à 15:40:21 ; son refus est correct et découle du défaut 2.
- **Test** : `incident171_elargissement_impossible_garde_la_page_bornee`.
