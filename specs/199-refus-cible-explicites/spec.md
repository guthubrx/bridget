# Session199 — Refus de cible explicites

Correction autorisée : distinguer les contrôles ayant produit routing_connection_changed et conserver leur diagnostic au rejeu. Le code public reste inchangé. Les décisions et gardes restent strictement identiques. Aucun secret, hash privé ou contexte non autorisé dans le diagnostic. Les refus historiques sans détail restent inconnus.

Incident : opus2d-sys-02-suite-production-20261028-01, refus durable sans tâche à1793201161. Le worker a répondu à sa sonde. Le maître183 ne conserve pas le champ divergent ; ne pas inventer la cause historique ni rouvrir ce reçu.

Tests : contrôles ciblés Rust natifs, projection stable, causes fermées, persistance/rejeu et ancien reçu, aucun effet métier avant refus. Pas de nouvelle mission Opus2D ni de modification de store vivant.
