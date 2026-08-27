# Journal d'implémentation — Session 038

## Métadonnées

- **Spec** : 038-regeneration-politique
- **Branche** : session-038-regeneration-politique
- **Base gelée initiale** : b676afa86174df1def7a57caa73b969567364c42
- **Base de livraison rebasée** : 90802b0377741b509f3743c5675544315b6f0f29
- **Démarré** : 2026-08-27
- **Terminé** : 2026-08-27

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

- **Statut** : Terminé, revue externe demandée
- **Commits** : `docs(038): Consigne les gates de livraison`, puis
  `docs(038): Consigne les amendements finaux` après le dernier rebase.
- **Compilation avant comptage** : `cargo test --workspace --no-run` vert sur
  la base 90802b0 et sur la tête finale.
- **Clôture déterminée par le diff** : `bridget-transport`, `maicie` et
  `bridget-daemon`.
- **Paquet transport** : univers listé 196 ; 195 passed / 0 failed / 1 ignored.
- **Paquet Maicie** : univers listé 405 ; 398 passed / 0 failed / 7 ignored.
- **Paquet daemon** : univers joué 649 ; 631 passed / 7 failed / 11 ignored.
  L'unique exclusion, vérifiée comme une seule correspondance, est
  `daemon::presence_tests::stop_apres_register_traverse_le_wrapper_et_le_superviseur_reels`.
- **Clôture paquet et dépendants** : univers joué 1250 ; 1224 passed / 7 failed /
  19 ignored.
- **Workspace final, exécuté une seule fois** : univers joué 1294 après
  l'exclusion exacte ; 1268 passed / 7 failed / 19 ignored. Le compte ferme sans
  additionner les lignes `test result` produites par des sous-processus.
- **Imputation des sept rouges** : aucun n'appartient au delta 038.
  `enregistrement_auxiliaire_mcp_ne_revendique_pas_la_presence_du_wrapper_vivant`
  et `sigkill_daemon_reconcilie_l_ancien_groupe_avant_une_reprise_unique` sont
  0/1 sur la base et sur la tête avec la même assertion ;
  `claude_gere_sans_bypass_reste_sans_outil`,
  `wrapper_codex_natif_repond_et_reste_attachable` et
  `wrapper_codex_sans_signal_laisse_effort_et_limite_inconnus` reproduisent
  aussi 0/1 sur la base avec le même message ;
  `prompt_reduit_rejoue_le_corpus_dans_la_meme_session` et
  `reprise_codex_rejoue_la_panne_mcp_et_clot_les_demandes_liees` reproduisent
  0/1 sur la base avec le même refus « nom déjà pris ». Les sorties brutes sont
  conservées.
- **Formatage** : `rustfmt --check` ciblé vert. `cargo fmt --all --check` reste
  rouge sur les mêmes 74 emplacements à la base et à la tête, sorties
  normalisées octet-identiques et aucun emplacement dans le delta 038.
- **Clippy** : la commande stricte sur transport + Maicie + daemon reste rouge
  sur les mêmes 14 diagnostics à la base et à la tête ; les messages et fichiers
  normalisés sont identiques. Les deux états compilent avant cette comparaison.
- **Hygiène** : `git diff --check` vert ; les cinq univers touchés ferment à
  14/0/0, 15/0/0, 9/0/0, 3/0/0 et 2/0/0 après restauration des mutants.
- **Résidus** : aucun processus ni descripteur issu des TMPDIR de la passe
  finale ne subsiste.

### Amendements après gel

- **Issue après renommage** : l'erreur pré-renommage garde l'original ; une
  erreur post-renommage relit la politique et rend une issue indéterminée
  typée. Le mutant qui oublie `AfterRename` meurt à l'égalité métier : `Io`
  observé contre `WriteOutcomeIndeterminate` attendu.
- **Entrée privée** : propriétaire et mode de l'inventaire sont vérifiés sur le
  descripteur déjà ouvert par la même primitive que la politique. Sans cette
  garde, le vrai chemin rend 0 au lieu du refus 2 attendu.
- **Entrées uniques** : les liens physiques multiples de la politique et du
  verrou sont refusés avant plan. Les deux retraits isolés meurent contre
  `PolicyPathNotUnique` et `LockPathNotUnique` plutôt que d'atteindre
  `MissingInventory`.
- **Types spéciaux** : marqueur, fichier de nom, inventaire et politique sont
  ouverts avec `O_NONBLOCK`, puis leur type est vérifié sur le descripteur. Le
  retrait de `O_NONBLOCK` meurt par la limite propre de deux secondes et laisse
  zéro enfant ; rabattre le motif de type vers `politique invalide` meurt sur
  l'égalité littérale du message opérateur.

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
- Deux worktrees ne doivent pas partager un `CARGO_TARGET_DIR` : Cargo peut
  réutiliser les métadonnées d'une dépendance de chemin compilée depuis l'autre
  arbre. La première tentative du mutant d'inventaire a été rejetée comme
  mesure invalide, puis rejouée avec des targets séparés.
