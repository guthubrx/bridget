# Journal d'implémentation — Activation gouvernée des outils de pilotage

## Métadonnées

- **Spec** : 018-activation-outils-pilotage
- **Branche** : `session-18-activation-outils-pilotage`
- **Démarré** : 2026-08-25
- **Terminé** : 2026-08-25

## Progression

### Tâche T001 : Spécification, contrat et décision

- **Statut** : ✅ Complété
- **Commit** : `853206922f2d021d3985fc547a592701820ce952`
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
- **Commit** : `da7a497312d6eff871771bfc0386edd6b74b39a3`
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
