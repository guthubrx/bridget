# Preuve RED — SPEC-167

Date : 2026-10-15

Commande :

```text
cargo test -p bridget-daemon spec167 --lib
```

Résultat avant implémentation : code retour 101.

Le compilateur a refusé les cinq oracles. Les symboles suivants étaient absents :

- `parse_delegate_args` ;
- `parse_task_args` ;
- `TaskCliAction` ;
- `render_native_cli_result` ;
- `coordination_identity_error`.

Cette preuve montre que les tests ne validaient pas un comportement préexistant caché.
