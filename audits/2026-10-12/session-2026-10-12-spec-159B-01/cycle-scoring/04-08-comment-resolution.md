Résolution ciblée après C1 — 2026-10-12T19:44:08.770060+00:00

PERF-001, fingerprint 2a54f10c16c5, RESOLVED par lecture fraîche des commentaires ajoutés par ROOT. Source : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159B-edition-file/crates/bridget-daemon/src/mcp.rs ; SHA c13be13df05ffd4d1d417c95bd7c897109f0bff3ce494dc5ce393a8084bf21f6. Plage fraîche : 1300–1308.

Preuve exacte :
```rust
/// Côté client : temps et espace O(b), b = octets traités dans les échanges.
/// Le nombre d'échanges est borné ; le coût du traitement daemon est exclu.
```

L'annotation couvre les octets de l'annuaire optionnel et les enveloppes. Elle borne le calcul local sans annoncer de délai IPC ni de coût daemon. Aucun code exécutable ni test changé par cette correction documentaire. Aucun test, banc ou nouveau scénario exécuté ici.

Les JSON C1 restent historiques et byte-identiques. Cette annexe ne constitue pas deux nouveaux audits complets : seule la plage du commentaire modifié est fraîche ; aucun nouveau crédit de couverture des autres sources. PERF-002 reste LOW non bloquant sur la preuve C1 ; aucune latence mesurée. FULL ROOT en cours, aucun GREEN global ni grade final.
