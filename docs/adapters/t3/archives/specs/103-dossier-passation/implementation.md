# Archives techniques de l’adaptateur T3

Document d’origine : [specs/103-dossier-passation/implementation.md](../../../../../../specs/103-dossier-passation/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Prompt 105 des wrappers sans « ; » (tests `claude_interactive_097_test`), exemple de
réponse de SKILL.md raccourci (`core_089_skill_test`), liste attendue de
`claude_native_permissions_test` complétée (`bridget_events`, `bridget_journal`),
emprunt inutile t3code.rs:2780 (clippy).

<a id="extrait-2"></a>

## Extrait 2

### T020 Recette complète (état définitif du code, 2026-09-17 22:52–22:58 CEST)
- ✅ Environnement : `umask 077`, `env -i HOME TMPDIR=/tmp/b.XXXX BRIDGET_HOME=/tmp/b102/ns
  PATH=~/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin`, `perl -e 'alarm 1800'`.
- `cargo fmt --all -- --check` : rc 0 ; `cargo clippy --workspace --all-targets -- -D warnings` : rc 0.
- `cargo test --workspace --release --no-fail-fast` : **1426 réussis, 0 échec, 50 ignorés**
  (/tmp/b102/r103c-release.log). Une première passe (22:39–22:44, avant les deux corrections
  de la contre-revue) avait donné le même bilan 1426/0/50 ; une relance intermédiaire dont le
  journal mélangeait deux exécutions a été refaite proprement.
- Ignorés : recettes réelles Codex/Claude/T3 conditionnées par variables d'environnement et
  worker de performance privé du harnais.
