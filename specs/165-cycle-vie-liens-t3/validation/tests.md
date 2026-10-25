# Validation des tests — SPEC-165

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/validation/tests.md#extrait-1).

## Résultat ciblé

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/validation/tests.md#extrait-2).

La panique affichée par `spec165_panique_worker_ferme_et_joint_le_lecteur_une_fois` est volontaire. Le test vérifie que le propriétaire ferme le flux et joint le lecteur après cette panique.

## Preuve RED

Les premiers oracles ont échoué à la compilation avant le code de production. `ThreadSummary` ne portait pas `relationship_to_parent`. Le prédicat de conservation n'existait pas. Les mêmes oracles sont verts après l'implémentation.

## Suite complète du crate

Commande : `cargo test -p bridget-daemon`

Résultat final après le correctif : 1 944 PASS, 91 FAIL, 16 ignored, RC 101.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/validation/tests.md#extrait-3).

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
