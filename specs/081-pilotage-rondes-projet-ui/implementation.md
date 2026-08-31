# Implémentation - Pilotage des rondes par projet dans l'interface

## Résultat

Statut: terminé dans le worktree, non committé, non mergé et non déployé.

La politique autoritaire de la SPEC-079 est maintenant lisible et pilotable depuis le menu projet de la SPEC-080. Le navigateur ne change jamais localement la valeur de ronde avant confirmation du daemon.

## Réalisation

| Zone | Réalisation |
|---|---|
| Contrat | Ajout d'un résultat fermé `deposited`, `refused` ou `indeterminate` et de trois faits optionnels de dernier passage. |
| Store | Migration additive de `project_round_policies`, lecture compatible des anciennes lignes et écriture monotone sur `project_id + binding_generation`. |
| Daemon | Classement et persistance du verdict après passage par le socle idempotent existant. |
| Relais UI | Négociation obligatoire de `ProjectRoundPolicyV1`, jointure O(p) avec les projets et route stricte `POST /v1/projects/round`. |
| Navigateur | Case d'état dans le menu existant, état occupé, erreur sans mutation optimiste, détail du dernier passage et délai maximal du prochain cycle. |
| Présentation | Suffixe discret `actif · ronde activée` et styles limités au menu existant. |

## Réutilisation

- `ProjectRoundRequest`, `ProjectRoundProjection` et l'idempotence de la SPEC-079.
- `project_round_policies` et sa clé primaire existante.
- Le timer global de 420 secondes, sans nouveau timer.
- `GET /v1/projects`, `refreshProjects` et le menu projet de la SPEC-080.
- Les trois accès existants au menu: trois points, clic droit et `Maj + F10`.

Aucune dépendance, table, écran, scheduler, commande libre ou branche fournisseur n'a été ajouté.

## Déroulement TDD et corrections

1. Les premiers tests Node ont échoué car `projectRoundView` et `buildProjectRoundMutation` n'existaient pas.
2. Les premiers tests Rust ont échoué car le résultat fermé, les champs de projection et l'écriture du store n'existaient pas.
3. Pendant la revue, une preuve de perte de réponse a révélé que le relais renvoyait `daemon_unavailable` au lieu du code contractuel `round_service_unavailable`. La traduction a été corrigée dans la frontière ronde, puis la preuve est passée.
4. Deux filtres du quickstart ne sélectionnaient aucun test. Ils ont été remplacés par les identifiants exacts afin que chaque commande atteste réellement une preuve.

## Tests finaux

| Commande | Résultat |
|---|---|
| `cargo test -p bridget-transport spec_079_contrat_ronde_projet_est_ferme_versionne_et_rejouable --lib` | PASS - 1 test |
| `cargo test -p bridget-daemon spec_079_politique_ronde_absente_idempotente_et_epinglee_au_rebind --lib -- --test-threads=1` | PASS - 1 test |
| `cargo test -p bridget-daemon spec_081 --lib -- --test-threads=1` | PASS - 6 tests |
| `cargo test -p bridget-transport --lib` | PASS - 230 réussis, 1 ignoré |
| `cargo test -p bridget-daemon --lib -- --test-threads=1` | PASS - 758 réussis, 7 ignorés |
| `cargo test -p bridget-daemon --test ui_relay_test -- --test-threads=1` | PASS - 25 tests |
| `node --check crates/bridget-daemon/assets/ui/app.js` | PASS |
| `node crates/bridget-daemon/assets/ui/app.js` | PASS - 101 tests |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `git diff --check` | PASS |

## Garanties vérifiées

- Une génération obsolète, un projet inactif, une capacité absente et une réponse perdue sont refusés sans projection inventée.
- Les trois faits de dernier passage sont tous présents ou tous absents.
- Un rebind présente une génération non configurée et sans dernier passage.
- Une occurrence ancienne ne remplace pas une occurrence récente.
- La réponse UI ne contient aucun secret, profil, fournisseur ou contenu de message supplémentaire.
- Désactiver la ronde ne touche ni aux agents, ni aux exécutions, ni à leurs contrôles.

## Validation non exécutée

La validation visuelle sur le daemon de production n'a pas été exécutée, car ce pipeline s'arrête volontairement avant commit, merge et déploiement. Les étapes opérateur restent décrites dans `quickstart.md`.
