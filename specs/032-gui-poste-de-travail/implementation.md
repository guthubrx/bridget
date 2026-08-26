# Journal d'implémentation — lot L7

**Date** : 2026-08-26 · **Branche** : `feat/032-l7-contenu-messages`
**Base figée** : `c772f0b8c43741d83dc035c1a13e6377100cde78`

## Cause vérifiée

Les traces `peer_exchange` ne contiennent volontairement que leur direction,
leur cardinal et leurs `delivery_ids`. Les corps n'étaient donc pas perdus :
ils étaient publiés séparément par `/v1/journal`, dans les événements
`turn_start` ou `prompt_dispatched` riches.

Deux mesures directes ont confirmé la source avant toute modification :

- `7980b795-57d9-4241-b667-cac162ae2f57` porte le corps exact du mandat et
  `ts=2026-08-26T03:05:49Z` ;
- `667db3566b784` porte le message humain exact de contrôle et
  `ts=2026-08-25T20:20:46Z`.

Le relais Rust n'a donc pas été modifié.

## Réalisation

- Index des corps par `message_id`, alimenté par le journal de l'agent courant
  et, lors d'un dépli, par celui du pair.
- Projection de chaque message entrant riche en bulle à droite et de la
  réponse de l'agent courant en bulle à gauche.
- Dépli des traces avec les textes dans l'ordre des `delivery_ids`, sans rendre
  les identifiants.
- Heures et séparateurs calculés dans le fuseau local du navigateur ; tri
  conservé sur le `ts` d'émission.
- Rattrapage `from_seq=0` pour traverser les rotations, ajouté par lot avant un
  seul rendu à `SnapshotCaughtUp`. Ce regroupement évite un rendu du DOM par
  fragment sur les gros journaux.
- Course de rendu du dépli fermée : si le fil est remplacé pendant le
  chargement, la résolution redessine le fil courant plutôt qu'un ancien nœud
  détaché.

## Vérifications

- Précompilation : `cargo test -p bridget-daemon --no-run` verte.
- Univers Cargo listé : **539 tests**, 0 benchmark.
- Closure `bridget-daemon` avec `--no-fail-fast`, passage final : **527 passés,
  2 échoués, 10 ignorés**. Les deux échecs sont hors diff et nommés :
  `attach::tests::raw_mode_restaure_le_terminal_apres_eof_du_pseudo_tty`,
  `lifecycle::tests::matrice_sc003_couvre_les_onze_familles_sans_residu_operationnel`.
  Un passage précédent mesurait **526/3/10** :
  `wrapper::reconnect_tests::livraison_idempotente_interactive_injecte_une_fois_et_rejoue_l_accuse`
  avait rencontré son verrou de reçu intermittent, puis a repassé au passage
  final. Aucun de ces trois tests ni leur code Rust n'appartient au diff L7.
- Tests JavaScript : **23 passés, 0 échoué, 0 ignoré**.
- Mutant « rendre les identifiants » : tue
  `depli_echange_resout_les_corps_exacts_sans_exposer_les_identifiants` ; le
  test nominal repasse après restauration.
- `cargo fmt --all -- --check`, Clippy `-D warnings` et `git diff --check`
  verts.
- Navigateur piloté en `Europe/Paris` :
  - le corps exact du mandat apparaît en bulle à **05:05**, cohérent avec le
    `ts` brut **03:05:49Z** ;
  - le corps humain exact de `rc5` apparaît à **22:20**, cohérent avec le `ts`
    brut **20:20:46Z**, sous « mardi 25 août » ;
  - une trace sortante à **23:25** déplie le texte commençant par
    « L4 6c125d74 — MAINTIEN AMENDER » ; aucun identifiant n'est visible ;
  - les séparateurs « mardi 25 août » et « mercredi 26 août » sont présents.

## Limites explicites

- Aucune route, donnée persistée, règle d'authentification, approbation ou
  structure du relais Rust n'est ajoutée.
- Une clé historique absente des deux journaux reste irrésoluble. La page
  affiche alors « Contenu indisponible » et ne révèle jamais la clé. Le lot ne
  recrée pas un corps qui n'est plus publié par la source.
- Le lot ne change ni la rétention des journaux, ni la borne de projection du
  ledger, ni les pilotes d'agents.
- Les échecs Cargo nommés ci-dessus ne sont pas corrigés : aucun fichier Rust
  n'appartient au diff L7.
