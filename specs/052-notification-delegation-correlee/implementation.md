# Journal d'implémentation 052

## Métadonnées

- **Spec** : 052-notification-delegation-correlee
- **Branche** : session-052-notification-delegation-correlee
- **Démarré** : 2026-08-27
- **Terminé** : 2026-08-27

## Progression

### T5201 — Formaliser le contrat et l'inventaire

- **Statut** : Complété
- **Faits mesurés** : construction immédiate et différée, sérialisation durable,
  reprise sans reconstruction et surfaces réellement injectées au destinataire.
- **Décision** : notification autonome portant les trois identifiants ; la
  préannonce est rejetée car elle conserve le second mandat et le délai.
- **Tests** : non applicable, documentation seule ; `git diff --check` requis
  avant commit.

### T5202 — Fermer le chemin immédiat

- **Statut** : Complété
- **Oracle rouge avant production** :
  `spec_052_delegation_immediate_transmet_les_trois_identifiants_dans_le_message_reel`
  atteint l'assertion finale puis rend `0 passed / 1 failed`, car le corps lu
  dans le `PublicMessage` ne contient encore que l'instruction.
- **Raccord** : le `message_id` est créé avant l'outbox, puis une règle du
  domaine finalise l'instruction avec les trois identifiants durables.
- **Contrôle nominal** : oracle ciblé `1 passed / 0 failed`; le témoin de cible
  de revue reste `1 passed / 0 failed` et conserve son texte initial.

### T5203 — Fermer le chemin différé

- **Statut** : Complété
- **Oracle rouge avant production** :
  `f37_depends_on_cree_arete_et_deblocage_a_la_cloture` ferme un vrai dernier
  prérequis, trouve l'outbox créée, puis rend `0 passed / 1 failed` sur le corps
  final encore limité au but.
- **Raccord** : la transaction de déblocage crée le `message_id`, finalise la
  délégation, persiste ce nouvel état et sérialise les mêmes octets dans
  l'outbox.
- **Contrôles nominaux** : chemin différé `1 passed / 0 failed`; chemin immédiat
  rejoué `1 passed / 0 failed`.

### T5204 — Prouver et livrer

- **Statut** : Complété
- **Mutant d'effet** : neutraliser la finalisation commune laisse réussir la
  création et le déblocage, puis tue séparément les deux oracles à leur
  assertion finale (`0 passed / 1 failed` dans chaque univers de 1). Après
  restauration, les deux rendent `1 passed / 0 failed`. Le SHA-256 de
  `domain.rs` vaut avant et après
  `129662caa4b0bfb1c6420a80606a3f0d7f83eed668451a8ba0e1ead4c0811769`.
- **Reprise historique** :
  `transaction_unique_expose_l_enveloppe_exacte_et_le_snapshot` et
  `deux_delegations_cli_avec_la_meme_cle_rejouent_les_memes_ids_sans_seconde_outbox`
  rendent chacun `1 passed / 0 failed`. Les octets durables ne sont pas
  reconstruits au rejeu.
- **Gate avant comptage** : `cargo test -p maicie --no-run` vert sur la base et
  la tête, 41 exécutables de test de chaque côté.
- **Univers relistés avant exécution** : base 413, tête 414.
- **Comptes** : base `406 passed / 0 failed / 7 ignored`; tête
  `407 passed / 0 failed / 7 ignored`. Le delta est exactement le nouvel
  oracle immédiat ; l'oracle différé renforce un témoin existant.
- **Statique** : `git diff --check` et `rustfmt --check` sur les cinq fichiers
  Rust touchés sont verts. Le gate global `cargo fmt -p maicie -- --check`
  reste rouge à l'identique sur base et tête dans des fichiers de catalogue
  hors lot. Le Clippy strict rencontre à l'identique des diagnostics
  préexistants dans `bridget-transport` puis Maicie ; avec les seuls
  diagnostics préexistants `needless_borrow` et `collapsible_if` neutralisés,
  le périmètre Maicie `--all-targets --no-deps -D warnings` est vert des deux
  côtés.
- **Composition** : base `e047a88284a85b3ef4b0ef51ed20b91b396d7bac`,
  divergence `0/4`; `git merge-tree --write-tree` produit exactement l'arbre
  de la tête. Les sous-arbres exécutables sont identiques à la base déjà
  comptée ; seul le registre documentaire a avancé avant ce dernier rebase.

## Non mesuré

- Aucun vrai destinataire tmux ou géré n'a reçu le message sur cette campagne ;
  les oracles s'arrêtent au `PublicMessage` durable commun aux transports.
- Aucun taux post-déploiement du délai entre notification et ancien mandat
  manuel n'est revendiqué.
- macOS et le rejeu d'une base de production ancienne ne sont pas mesurés ; il
  n'existe toutefois aucun changement de schéma ou de décodage.
