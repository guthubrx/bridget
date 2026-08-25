# Journal d’implémentation 025

## Métadonnées

- **Branche** : `session-025-carte-criticite-regime`
- **Base** : `b6eea777facf929d99a9c4f9ae75fb50e06dc2fd`
- **Démarré** : 2026-08-25
- **Terminé** : en cours
- **Migration** : v20 réservée

## Progression

### T2501–T2504 — Contrat et preuves métier

- **Statut** : complété
- **Fichiers** : spec, plan, modèle, contrat, tâches, Gherkin et ADR 012
- **Tests** : `git diff --check` et contrôle des termes interdits
- **Note** : le lot est coupé avant l’élection des relecteurs sur la dépendance
  réelle au capteur de la session 023. La migration v20 attend v17–v19.

### T2505–T2508 — Noyau pur F38

- **Statut** : complété
- **Fichiers** : `plugins/maicie/src/review.rs`, surface publique et sept
  tests contractuels
- **Mesure** : 7 passés, 0 échoué, 0 ignoré ; compilation préalable avec
  `cargo test -p maicie --test contract --no-run`
- **Mutants tués dans les assertions** : retrait séparé des quatre germes,
  élection arbitraire du premier homonyme et seuil Major abaissé de deux à un
- **Garde** : les ambiguïtés ne portent que l’empreinte du jeton et les chemins
  candidats ; aucun contenu de diff ou de constat ne franchit la sortie pure

### T2509–T2511 — Mesure Git exacte

- **Statut** : complété
- **Mesure** : 6 passés, 0 échoué, 0 ignoré après compilation `--no-run` ;
  Clippy strict vert sur la bibliothèque et le test d’intégration
- **Cas couverts** : worktree divergent, référence déplacée, base non ancêtre,
  renommage, référence/SHA/racine invalides, limites et variables `GIT_*`
  hostiles dans un sous-processus
- **Voie contractuelle réelle** : l’intersection des citations complètes de
  l’arbre `b6eea777` avec ses chemins suivis contient onze chemins, dont dix
  artefacts sous `specs/*/contracts/`; leur exclusion récursive laisse l’unique
  zone `crates/bridget-transport/src/protocol.rs`
- **Régime propre** : `jury_2x2` est une constante issue de la décision
  explicite du référent ; la carte ne la calcule, ne l’apprend ni ne la remplace

### T2511A–T2511C — Noyau pur de soumission et décision

- **Statut** : complété
- **Rouge initial** :
  `CARGO_TARGET_DIR=/home/moi/revue/rc2/.git/target-rc2-025 cargo test -p maicie --test contract --no-run`
  échouait sur les neuf symboles de contrat encore absents
- **Compilation préalable** : même commande, verte après implémentation
- **Mesure ciblée** :
  `CARGO_TARGET_DIR=/home/moi/revue/rc2/.git/target-rc2-025 cargo test -p maicie --test contract t2511a -- --nocapture`
  — 7 passés, 0 échoué, 0 ignoré
- **Non-régression** : contrat complet — 94 passés, 0 échoué, 0 ignoré ;
  adaptateur Git — 6 passés, 0 échoué, 0 ignoré après son propre `--no-run`
- **Invariants** : identifiant SHA-256 à champs préfixés par leur longueur ;
  attente sans régime retenu ; décision exacte du référent ; refus sans mutation ;
  aucun remplacement ; aucun champ libre ; état projeté plutôt que dupliqué
- **Régime propre** : la soumission relit la preuve d’autoprotection mais prend
  `jury_2x2` depuis `F38_FIXED_REGIME`, même si la proposition calculée est
  mutée ; la décision humaine fixe reste donc la source de vérité
- **Mutants tués dans les assertions** : régime proposé retenu implicitement
  (`review_submission.rs:155`), sens durcir/alléger inversé (`:182`), garde
  d’autoprotection retirée (`:253`), valeur propre reprise de la carte au lieu
  de la constante (`:252`) et acceptation d’un champ `motif` (`:277`)
- **Zéro écarté** : une première commande avec `--exact` mais sans nom de
  module avait filtré les 94 tests ; elle n’est pas comptée et chaque mutant a
  été rejoué avec exactement 1 test exécuté
- **Lint** : strict sur la bibliothèque et l’intégration Git ; le contrat est
  strict avec la seule règle `doc-lazy-continuation` neutralisée, car deux
  avertissements préexistants se trouvent dans `contract/routines.rs`
- **Gate** : aucune ouverture du store, aucune opération guichet et aucun DDL
  v20 ; ces écritures restent interdites avant l’absorption réelle de v17–v19

### T2511D–T2511E — Configuration fermée du projet

- **Statut** : complété
- **Rouge causal** :
  `CARGO_TARGET_DIR=/home/moi/revue/rc2/.git/target-rc2-025 cargo test -p maicie --test config_contract --no-run`
  échouait sur les trois accès au champ `review_project` absent
- **Univers vérifiés avant comptage** : `config_contract` 18 tests,
  `contract` 94 tests et le binaire `maicie` 11 tests, mesurés avec
  `cargo test ... -- --list` après leur compilation `--no-run`
- **Mesure** : 18 passés, 0 échoué, 0 ignoré ; 94 passés, 0 échoué,
  0 ignoré ; 11 passés, 0 échoué, 0 ignoré
- **Contrat** : un seul projet optionnel porte `project_id`, racine absolue
  lexicalement normalisée et `referent_id`; toute liste critique est un champ
  inconnu, et Maicie ne peut pas être son propre référent
- **Frontière d’effet** : une racine inexistante reste valide au chargement ;
  son existence sera mesurée à la soumission afin que le refus appartienne au
  greffe et soit comptable
- **Mutants tués dans les assertions, cardinal 1 vérifié** : carte manuelle
  acceptée (`config.rs:368`), existence exigée au chargement (`:391`),
  normalisation supprimée (`:462`), Maicie admise comme référent (`:462`) et
  retrait de `config.rs` du noyau fixe (`review_criticality.rs:329`)
- **Faux zéro rejeté** : les trois nouveaux oracles vivent dans
  `config_contract`; une commande filtrée sur `contract` avait exécuté zéro
  test et n’a pas été comptée
- **Lint** :
  `CARGO_TARGET_DIR=/home/moi/revue/rc2/.git/target-rc2-025 cargo clippy -p maicie --lib --test config_contract --bin maicie -- -D warnings`
  est strictement vert
- **Compatibilité daemon** : le refus d’enveloppe antérieure à Maicie aura un
  propriétaire unique, le greffe daemon ; son libellé attend la source fédérée
  de la session 026

## REX

À compléter après les gates et la revue hostile.
