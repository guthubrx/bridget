# Lot 6 - Blocs `unsafe` justifiés (T069-T080)

But : chaque bloc `unsafe` porte un commentaire `// SAFETY:` qui dit pourquoi l'appel est sûr. La lecture de l'identité (76 fois) passe par une seule fonction. Un lint empêche le retour en arrière. Mesures : `releves.md` section 5.
Branche : `session-260-lot06-unsafe`. Worktree : `.worktrees/260-lot06-unsafe`. Code exécutable modifié seulement par T069 à T072 (appel de fonction au lieu du bloc). Les commentaires ne changent pas le binaire. « Activation en attente » (R8).
Règle de fond : **un commentaire `SAFETY` est une affirmation.** Elle doit être vraie. Lire le code autour avant d'écrire. En cas de doute, écrire la phrase de repli (voir « Repli »), jamais une affirmation non vérifiée.

## T069 - Fonction unique `effective_uid()`

Fichier : `crates/bridget-transport/src/fsutil.rs` (module public `fsutil`, déclaré dans `lib.rs`). Ajouter, avant le premier test du fichier :

```rust
/// Identifiant d'utilisateur effectif du processus courant.
pub fn effective_uid() -> u32 {
    // SAFETY: geteuid n'a aucun argument, ne déréférence aucun pointeur et ne peut pas échouer.
    unsafe { libc::geteuid() }
}
```

Ajouter un test dans le module de tests du fichier (chercher `#[cfg(test)]` dans `fsutil.rs`) :

```rust
#[test]
fn effective_uid_est_le_proprietaire_d_un_fichier_cree_par_ce_processus() {
    use std::os::unix::fs::MetadataExt;
    let chemin = std::env::temp_dir().join(format!("bridget-euid-{}", std::process::id()));
    std::fs::write(&chemin, b"x").unwrap();
    let uid = std::fs::metadata(&chemin).unwrap().uid();
    std::fs::remove_file(&chemin).unwrap();
    assert_eq!(effective_uid(), uid);
}
```

Vérifier : `C-CHECK`, puis `C-CRATE` avec `<crate>` = `bridget-transport` (le nouveau test passe). Ce lot **ajoute** un test : la ligne de base des noms change d'une ligne. Refaire `C-NOMS` à la porte et l'écrire dans le journal (l'écart attendu : +1 ligne, ce nom). Commit : `refactor(transport): Ajouter la fonction unique d'identite effective`.

## T070 - Remplacer les lectures dans le crate de transport

Seulement dans `crates/bridget-transport/src`, hors `fsutil.rs` :

```sh
rg -l 'unsafe \{ libc::geteuid\(\) \}' crates/bridget-transport/src | rg -v 'fsutil.rs$' > /tmp/geteuid-t.txt
xargs perl -pi -e 's/unsafe \{ libc::geteuid\(\) \}/crate::fsutil::effective_uid()/g' < /tmp/geteuid-t.txt
git diff --stat
```

Vérifier : `C-CHECK`. Contrôle : `git diff -U0 | rg '^[+-]' | rg -v 'geteuid|effective_uid|^(\+\+\+|---)'` ne montre aucune ligne. Commit : `refactor(transport): Utiliser la fonction d'identite effective`.

## T071 - Remplacer les lectures dans le crate du démon

Vérifier d'abord l'écriture du chemin : `rg -m1 'bridget_transport::' crates/bridget-daemon/src/daemon.rs`. Puis :

```sh
rg -l 'unsafe \{ libc::geteuid\(\) \}' crates/bridget-daemon > /tmp/geteuid-d.txt
xargs perl -pi -e 's/unsafe \{ libc::geteuid\(\) \}/bridget_transport::fsutil::effective_uid()/g' < /tmp/geteuid-d.txt
```

Inclut `crates/bridget-daemon/tests/` (tests d'intégration). Vérifier : `C-CHECK`, `C-CRATE` (`bridget-daemon`). Même contrôle `git diff -U0` qu'en T070. Commit : `refactor(daemon): Utiliser la fonction d'identite effective`.

## T072 - Variantes sur plusieurs lignes

```sh
rg -n -U 'unsafe\s*\{\s*libc::geteuid\(\)\s*\}' crates --glob '*.rs'
rg -n 'geteuid' crates --glob '*.rs'
```

La seconde commande ne doit plus montrer que les lignes de `fsutil.rs` (la fonction et son commentaire). Pour chaque variante restante : remplacer à la main par l'appel de T070 ou T071 selon le crate. Un `geteuid` dans un test qui compare volontairement avec libc : le laisser et noter. Critère SC08 : 76 lectures remplacées. Commit : `refactor(daemon): Remplacer les dernieres lectures d'identite`.

## T073 à T078 - Commentaires `SAFETY` par groupe de fichiers

Liste des blocs sans commentaire (prend 2 à 4 minutes de compilation) :

```sh
python3 scripts/build.py cargo clippy --offline --locked --workspace --all-targets --features bridget-daemon/test-support -- -A clippy::all -W clippy::undocumented_unsafe_blocks 2>&1 | rg -A2 'undocumented_unsafe_blocks|^warning: unsafe block missing' | rg -- '-->' | sed -E 's/^ *--> //' | sort > "$V/lot06-liste.txt"
wc -l "$V/lot06-liste.txt"
```

Chaque ligne est `fichier:ligne:colonne`. Pour chaque bloc : lire 10 lignes autour, puis insérer **juste au-dessus de la ligne qui contient `unsafe`** une ligne `// SAFETY: ...`. Le commentaire reste collé au bloc (aucune ligne vide entre les deux). Un bloc dans une expression sur plusieurs lignes : mettre le commentaire sur la ligne précédant l'instruction.

Phrases modèles (adapter, ne pas copier sans lire) :

| Appel | Commentaire |
|-------|-------------|
| `libc::kill(pid, 0)` | `// SAFETY: kill(pid, 0) est l'idiome POSIX de sonde d'existence ; aucun signal n'est envoyé.` |
| `libc::kill(pid, signal)` | `// SAFETY: kill ne déréférence aucun pointeur ; pid et signal sont des entiers.` |
| `libc::close(fd)` | `// SAFETY: fd appartient à ce code et n'est plus utilisé après la fermeture.` |
| `libc::fcntl(fd, cmd, ...)` | `// SAFETY: fd est ouvert pendant l'appel ; la commande n'utilise aucun pointeur.` (avec pointeur : « le pointeur désigne une structure valide pour la durée de l'appel ») |
| `libc::flock(fd, op)` | `// SAFETY: fd est un descripteur ouvert ; flock ne déréférence aucun pointeur.` |
| `libc::poll(fds, n, t)` | `// SAFETY: fds pointe vers n éléments valides pendant l'appel.` |
| `libc::write` / `read` | `// SAFETY: le tampon est valide pour len octets pendant l'appel.` |
| `from_raw_fd(fd)` | `// SAFETY: fd est ouvert et ce code en est l'unique propriétaire ; la valeur en prend la propriété.` |

Chaque phrase doit être **vraie pour ce bloc**. Vérifier : le descripteur est-il ouvert ? Le tampon est-il assez long ? Le pointeur vit-il assez ? Sinon utiliser le repli.

**Repli** : si la lecture locale ne permet pas d'établir l'invariant : `// SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).` Noter le fichier dans le journal, section « Idées ». Ne jamais inventer.

| Tâche | Fichiers (depuis la liste `lot06-liste.txt`) |
|-------|----------------------------------------------|
| T073 | `workspace_commit_view.rs` et `claude_interactive.rs` |
| T074 | `managed_process.rs` et `attach.rs` |
| T075 | `daemon.rs` et le dossier `daemon/` |
| T076 | tout le crate `bridget-transport` |
| T077 | les autres fichiers de `crates/bridget-daemon/src` |
| T078 | `crates/bridget-daemon/tests/` |

Après chaque tâche : `C-CHECK`, puis relancer la commande de liste : le nombre de lignes baisse du nombre de blocs traités. Un commentaire ne doit pas changer une ligne de code : contrôle `git diff -U0 | rg '^\+' | rg -v '^\+\+\+|^\+\s*//'` ne montre aucune ligne de code ajoutée. Commit par tâche : `docs(unsafe): Justifier les blocs unsafe de <groupe>`.

## T079 - Lint permanent

Dans `Cargo.toml` (racine), section `[workspace.lints.clippy]` (créée au lot 1), ajouter `undocumented_unsafe_blocks = "warn"`. Si le lot 1 n'est pas intégré : ajouter la section et les lignes `[lints] workspace = true` des 3 crates (voir guide du lot 1, T009).
Vérifier : `C-CLIPPY` ne montre plus aucun avertissement `undocumented_unsafe_blocks` (critère SC07 : 247 sur 247). Contre-épreuve : ajouter temporairement `unsafe { libc::getpid() };` dans un fichier de test, lancer `C-CLIPPY`, constater l'avertissement, retirer la ligne.
Commit : `chore(lint): Exiger un commentaire SAFETY sur chaque bloc unsafe`.

## T080 - Porte du lot

1. `C-TESTS` (`<nom>` = `lot06`), `C-ECHECS` : aucune différence.
2. `C-NOMS` : exactement une ligne de plus que la base (le test de T069). Mettre à jour `validation/base-noms.txt` avec le nouveau fichier et le dire dans le journal (ligne de base actualisée).
3. Bilan dans `implementation.md` : nombre de blocs commentés, nombre de replis, « activation en attente ». Fin de lot (`quickstart.md`).

## Arrêts propres à ce lot

- Une phrase `SAFETY` ne se vérifie pas : repli, jamais d'invention.
- Le remplacement de masse (T070, T071) change autre chose que `geteuid` : `git checkout -- <fichier>` et traiter ce fichier à la main.
- Interdit : introduire `nix`, `rustix` ou toute dépendance (R6.7). Interdit : envelopper `kill` ou `fcntl` dans de nouvelles fonctions (aucune abstraction à moins de 3 usages communs identiques).
