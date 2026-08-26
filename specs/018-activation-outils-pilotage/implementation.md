# Journal d'implémentation — Activation gouvernée des outils de pilotage

## Métadonnées

- **Spec** : 018-activation-outils-pilotage
- **Branche** : `session-18-activation-outils-pilotage`
- **Démarré** : 2026-08-25
- **Terminé** : 2026-08-26

## Progression

### Tâche T001 : Spécification, contrat et décision

- **Statut** : ✅ Complété
- **Commit rebasé** : `7070636aa4821e532d54a117ccd5c36e317a100b`
- **Fichiers créés** : spec, checklist, recherche, contrat, plan, quickstart,
  tâches, journal et ADR de la session 018
- **Tests exécutés** :
  - [x] Checklist de spécification : 17/17
  - [x] Format des tâches : 8/8
  - [x] `git diff --check` : OK
- **Notes** : `SPEC-017` n'est pas une dépendance ; les outils nécessaires sont
  déjà présents sur `main` et la frontière d'activation dépend de SPEC-011.

### Tâches T002 à T006 : Oracles, politique commune et deux installateurs

- **Statut** : ✅ Complété
- **Commit rebasé** : `495e142e2dd60c05fab51e14a2733881722bb6c2`
- **Contrôle positif TDD** : avant l'implémentation, le nouveau harnais a
  échoué sur le worktree lié avec `baseline_rc=1` ; l'ancien installateur avait
  posé le lien interdit.
- **Résultat** :
  - refus ordonnés du worktree lié, de la branche hors `main`, de l'arbre sale,
    de la copie préexistante sans `--force` et de la tête non admise ;
  - refus supplémentaire si `origin/main` est absente, sans accès réseau ;
  - rollback vers un ancêtre admis autorisé ;
  - artefact extrait du blob Git, origine lisible, activation atomique et
    rejeu idempotent ;
  - corruption d'une release existante refusée, même avec `--force` ;
  - `bridget-idle` et `bridget-ronde` restent exécutables après suppression du
    dépôt source de test ;
  - la ronde refuse avant de créer son répertoire de rapports ou ses unités.

### Tâche T007 : Validation et imputation

- **Statut** : ✅ Complété
- **Syntaxe Bash** : 5/5 fichiers contrôlés par `bash -n`.
- **Harnais métier** :
  - `scripts/test-018-pilotage-install.sh` : 10/10 scénarios ;
  - `scripts/test-bridget-idle.sh` : vert ;
  - `scripts/test-bridget-ronde.sh` : vert.
- **Mutants** : 5/5 tués — garde worktree, garde branche, garde propreté,
  garde remplacement et garde admission.
- **Compilation** : `cargo test --workspace --no-run` vert.
- **Suite complète** : `928 réussis / 5 rouges / 16 ignorés`. Le compte somme
  les 58 lignes de résultat correspondant aux 54 exécutables et 4 doctests ;
  deux sorties de sous-processus avec 108 tests filtrés ne sont pas recomptées.
- **Rouges déterministes préexistants** :
  1. `attach::tests::raw_mode_restaure_le_terminal_apres_eof_du_pseudo_tty` —
     `tcgetattr` retourne `EIO` ;
  2. `attach::tests::reconnexion_socket_reprend_exactement_a_last_seq_plus_un`
     — `unwrap_err()` reçoit `Ok(())` ;
  3. `lifecycle::tests::matrice_sc003_couvre_les_onze_familles_sans_residu_operationnel`
     — la famille `cursor` est présente dans le résultat réel.
- **Rouges de charge non imputables au lot** :
  1. `sc001_vingt_spawns_survivent_a_la_fermeture_du_client_et_repondent` —
     `WouldBlock` dans la suite complète, puis 3/3 vert isolément ;
  2. `sc005_deux_vues_reelles_ne_degradent_pas_le_p95_d_append_de_plus_de_cinq_pourcent`
     — accusé absent sous charge globale, puis 3/3 vert isolément.
- **Imputation** : aucun fichier Rust ne diffère de `origin/main`; les cinq
  rouges sont dans le socle. Aucun des quatre tests instables préannoncés
  (`matrice_fr008`, `eof_pendant_un_tour`,
  `redelivery_idempotente_apres_reconnexion`,
  `acp::ignored_cancel_kills_transport`) ne figure parmi les rouges de cette
  campagne.
- **Formatage Rust** : rouge préexistant dans quatre fichiers Maicie inchangés.
- **Clippy** : rouge préexistant, deux `doc_lazy_continuation` dans
  `plugins/maicie/tests/contract/routines.rs` lignes 964-965.

### Tâche T008 : REX, gel et livraison

- **Statut** : ✅ Complété
- **Base admise** : `b6eea777facf929d99a9c4f9ae75fb50e06dc2fd`.
- **Branche** : `session-18-activation-outils-pilotage`.
- **Gel** : le SHA de ce dernier commit documentaire est contrôlé après le
  commit, poussé sans réécriture et annoncé dans Bridget.
- **Déploiement** : aucun chemin de production n'a été modifié depuis la
  branche de feature ; l'activation doit être rejouée depuis le checkout
  principal propre après merge.

## REX

### Ce qui a bien fonctionné

- Le contrôle positif initial a reproduit exactement le canary clandestin.
- Une politique commune évite que `idle` et `ronde` divergent à nouveau.
- La preuve adjacente à l'artefact rend l'origine lisible sans recopier une URL
  distante susceptible de contenir des identifiants.
- Les fixtures Git principales et jetables testent la frontière réelle sans
  jamais activer le code de la branche sur la machine de pilotage.

### Ajustements pendant l'implémentation

- L'ancien harnais de ronde lançait son installateur depuis le worktree de
  développement ; ce comportement est désormais interdit par contrat. La
  couverture des unités et de leur passivité a été transférée au harnais 018.
- Le premier total Cargo incluait deux résumés de sous-processus. Le compte a
  été recalculé sur les seuls résultats de la campagne principale : 928/5/16.

### Non mesuré

- Le gate `--features test-support` n'a pas été lancé sur Linux : le blocage
  connu `kqueue`/`kevent` sans garde `cfg` dans `idempotency_crash_test.rs` est
  hors de cette session.
- `shellcheck` n'est pas installé sur l'hôte.
- L'activation launchd et l'exécution réelle sur macOS ne sont pas mesurées.
- Aucun installateur n'a été exécuté sur les commandes actives de production.

## Reprise après verdict STOP — 2026-08-26

- **Base rebasée** : `7024df31de5b23bfeca27eb5588a5465a872a8b8`.
- **Commit de formalisation** : `496542d14fc974dbbbb96df564318e6ac62c1b61`.
- **Commit productif** : `ea6f49c7925788bd2fddb2076d3b049c56cd5096`.
- **Statut** : complété, prêt en contre-relecture.

### Reproduction avant correction

- `release_lien` : une release remplacée par un lien vers le dépôt, mêmes
  octets, était annoncée `déjà en place`.
- `release_mode` : une release passée de `0555` à `0755` était annoncée
  `déjà en place`.
- `force_remplace_entree_exacte` : `mv -f` déposait le lien préparé dans le
  répertoire visé par l'ancienne entrée et la cible active restait inchangée.
- `configuration_divergente_refusee_sans_faux_succes` : une configuration B
  terminait à zéro et annonçait la ronde prête tandis que service et timer
  conservaient A.

### Correction

- Une release et sa preuve existantes doivent être des fichiers réguliers non
  liens, aux modes exacts `0555` et `0444`, avant la comparaison des octets.
- `os.replace` remplace atomiquement l'entrée active exacte sur le même système
  de fichiers, y compris un lien vers répertoire. La cible textuelle réellement
  obtenue est relue avant le message `posé`.
- Chaque unité est d'abord matérialisée séparément. Un rejeu ne réussit que si
  représentation, mode et contenu sont exacts ; sinon il refuse sans
  `--force`. Sous `--force`, le remplacement est explicite et attesté.
- Les trois `|| true` placés sur les appels à `write_unit` ont été supprimés.

### Vérifications de reprise

Univers shell listé :

1. `scripts/test-018-pilotage-install.sh` — **1 passé, 0 échec, 0 ignoré** ;
2. `scripts/test-bridget-idle.sh` — **1 passé, 0 échec, 0 ignoré** ;
3. `scripts/test-bridget-ronde.sh` — **1 passé, 0 échec, 0 ignoré**.

Le harnais 018 couvre les deux refus B1, le remplacement exact B2, le refus M1,
le remplacement explicite de B par `--force`, l'idempotence nominale et
l'exécution des deux commandes après suppression du dépôt source.

- Syntaxe : six fichiers Bash passés, zéro échec, zéro ignoré.
- `git diff --check` : vert.
- Preuve Cargo conservée de la contre-revue, non rejouée conformément au
  mandat : `cargo test --workspace --no-run` vert sur base et composition ;
  univers 1014, base et composition **993 passés, 3 échecs connus, 18 ignorés**.
  Le diff de reprise ne touche ni Rust ni manifeste Cargo.
- Mutant sans contrôle de représentation :
  `release_lien` meurt sur l'absence du contenu
  `attendu=fichier_regulier_non_lien`.
- Mutant sans contrôle du mode : `release_mode` meurt en acceptant `0755`.
- Mutant restaurant `mv -f` : `force_remplace_entree_exacte` meurt sur la
  cible demeurée dans l'ancien répertoire.
- Mutant restaurant l'avalement d'échec :
  `configuration_divergente_refusee_sans_faux_succes` meurt sur le succès
  interdit et l'annonce mensongère.

### Minimalisme et responsabilité future

- Complexité O(1), sans boucle ni nouvelle dépendance : Python était déjà le
  runtime obligatoire des deux outils et `os.replace` porte l'invariant exact
  qui manquait à `mv` sur macOS/Linux.
- Les helpers de mode et de remplacement servent à la fois les releases et les
  unités ; ils portent une règle de compatibilité ou d'atomicité, pas une couche
  abstraite supplémentaire.
- Potentiel minimalisme : environ **0 ligne productive supprimable à
  comportement constant** ; retirer une garde, l'attestation ou la propagation
  d'échec ressuscite un témoin nommé.
- La correction réduit la charge future : chaque succès correspond désormais
  à un état relu, et non à l'intention d'une commande système.

- **Non visité** : macOS/launchd réel, activation systemd réelle sans saut,
  concurrence entre installateurs et falsification volontaire de références
  Git.
