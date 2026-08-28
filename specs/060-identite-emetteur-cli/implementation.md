# Journal d’implémentation — Session 060

## Métadonnées

- **Spec** : `060-identite-emetteur-cli`
- **Branche** : `session-060-identite-emetteur-cli`
- **Base gelée** : `70ef619`
- **Démarré / terminé** : 2026-08-28

## Fichiers modifiés

- `crates/bridget-core/src/message.rs` — champ `from_declared`
- `crates/bridget-daemon/src/cli.rs` — pose du drapeau, aide et usage
- `crates/bridget-daemon/src/daemon.rs` — décision d’attribution et refus

## Décision de conception

Le daemon ne pouvait pas distinguer un `from` explicite d’un `from` de repli :
`cli.rs` faisait `from.unwrap_or_else(current_agent_name)`, si bien que le champ
était toujours rempli. Refuser sur la seule valeur du nom aurait cassé les
agents lancés avec `BRIDGET_AGENT_NAME` hors annuaire.

Un champ `from_declared` a donc été ajouté à l’enveloppe, avec
`#[serde(default)]` : absent du flux, il vaut `false`. Un émetteur qui n’a pas
été recompilé continue d’être traité comme avant. La compatibilité ascendante
est portée par serde, non par une convention de nom.

La décision a été extraite du `match` du daemon vers une fonction pure,
`resolve_sender_attribution`, pour que l’assertion métier soit mutable et
observable. Complexité O(1), sans allocation.

## Preuves

Conformes à la contrainte du mandat : témoin, mutant, restauration SHA-256.

- **Témoin nominal** : `4 passed / 0 failed / 599 filtered`
  (témoin métier seul : `1 / 0 / 602`).
- **Mutant causal** : `RefuseUnaddressable` → `UseConnectionName` dans
  `resolve_sender_attribution`, c’est-à-dire le rétablissement exact de
  l’écrasement silencieux d’origine. Témoin : `0 passed / 1 failed / 602`.
  Le témoin meurt sur l’assertion métier, et sur elle seule.
- **Restauration** : SHA-256 de `daemon.rs` avant mutation et après
  restauration : `3cce3376126e11486e66f4da81c693f5016daa4c7f2da8a0f234b564e063a59a`,
  identique. SHA-256 sous mutant :
  `817d314d4da76da73ad3bf81ff8d4a45262b19ecad9fff501aa3b400b100b1b9`.
  Témoins re-mesurés verts après restauration : `4 / 0 / 599`.

## Non-régression

- `cargo check --workspace --all-targets` : vert.
- `rustfmt --edition 2024 --check` sur les trois fichiers : conforme.
- `cargo test --workspace --lib` **avec** le delta : `585 passed / 11 failed`.
- `cargo test --workspace --lib` **sans** le delta, sur `70ef619` nu
  (delta mis de côté par `git stash`) : `581 passed / 11 failed`.
- Les 11 échecs sont **les mêmes** dans les deux mesures
  (`daemon::presence_tests` ×10, `wrapper::prompt_tests` ×1). Ils préexistent à
  ce correctif et ne lui sont pas imputables. Le delta ajoute 4 tests verts et
  n’en casse aucun.

## Non attesté

- **Le correctif n’est pas en vigueur sur le daemon du parc.** Il exige une
  recompilation et un redémarrage du daemon, qui sert 22 agents connectés. Ce
  redémarrage n’a pas été fait et relève d’une décision humaine.
- Les bancs d’intégration instanciant un daemon n’ont pas été lancés, pour ne
  pas interférer avec le daemon en production. Seule la suite `--lib` a été
  mesurée.
- L’intégration de cette branche dans `main` n’est pas attestée par moi.
