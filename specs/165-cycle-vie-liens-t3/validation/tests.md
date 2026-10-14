# Validation des tests — SPEC-165

Date : 2026-10-14  
Worktree : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3`

## Résultat ciblé

| Contrôle | Résultat |
|---|---|
| `rustfmt --edition 2024` sur les trois fichiers Rust affectés | RC 0 |
| `git diff --check` | RC 0 |
| `cargo test -p bridget-daemon spec165_ -- --nocapture` | 13 PASS, 0 FAIL |
| `cargo test -p bridget-daemon 't3code::tests' -- --nocapture` | 102 PASS, 0 FAIL |
| `cargo test -p bridget-daemon t3code_contract -- --nocapture` | 22 PASS, 0 FAIL, 1 ignored |

La panique affichée par `spec165_panique_worker_ferme_et_joint_le_lecteur_une_fois` est volontaire. Le test vérifie que le propriétaire ferme le flux et joint le lecteur après cette panique.

## Preuve RED

Les premiers oracles ont échoué à la compilation avant le code de production. `ThreadSummary` ne portait pas `relationship_to_parent`. Le prédicat de conservation n'existait pas. Les mêmes oracles sont verts après l'implémentation.

## Suite complète du crate

Commande : `cargo test -p bridget-daemon`

Résultat final après le correctif : 1 944 PASS, 91 FAIL, 16 ignored, RC 101.

Ce résultat n'est pas déclaré GREEN. Les 91 échecs sont hors des trois fichiers affectés. Les causes visibles sont déjà présentes dans l'environnement de test global : chemins de sockets Unix trop longs, différence macOS `/private/var` contre `/var`, fixtures de migration absentes, environnement fournisseur non apte et matrices de commandes historiques désynchronisées. Le test T3 historique `spec147_native_t3_refresh_publishes_and_revokes_real_os_processes` échoue aussi sur `SUN_LEN`. Les 102 tests du module affecté et les 23 tests de contrat ne reproduisent aucun de ces échecs.

## Clippy

Commande finale : `cargo clippy -p bridget-daemon --all-targets -- -D warnings`

Résultat : RC 101 sur 18 erreurs héritées dans `crates/bridget-transport`. Elles concernent notamment la complexité de types, des expressions booléennes, des variantes d'énumérations volumineuses et des blocs conditionnels historiques. Aucun message Clippy ne cible les trois fichiers Rust modifiés par le correctif 165.

## Couverture comportementale

Les tests affectés prouvent :

- le parsing V1 compatible et la filiation V2 stricte ;
- la conservation des racines, forks et sous-agents actifs ;
- l'absence de 1 000 liens pour 1 000 enfants terminaux ;
- le drain borné et cinq lectures silencieuses ;
- les obligations persistées et le plafond 32 sur 100 ;
- la fermeture des liens absents du snapshot ;
- la consommation d'une tentative seulement après une fermeture attestée ;
- l'absence de famine quand 32 reprises sont en délai de reconnexion ;
- l'invalidation de la quiescence dès la lecture réseau, sans republication entre le lecteur et le worker ;
- l'acceptation de `lineage: null` et le refus des relations inconnues ;
- l'attente d'un nouveau snapshot après un dispatch accepté ou incertain ;
- la fermeture et la jointure du lecteur après une panique du worker.
