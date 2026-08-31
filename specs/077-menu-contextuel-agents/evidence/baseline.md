# Ligne de base - SPEC-077

Date: 2026-08-30
Commit de départ: f532a1bcd805d764fc97c1a6f7225f2449178451

## Tests Node

Commande:

```bash
node --test crates/bridget-daemon/assets/ui/app.js
```

Résultat: PASS, 89 tests passants, 0 échec.

## Tests Rust UI

Commande prévue:

```bash
cargo test -p bridget-daemon ui --lib -- --test-threads=1
```

Résultat: NON EXÉCUTÉ dans le shell SSH courant, car `cargo` n'est pas dans
le PATH de cet environnement. Ce point est un obstacle d'environnement, pas
un échec produit. La commande sera retentée avec l'environnement Rust du
projet pendant la validation finale.
