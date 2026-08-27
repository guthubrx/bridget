# Journal d'implémentation — Session 038

## Métadonnées

- **Spec** : 038-regeneration-politique
- **Branche** : session-038-regeneration-politique
- **Base gelée initiale** : b676afa86174df1def7a57caa73b969567364c42
- **Base de livraison rebasée** : 7a9b582f23a05d10df624a61c04a223ce19d7753
- **Démarré** : 2026-08-27
- **Terminé** : En cours

## Progression

### T001 — Contrat et décision

- **Statut** : Terminé
- **Commit** : `docs(038): Formalise la regeneration de politique`
- **Fichiers modifiés** : documentation de session et ADR
- **Tests exécutés** : `git diff --check`
- **Notes** : la politique réelle en service n'a été ni lue ni modifiée.

### T002 à T007 — Inventaire et régénération

- **Statut** : Terminés
- **Commit** : `feat(038): Regenere les instances approuvees`
- **Fichiers modifiés** : contrat partagé de politique, module de régénération,
  scanner MCP, binaire dédié et exemple 038.
- **Tests exécutés** : `cargo check -p bridget-transport`,
  `cargo check -p bridget-daemon --bin bridget-greffe-policy-refresh`.
- **Notes** : `marker_source` reste optionnel pour la garde historique et
  devient obligatoire uniquement pour la régénération. Le chemin de politique
  est explicite, absolu, canonique et non lié.

### T008 — Oracles unitaires

- **Statut** : Terminé
- **Commit** : `feat(038): Regenere les instances approuvees`
- **Univers** : 11 tests transport ; 9 tests `mcp_identity`, dont 3 nouveaux ;
  2 tests du parseur de commande.
- **Résultats** : 11/0/0, 9/0/0 et 2/0/0.
- **Propriétés** : sources exactes, zéro vivant, PID recyclé, ambiguïté,
  fraîcheur, conservation d'un principal arrêté, refus d'enrôlement,
  génération croissante et deux frontières du remplacement atomique.

### T009 — Chemin réel et mutants

- **Statut** : Terminé
- **Commit** : `test(038): Prouve le renouvellement par le vrai binaire`
- **Univers** : 1 test d'intégration réel.
- **Résultat nominal** : 1 passed / 0 failed / 0 ignored.
- **Mutant génération** : 0 passed / 1 failed ; assertion métier dans
  `greffe_policy_refresh.rs`, valeur observée 7 contre valeur attendue 8.
- **Mutant écriture directe** : 0 passed / 1 failed ; assertion métier sur les
  phases observées, liste vide contre `[BeforeRename, AfterRename]`.
- **Restauration** : test exact revenu à 1/0/0 après chaque mutant.
- **Contrôle positif** : le vrai binaire scanne, prévisualise puis applique ; la
  garde réelle refuse l'ancienne instance avant effet et la nouvelle écrit le
  fichier durable.

### T010 — Gates et livraison

- **Statut** : Gates terminés, revue externe demandée
- **Commit** : `docs(038): Consigne les gates de livraison`
- **Compilation avant comptage** : `cargo test --workspace --no-run` vert
  après rebase, `Finished test profile` en 11,41 s.
- **Clôture déterminée par le diff** : `bridget-transport`, `maicie` et
  `bridget-daemon`.
- **Paquet transport** : univers listé 191 ; 190 passed / 0 failed / 1 ignored.
- **Paquet Maicie** : univers listé 405 ; 398 passed / 0 failed / 7 ignored.
- **Paquet daemon** : univers listé 624 ; 602 passed / 10 failed / 11 ignored /
  1 filtered. L'unique exclusion, vérifiée comme une seule correspondance, est
  `daemon::presence_tests::stop_apres_register_traverse_le_wrapper_et_le_superviseur_reels`.
- **Workspace final, exécuté une seule fois** : univers listé 1262 ;
  1232 passed / 10 failed / 19 ignored / 1 filtered. Le compte ferme sans
  additionner les lignes `test result` produites par des sous-processus.
- **Imputation des dix rouges** : aucun n'appartient au delta 038.
  `enregistrement_auxiliaire_mcp_ne_revendique_pas_la_presence_du_wrapper_vivant`
  est 0/1 sur la base et sur la tête avec la même assertion ; les binaires
  `claude_native_permissions_test` et `codex_native_test` reproduisent les mêmes
  rouges sur la base ; `ui_relay_test` rend 17/3 des deux côtés avec des noms
  variables ; `sigkill_daemon_reconcilie_l_ancien_groupe_avant_une_reprise_unique`,
  `prompt_reduit_rejoue_le_corpus_dans_la_meme_session` et
  `reprise_codex_rejoue_la_panne_mcp_et_clot_les_demandes_liees` sont les rouges
  préexistants ou instables déjà instruits. Les sorties brutes sont conservées.
- **Formatage** : `rustfmt --check` ciblé vert. `cargo fmt --all --check` reste
  rouge sur les mêmes 81 emplacements à la base et à la tête, aucun dans le
  delta 038.
- **Clippy** : la commande stricte sur transport + Maicie + daemon reste rouge
  sur le même ensemble de diagnostics préexistants à la base et à la tête. Les
  commandes ciblant les surfaces 038, après neutralisation explicite de ces
  seuls diagnostics hérités, sont vertes sous `-D warnings`.
- **Hygiène** : `git diff --check` vert ; `git merge-tree` contre
  `origin/main` 2561dce20e2b7154c19d5ff6f3b44d79921f9003 vert.
- **Résidus** : le correctif de terminaison de 7a9b582 fait finir
  `attach_journal_attestation_test` en 0,26 s ; aucun processus issu de la passe
  rebasée ne subsiste.

## REX — Retour d'expérience

- Une collecte distribuée sûre doit rendre sa complétude structurelle : la
  politique énumère les sources exigées, l'invocation ne peut donc pas réduire
  silencieusement le périmètre observé.
- Un test négatif ne suffit pas pour une garde. Le chemin réel du binaire prouve
  aussi qu'une politique régénérée autorise effectivement la nouvelle instance
  à produire un effet durable.
- Une ligne `test result` n'est pas nécessairement un harnais : les comptes
  finaux sont rapprochés de l'univers listé et utilisent le résultat extérieur.
- Une base portant un correctif d'infrastructure doit précéder sa mesure. La
  première passe, restée sur b676afa, bloquait sur l'ancien arrêt ; le rebase
  demandé sur 7a9b582 a fait terminer le même harnais sans résidu.
