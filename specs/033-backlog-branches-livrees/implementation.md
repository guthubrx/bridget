# Journal d'implémentation 033 — Backlog des branches livrées non fusionnées

**Date** : 2026-08-26

**Branche** : `session-033-backlog-branches-livrees`

**Base gelée** : `52b831b791b44382bfe0b90715fd7caf37acdcea`

## Diagnostic préalable

- `/home/moi/.config/maicie/config.json` désigne
  `/home/moi/.cache/bridget/maicie-state/maicie.sqlite3`.
- Sur une copie ouverte en lecture seule : zéro objectif, zéro délégation et
  zéro réception guichet.
- Sur une copie de `/home/moi/.cache/bridget/bridget.db` : dix-sept demandes
  guichet, toutes refusées, et zéro verdict structuré.
- Le catalogue versionné porte des verdicts en prose. Aucun rapprochement par
  mots-clés, branche ou SHA n'est admis.

Décision validée par le référent : livrer les colonnes Git et rendre
`BACKLOG BRANCHES INDISPONIBLE (greffe sans etat exploitable)` ; ne jamais
inventer `attend un relecteur` ou `attend le referent`.

Les copies diagnostiques ont été retirées après contrôle `lsof`. Les bases
sources sont intactes.

## Progression

### T3301 — Contrat et diagnostic

- **Statut** : complété
- **Preuve** : session 033 et surface confirmées ; source structurée mesurée ;
  absence de raccourci explicitement arbitrée.

### T3302 à T3307

- **Statut** : complété
- **Commit productif** : `29cd41e` — lecture des refs, budget global,
  `merge-tree` isolé et harnais Git réel.

## Réalisation

- `origin/main` est résolu une fois par exécution, puis utilisé comme snapshot
  cohérent pendant toute la mesure. Le prochain passage relit la ref : aucune
  base runtime n'est figée dans le code.
- `for-each-ref` lit les refs `origin/*` locales. Une tête ancêtre de main est
  exclue ; les autres reçoivent âge, `merge-base` et état de conflit.
- `merge-tree --write-tree` écrit uniquement dans un object store temporaire
  hors dépôt. L'object store réel est une alternative en lecture seule et
  `GIT_OPTIONAL_LOCKS=0` interdit les rafraîchissements optionnels.
- L'échéance monotone de cinq secondes couvre toutes les commandes Git. Une
  expiration ou une erreur invalide la vue entière au lieu de rendre une liste
  tronquée.
- La sortie texte place `ce qui bloque` en dernière colonne. La sortie JSON
  expose `state=partial`, la raison de l'indisponibilité métier et les trois
  limites de preuve.

## Vérifications

- Syntaxe Python : `python3 -m py_compile` verte.
- Analyse statique : Ruff vert.
- Syntaxe shell : `bash -n` verte.
- Univers du harnais : **20 passés, 0 échec, 0 ignoré**. Il comprend la
  partition historique, la présence exacte avant l'absence, le conflit, la
  base périmée, le remote non récupéré, le timeout, le dépôt invalide, les
  sorties texte/JSON et l'invariance du dépôt.
- Lecture seule : empreintes et dates de tous les fichiers sous `.git`, refs et
  fichiers du worktree identiques avant/après l'analyse.
- Mutant supprimant uniquement l'exclusion des têtes ancêtres de main :
  `branche_non_fusionnee_presente` reste vert, puis
  `branche_fusionnee_absente_apres_presence` meurt avec
  `origin/merged-lot est encore visible`. Le nominal repasse après
  restauration.
- Dépôt réel avant actualisation explicite : `origin/main=52b831b7`, 22 lignes,
  `origin/session-031-lecture-totale-remises-sans-enveloppe` visible. Après un
  fetch externe au script : `origin/main=7024df31`, 20 lignes et 031 absente.
  Le code n'a pas changé entre les deux passages.
- Dépôt réel final : **20 lots**, dont **14 sans conflit textuel** et **6 en
  conflit**, en **0,60 s** pour un budget de cinq secondes.
- `git diff --check` vert.

La closure Cargo est vide : le diff ne touche ni crate, ni manifeste, ni code
Rust. Aucun univers Cargo n'est donc attribué à ce lot.

## Minimalisme et responsabilité future

- Deux fichiers exécutables sont touchés : le script existant et son harnais.
  Aucune dépendance, cache, fichier durable ou nouvelle commande n'est ajouté.
- Complexité : O(n log n) pour le tri et O(n) commandes Git, sous budget mural
  global. Sur les vingt refs réelles, la mesure prend 0,60 s.
- Les options `--git-repo`, `--git-bin` et `--now` sont des coutures de test
  nécessaires aux oracles réels ; elles n'ajoutent aucun comportement métier.
- Potentiel minimalisme estimé : environ **0 ligne productive supprimable à
  comportement constant**. Les branches d'erreur distinctes empêchent une
  indisponibilité ou une liste partielle de devenir un faux zéro.

## Limites et dû restant

- Les verdicts ne sont pas structurés. Le lot ne peut donc pas distinguer
  `attend un relecteur` de `attend le referent` et le dit dans chaque sortie.
- Une branche distante non récupérée localement est invisible ; aucun fetch
  automatique n'est ajouté.
- L'âge est celui du commit de tête. Un rebase récent rajeunit visuellement un
  travail plus ancien.
- Une ref distante prouve une ref, pas une livraison. Une intégration par
  squash ou cherry-pick sans ascendance peut rester visible.
- `sans conflit textuel` ne prouve ni compilation, ni tests, ni sûreté de la
  composition.
- Les jurys zombies sur ancien SHA, le routage et les transitions métier
  `livré`/`jugé` restent un dû séparé.
