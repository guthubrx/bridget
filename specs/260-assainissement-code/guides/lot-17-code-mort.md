# Lot 17 - Code mort (T151-T154)

But : traiter les 9 attributs `#[allow(dead_code)]` (`releves.md` section 12) : chaque élément est utile et visible du compilateur, ou il porte une raison écrite. Aucun code n'est supprimé.
Branche : `session-260-lot17-code-mort`. Worktree : `.worktrees/260-lot17-code-mort`. Code exécutable : attributs seulement. « Activation en attente » (R8).
Règle de fond : **ne rien supprimer** (spec : FR27 et D12). Un élément inutilisé qui reste reçoit une raison.

## T151 - Constat

```sh
rg -n 'allow\(dead_code\)' crates --glob '*.rs' | tee "$V/lot17-avant.txt"
wc -l "$V/lot17-avant.txt"          # attendu : 9 (peut varier de 1 ou 2)
```

Lieux attendus : `workspace_git.rs` ; `native_permissions.rs` ; `provider_accounts.rs` ; `attach.rs` (3, boucle T806b) ; `remote_worker.rs` (2, dans des tests) ; `daemon/native_delegation.rs`. Si le lot 4 est intégré, ceux qui étaient dans des tests sont dans des fichiers `*_tests.rs`. Commit : `docs(specs): Relever les attributs dead_code`.

## T152 - Traiter chaque attribut (méthode)

Pour chaque ligne de `lot17-avant.txt`, **hors `attach.rs`** (voir T153) :

1. Lire l'élément sous l'attribut (fonction, champ, constante) et chercher ses usages : `rg -n '\b<nom>\b' crates --glob '*.rs'`.
2. Retirer la ligne `#[allow(dead_code)]` et lancer `C-CHECK`.
   - Aucun avertissement `never used` : l'attribut était inutile. Laisser retiré.
   - Avertissement, et l'élément n'est utilisé **que dans des tests** : le faire précéder de `#[cfg(test)]`. Vérifier que les tests compilent encore (`C-CHECK` compile les tests).
   - Avertissement, élément utilisé nulle part : le conserver avec `#[expect(dead_code, reason = "<raison>")]`. Raison lue dans le code ou l'historique (`git log -S'<nom>' --oneline | tail -1`) ; sinon : `reason = "raison non établie (programme 260) : à revoir"`. `expect` (Rust 1.81 et plus ; le projet est en 1.92) échoue s'il n'y a plus d'avertissement, ce qui évite l'oubli.
3. Commit par fichier : `refactor(<crate>): Traiter l'attribut dead_code de <fichier>`.

## T153 - `attach.rs` : boucle T806b (D12)

Les 3 attributs de `attach.rs` protègent du code réservé à la « boucle T806b ». Action par défaut de D12 : **conserver**. Pour chacun, remplacer `#[allow(dead_code)]` par `#[expect(dead_code, reason = "code réservé à la boucle T806b, conservé par décision D12 (programme 260)")]`. `C-CHECK` : si un avertissement `unfulfilled_lint_expectations` apparaît (l'élément est en fait utilisé), retirer l'attribut. Lire D12 dans `decisions.md` : si `tranchée` avec « supprimer », **ne pas supprimer ici** : écrire « suppression à spécifier » dans le journal et le dire à François. Commit : `refactor(daemon): Documenter le code reserve de la boucle T806b`.

## T154 - Porte du lot

1. `rg -c 'allow\(dead_code\)' crates --glob '*.rs'` : aucune ligne, ou chaque restant est justifié dans le journal.
2. `C-CLIPPY` (`-- -D warnings` si le lot 1 est intégré) : aucune alerte nouvelle.
3. `C-TESTS` (`<nom>` = `lot17`), `C-ECHECS`, `C-NOMS` : aucune différence (un `#[cfg(test)]` ajouté ne change pas les noms de tests).
4. Bilan : par attribut, le sort (retiré, `cfg(test)`, `expect` avec raison). « activation en attente ». Fin de lot (`quickstart.md`).

## Arrêts propres à ce lot

- Un retrait d'attribut fait apparaître plus de 5 avertissements d'un coup dans un même fichier : traiter par `expect` sur les éléments concernés, un par un ; ne pas supprimer de code.
- Un élément est utilisé par un autre crate ou par des tests d'intégration (`crates/*/tests/`) : le compilateur ne l'avertit pas ; dans ce cas l'attribut est inutile et on le retire.
