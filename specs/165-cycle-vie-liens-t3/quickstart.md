# Quickstart de validation — SPEC-165

## Tests ciblés

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/quickstart.md#extrait-1).

## Qualité locale

```bash
cargo fmt --check
cargo clippy -p bridget-daemon --all-targets -- -D warnings
git diff --check
```

## Observation runtime après activation

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/quickstart.md#extrait-2).

Un sous-agent terminal n'a plus d'identité Bridget tant qu'il reste arrêté. Quand il redémarre, le pont recrée cette identité au prochain snapshot. Ce délai dure normalement trois secondes. Il peut atteindre environ six secondes.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/quickstart.md#extrait-3).

## Critère d’arrêt

Ne pas présenter le correctif comme livré si la validation nécessite encore la limite locale de 16 384 ou si les sous-agents terminés restent connectés.
