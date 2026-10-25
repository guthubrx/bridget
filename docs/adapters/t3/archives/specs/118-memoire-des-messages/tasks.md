# Archives techniques de l’adaptateur T3

Document d’origine : [specs/118-memoire-des-messages/tasks.md](../../../../../../specs/118-memoire-des-messages/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

# Tâches 118
Statut: Implemented — 6/6, livré le 2026-09-25 à 03:36.
- [x] T001 Diagnostic sur données réelles : source qui bat, file saturée, 131 copies par message, borne de 1 000 sous 1 363 messages.
- [x] T002 `ThreadState::forget_invisible` : oubli limité à ce que T3 ne montre plus ; `remember` n'évince plus ; index des messages recopiés (`t3code.rs`).
- [x] T003 Annonce « prête » anticipée réservée aux lectures qui portent une lacune (`t3code.rs`).
- [x] T004 Tests `spec118_*` (3), chacun vérifié en échec sur l'ancien code pour les deux de bout en bout.
- [x] T005 fmt, clippy `-D warnings` OK ; recette complète 1550 réussis, 0 échec, 52 ignorés ; 0 fixture restante.
- [x] T006 Fusion, construction, relance du pont, vérification sur `opus-city-glmkrkr4`.
