# Registre des dépréciations

Ce registre recense les chemins maintenus temporairement pendant une migration.
Une entrée est supprimée avec son chemin dès que la condition indiquée est
atteinte.

| Chemin | Remplacé par | Supprimable quand |
|---|---|---|
| Liste blanche de binaires dans `crates/bridget-daemon/src/wrapper.rs` | Registre `~/.config/bridget/agents.json` | Supprimée en T703 |
