# Quickstart de validation — SPEC-165

## Tests ciblés

```bash
cargo test -p bridget-daemon t3code_contract
cargo test -p bridget-daemon 't3code::tests'
```

## Qualité locale

```bash
cargo fmt --check
cargo clippy -p bridget-daemon --all-targets -- -D warnings
git diff --check
```

## Observation runtime après activation

1. Relever le PID exact du pont avec `launchctl print gui/$(id -u)/com.bridget.t3`.
2. Vérifier que le PID correspond à `bridget t3 serve` et non à Firefox.
3. Compter les fichiers ouverts avec `lsof -p <PID>`.
4. Vérifier le statut avec `/Users/moi/.local/bin/bridget t3 status`.
5. Vérifier dans `/Users/moi/Library/Logs/com.bridget.t3.log` :
   - présence du fil courant ;
   - absence de `Too many open files` après la ligne de démarrage ;
   - absence de refus d’authentification ;
   - nombre de liens cohérent avec les conversations principales et le travail actif.
6. Vérifier qu'un enfant actif termine, publie son dernier résultat, puis disparaît du compteur après son drain.

Un sous-agent terminal n'a plus d'identité Bridget tant qu'il reste arrêté. Quand il redémarre, le pont recrée cette identité au prochain snapshot. Ce délai dure normalement trois secondes. Il peut atteindre environ six secondes.

Un message Bridget direct ne redémarre pas un sous-agent arrêté. Il est refusé
jusqu'à ce que T3 réactive le sous-agent. Un sous-agent sans premier tour suit
la même règle.

## Critère d’arrêt

Ne pas présenter le correctif comme livré si la validation nécessite encore la limite locale de 16 384 ou si les sous-agents terminés restent connectés.

