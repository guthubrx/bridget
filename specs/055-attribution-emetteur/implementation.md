# Journal d’implémentation — Session 055

## Métadonnées

- **Branche** : `session-055-attribution-emetteur`
- **Base** : `2750bdf6889e1f664fff11a72a4719d00509e599`
- **Objectif** : `20c7a2a8-4bff-49b5-b1ca-1ffe1145ee92`

## Correctif

Les pilotes Codex et Claude ajoutent `from` au payload de `turn_start`. Pour
les journaux historiques, Attach remplace l’en-tête initial non attesté si le
`prompt_dispatched` du même tour apporte ensuite un `from` attesté. Si aucune
source du tour ne porte cette provenance, le renderer affiche explicitement
`émetteur non attesté` plutôt que d’inférer `humain`.

Deux témoins de traversée lancent les pilotes réels sur des fournisseurs de
test, lisent le JSONL de `JournalWriter`, puis donnent la même ligne à Attach.
Ils vérifient ensemble le payload `from=jc2`, l’en-tête `jc2 →` et l’absence
d’étiquette `humain →`.

Un témoin de rendu rejoue dans le même `BlockRenderer` un tour historique
enrichi et un tour sans provenance. Il vérifie les deux issues : correction de
l’en-tête par `prompt_dispatched.from=jc2`, puis libellé non attesté quand ce
champ est absent partout.

## Mesures

- Univers des témoins ajoutés : 2 (Codex et Claude).
- Nominal : `2 passed; 0 failed; 0 ignored; 0 measured; 594 filtered out`.
- Mutant Codex, retrait de `from` de `turn_start` :
  `0 passed; 1 failed; 0 ignored; 0 measured; 595 filtered out`, à
  l’assertion `turn_start doit persister l’émetteur réel`.
- Mutant Claude, même retrait :
  `0 passed; 1 failed; 0 ignored; 0 measured; 595 filtered out`, à la même
  assertion métier.
- Univers du témoin historique : 1 test.
- Nominal historique :
  `1 passed; 0 failed; 0 ignored; 0 measured; 596 filtered out`.
- Mutant historique A, qui ignore le `prompt_dispatched` enrichi :
  `0 passed; 1 failed; 0 ignored; 0 measured; 596 filtered out`, à
  l’assertion exigeant l’en-tête `jc2 →`.
- Mutant historique B, qui rétablit le repli `humain` :
  `0 passed; 1 failed; 0 ignored; 0 measured; 596 filtered out`, à
  l’assertion exigeant le libellé `émetteur non attesté`.
- Restauration : empreintes SHA-256 des trois fichiers de code identiques aux
  empreintes prises avant les mutations, puis rejeu nominal vert.
- Non-régression ciblée : Codex `28/0/0`, Claude `19/0/0`, Attach `54/0/0`.
- `cargo check --workspace --all-targets` : vert, avec avertissements
  préexistants hors périmètre.

## Limite déclarée

`cargo fmt --all --check` reste rouge hors périmètre sur
`crates/bridget-daemon/src/reaper.rs` et `crates/bridget-transport/src/acp.rs`.
Le correctif ne les modifie pas.
