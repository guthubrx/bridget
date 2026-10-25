# Archives techniques de l’adaptateur T3

Document d’origine : [specs/165-cycle-vie-liens-t3/validation/tests.md](../../../../../../../specs/165-cycle-vie-liens-t3/validation/tests.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-14
Worktree : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3`

<a id="extrait-2"></a>

## Extrait 2

| Contrôle | Résultat |
|---|---|
| `rustfmt --edition 2024` sur les trois fichiers Rust affectés | RC 0 |
| `git diff --check` | RC 0 |
| `cargo test -p bridget-daemon spec165_ -- --nocapture` | 13 PASS, 0 FAIL |
| `cargo test -p bridget-daemon 't3code::tests' -- --nocapture` | 102 PASS, 0 FAIL |
| `cargo test -p bridget-daemon t3code_contract -- --nocapture` | 22 PASS, 0 FAIL, 1 ignored |

<a id="extrait-3"></a>

## Extrait 3

Ce résultat n'est pas déclaré GREEN. Les 91 échecs sont hors des trois fichiers affectés. Les causes visibles sont déjà présentes dans l'environnement de test global : chemins de sockets Unix trop longs, différence macOS `/private/var` contre `/var`, fixtures de migration absentes, environnement fournisseur non apte et matrices de commandes historiques désynchronisées. Le test T3 historique `spec147_native_t3_refresh_publishes_and_revokes_real_os_processes` échoue aussi sur `SUN_LEN`. Les 102 tests du module affecté et les 23 tests de contrat ne reproduisent aucun de ces échecs.
