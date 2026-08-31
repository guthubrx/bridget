# Implémentation - SPEC-081

## Résultat

Les identités de routage sont désormais des `agent_id` UUID v4 canoniques. Le
`display_name` est l'unique nom de présentation. Les anciens aliases ne sont
utilisés que par la migration puis sont supprimés. Les profils, l'API UI,
Maicie, les ordres de spawn et le runtime Docker transportent `agent_id`.

La projection vers un fournisseur conserve l'Agent ID pour le routage et
remplace uniquement l'émetteur du prompt par le `display_name`.

## Vérifications effectuées

- `cargo fmt --check`
- `cargo test --workspace --no-run`
- `cargo test -p bridget-core router --lib -- --test-threads=1`
- `cargo test -p bridget-transport --lib -- --test-threads=1`
- `cargo test -p bridget-daemon agent_profile::tests --lib -- --test-threads=1`
- `cargo test -p bridget-daemon ui::tests --lib -- --test-threads=1`
- `cargo test -p bridget-daemon --test identity_migration_test -- --test-threads=1`
- `cargo test -p maicie --test identity_migration -- --test-threads=1`
- `cargo test -p maicie --test bridget_client_contract --test approval_atomicity_integration -- --test-threads=1`
- `node crates/bridget-daemon/assets/ui/app.js`

Toutes ces commandes passent.

## Dette de tests historique

`cargo test -p bridget-daemon --lib -- --test-threads=1` exécute encore des
scénarios antérieurs à ce contrat et construit directement des noms libres
(`agent-2`, `cursor5`) à la place d'UUID v4. Ils échouent maintenant dès la
garde d'identité, ce qui atteste le refus mais demande une conversion
mécanique séparée de ces fixtures. Le code de production n'assouplit pas la
validation pour les faire passer.
