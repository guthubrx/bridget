# Audit final - SPEC-075 cycle de vie complet des agents gérés

## Verdict

**PASS pour le périmètre SPEC-075, non livré.**

Les deux constats majeurs de la contre-revue ont été corrigés: course entre
relance et décommissionnement, puis repli d'une action UI inconnue vers
`stop`. Aucun constat critique ou majeur ne reste ouvert dans le périmètre.

## Preuves vertes

- `cargo test -p bridget-daemon --lib -- --test-threads=1`:
  684 réussis, 0 échec, 7 ignorés.
- `node --test app.js`: 89 réussis, 0 échec.
- Test protocole lifecycle: 1 réussi, 0 échec.
- Test de course relance/décommissionnement: réussi.
- Test durable arrêt, réouverture, relance, décommissionnement, réouverture et
  conservation du journal: réussi.
- Test de persistance et réconciliation SC-005/SC-006: réussi après adaptation
  à la visibilité durable des agents arrêtés.
- `cargo fmt --all -- --check`: réussi.
- `git diff --check`: réussi.
- `cargo clippy -p bridget-daemon --lib` avec neutralisation des trois lint
  historiques du workspace: réussi avec `-D warnings`.

## Suite globale

`cargo test --workspace -- --test-threads=1` atteint trois échecs dans
`managed_parity_test.rs`:

- `matrice_fr008_compare_le_meme_corpus_et_les_frames_attach`;
- `reprise_codex_rejoue_la_panne_mcp_et_clot_les_demandes_liees`;
- `reprise_codex_sans_amorcage_ne_decouvre_pas_mcp`.

Ces trois tests échouent de manière identique sur un worktree détaché et propre
de `origin/main` au commit `aba60f0`. Ils sont donc préexistants et ne couvrent
pas le cycle de vie ajouté. Le quatrième échec initial, causé par l'ancienne
attente de disparition des agents non persistants, a été corrigé et son test
passe désormais.

Le clippy strict de tout le workspace reste bloqué par trois lint préexistants:
`collapsible-if` dans `codex_app_server.rs`, `should-implement-trait` dans
`protocol.rs` et `too-many-arguments` dans `wrapper.rs`. Le crate daemon modifié
passe strictement après neutralisation explicite de ces seuls baselines.

## Contre-revue externe

- Claude: quota épuisé.
- Gemini: authentification absente.
- GLM: aucun verdict produit.
- Codex indépendant en lecture seule, premier passage: sa trace a fourni les
  deux constats majeurs corrigés et consignés dans
  `adversarial-plan-review.md`.
- Codex indépendant en lecture seule, second passage ciblé après correction:
  verdict `PASS`, aucune faille P0, P1 ou P2 restante. Le verdict intégral est
  conservé dans `adversarial-implementation-review-2.md`.

## Risques résiduels

- Aucun canary production n'a été exécuté, car la consigne du pipeline interdit
  livraison et redémarrage automatiques.
- Le premier passage en production doit vérifier l'adoption explicite des
  anciens agents arrêtés avant le redémarrage du daemon.
- La purge définitive et la libération d'un nom restent volontairement absentes.

## État Git

- Branche: `session-075-cycle-vie-agents`.
- Base: `origin/main` à `aba60f0`.
- Modifications non commitées dans le worktree isolé.
- Production inchangée.

## Complément avant livraison

La dernière validation a fermé un écart de la matrice UI : pendant une reprise
ou une relance en cours, aucune action concurrente n'est présentée ni acceptée
par le relais HTTP. Le daemon projette désormais `relaunching` tant que la
nouvelle génération n'est pas connectée. Les états historiques actifs `idle`,
`alive` et `dnd` restent compatibles avec l'arrêt et le décommissionnement.

Preuves rejouées après cette correction :

- `node --test crates/bridget-daemon/assets/ui/app.js` : 89 réussis, 0 échec.
- `cargo test -p bridget-daemon --test ui_relay_test` : 24 réussis, 0 échec.
- `cargo test -p bridget-daemon --lib -- --test-threads=1` : 684 réussis,
  0 échec, 7 ignorés.
- `cargo test -p bridget-transport --quiet` : 220 réussis, 1 ignoré.
- `cargo clippy -p bridget-daemon --lib -- -D warnings` avec les trois
  exceptions historiques déjà documentées : réussi.
