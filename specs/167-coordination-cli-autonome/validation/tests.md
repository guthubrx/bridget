# Validation — SPEC-167

Date : 2026-10-15

## Oracles ciblés

| Commande | Résultat |
|---|---|
| `cargo test -p bridget-daemon cli::hook_tests --lib` | 50 PASS |
| `cargo test -p bridget-daemon delegation_mcp --lib` | 11 PASS |
| `cargo test -p bridget-daemon --test lineage_cli_test` | 9 PASS, 1 ignored |
| `cargo test -p bridget-daemon --test native_delegation_e2e` | 16 PASS, 1 ignored |
| `cargo test -p bridget-daemon spec094_inventaire_documente_toutes_les_commandes_du_repartiteur --lib` | 1 PASS après correction de l’inventaire |
| `git diff --check` | code retour 0 |

Total sans double compte : 87 PASS et 2 ignored.

Le test `native148_sans_t3_retry_reponse_correllee_annulation_et_reprise_durable` passe dans la recette E2E. Il couvre la création, le rejeu, la réponse corrélée, l’annulation et la reprise sans T3. Les tests SPEC-167 couvrent la traduction de chaque nouvelle sous-commande vers ce même moteur.

## Contrôles du binaire de développement

| Commande | Résultat |
|---|---|
| `target/debug/bridget task --help` | code retour 0, aide complète |
| `target/debug/bridget delegate --json` | code retour 2, un objet JSON `invalid_arguments` |
| Invocation `delegate` sans identité, environnement minimal | code retour 1, objet JSON `identity_not_found`, aucune tâche créée |

## Suite globale

`cargo test --workspace` a rendu le code retour 101 après 290,92 s : 1 949 PASS, 91 FAIL et 16 ignored.

Un échec appartenait au diff : l’inventaire de sécurité ne documentait pas encore les nouvelles racines. Il a été corrigé. Son test ciblé passe. Les 90 autres échecs sont hors diff ou liés à l’environnement : chemins de socket macOS trop longs, alias `/private/var`, fixture de migration 159 absente, préflights fournisseurs et baselines historiques. La suite globale complète n’a pas été rejouée après le correctif documentaire.

## Format et lint

- `rustfmt --edition 2024 crates/bridget-daemon/src/cli.rs` : conforme.
- `cargo fmt --all --check` : non utilisé comme succès global, car le dépôt contient des fichiers hérités non formatés hors diff.
- `cargo clippy -p bridget-daemon --lib` : bloqué par `clippy::never_loop` dans `crates/bridget-daemon/src/daemon/native_lineage.rs`, hors diff.
- `cargo clippy -p bridget-daemon --lib -- -A clippy::never_loop` : code retour 0, 89 avertissements hors diff, aucun dans les lignes SPEC-167.

## Audit

Audit v14 en lecture seule : note A, aucun finding, validateur 0 erreur et 0 avertissement.

Chemin : `/Users/moi/Nextcloud/10.Scripts/64.bridget/audits/2026-10-15/session-2026-10-15-spec-167-01`.
