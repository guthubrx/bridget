# Journal d'implémentation - Corrections d'usage 171

## Métadonnées
- **Session** : 171-corrections-usage (correction, validation automatique)
- **Branche** : session-171-corrections-usage
- **Démarré** : 2026-10-17
- **Base** : main 73b03a1f + correctifs 171 jusque-là actifs mais seulement dans
  `/Users/moi/.cache/bridget-delivery171/own-usage171/worktree-config171-r2/source`

## Défauts traités

### 1. Worktree opus2D jamais créé (branche orpheline)
- **Constat** : demande `opus2d-worktreeconfig-verification-20261017-73738533-r1`
  restée `chosen`, branche créée, aucun worktree.
- **Cause mesurée** : `git worktree add` du dépôt opus2D (13 656 fichiers,
  6,1 Go) dure 47,45 s ; Bridget arrêtait Git à 20 s. Git retire alors le
  dossier mais garde la branche, et la reprise refusait `workspace_branch_exists`.
- **Référence T3** : `GitVcsDriverCore.createWorktree`, `WORKTREE_ADD_TIMEOUT_MS = 300_000`.
  T3 ne retire pas la branche après échec ; Bridget le fait car sa reprise
  refuse une branche existante.
- **Correction** : délai `WORKTREE_ADD_TIMEOUT` de 300 s (local et worker
  distant) ; après une extraction échouée, retrait comparer-et-supprimer
  (`update-ref -d ref base`) de la branche neuve si aucun worktree ne l'utilise ;
  un rejeu Auto pendant l'extraction reçoit `admission_pending`.
- **Test** : `worktree171_failed_extraction_releases_only_its_unused_branch`.

### 2 et 3. Réponse d'agent non relayée, échéance « avant démarrage »
- **Constat** : réponse opus2D à 15:32:27, avant l'échéance 15:34:38, jamais
  relayée ; demande retirée « avant démarrage ».
- **Cause relevée** : 64 211 avertissements « champ response limit » depuis le
  15 octobre. État du fil : `seeded=false`. Le pont élargissait la page bornée
  vers le fil complet T3 v2 (> 16 Mio), échouait, et abandonnait la page bornée
  déjà lue : jamais amorcé, aucun tour ni réponse observés.
- **Référence T3** : `buildBoundedThreadProjection` + pagination d'historique ;
  T3 ne lit jamais le fil entier pour un client.
- **Correction** : un élargissement échoué conserve la page précédente et plafonne
  ce fil à cette taille (`detail_page_cap`).
- **Défaut 3** : la seconde demande (échéance 2 min) visait un fil occupé de
  15:36:11 à 15:40:21 ; son refus est correct et découle du défaut 2.
- **Test** : `incident171_elargissement_impossible_garde_la_page_bornee`.

### 4. Tests du lot 171 rouges, dont régressions 149/162
- **Constat** (compilations isolées par copie, cache Cargo séparé) : main 26/26,
  lot 171 13 échecs dont 4 anciens tests.
- **Causes et corrections code** :
  - hook sous un HOME via lien : comparaison chemin résolu/racine non résolue ;
  - capture depuis un cwd via lien (~/10.Scripts) : garde G054 ;
  - `managed-settings.json` macOS en root:admin 0644 refusé (G063).
- **Tests alignés sur le contrat 171** : révision v2, absence de
  `--permission-prompts` (Claude 2.1.251), preuve parent sans transplantation.
- **Ignorés avec motif** : 5 recettes à données privées réelles, 5 tests de la
  conception de transplantation remplacée par la configuration de destination.

### 4 bis. Régressions révélées par la comparaison avec main (suite complète Mac)
- main 2825 PASS/42 FAIL ; lot 171 initial 2838 PASS/56 FAIL, même environnement.
- **Worktree disparu avant lancement** : la tâche restait `starting`. Le correctif
  « lancement en vol » sautait tout contrôle ; un `stat` du cwd est rétabli.
- **Instrumentation `bridget_catalogue_observation171`** écrite sur stderr à
  chaque appel CLI et côté maître : retirée (cassait build-id et lineage CLI).
- **Tests alignés sur des contrats 171 voulus** : `extensions.worktreeConfig`
  admis, refus d'outil par hook remis avec mention, code
  `remote_start_failed:<cause>`, délai 154 sous `cfg(test)` à 20 s.
- **Non modifié** : `spec168_permis_reste_detenu_apres_timeout_demandeur_jusqu_au_drain_reel`
  échoue aussi sur main lancé seul (démarrage de `/usr/bin/python3`).
- Bancs 152/154/157 : à lancer avec un TMPDIR résolu (`/private/tmp/b.XXXX`) ;
  avec `/tmp`, main échoue aussi sur 16 d'entre eux.

### 5. Tâche c3b80e27 bloquée en annulation
- **Cause** : enveloppe tuée avant sa preuve de drainage ; groupe absent mais
  commande `unknown` pour toujours.
- **Correction** : preuve alternative si le sous-arbre cgroup v2 du service ne
  contient que le worker et qu'aucun enfant n'est suivi.
- **Vérification** : réelle après activation (cgroup Cartae relevé : 2819215 seul).

### 6. Correctifs hors Git
- Copie privée intégrée dans cette branche puis fusionnée dans main.
