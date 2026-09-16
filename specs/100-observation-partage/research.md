# Recherche 100

## Décisions et alternatives

- Extrait : réutiliser attach (Subscribe/Tail/Seq/SnapshotCaughtUp) plutôt que
  relire le journal par chemin arbitraire. Réassemblage existant dans attach.rs
  accept_fragment ; budget existant dans communication/client.rs. Maintenance :
  un seul comportement d'erreurs de journal, partage via send existant.
- Abonnements : état local borné, pas extension du flux guichet CoordinationSubscribe
  dont la responsabilité est différente. Aucune nouvelle table : ce ne sont
  pas des obligations durables comme les demandes reply. Limite de redémarrage
  annoncée dans les reçus et documents.
- Collisions : métadonnées structurées avant troncature des arguments (512
  caractères dans certains producteurs). Refus de la déduction par texte libre.
  Fenêtre 30 secondes et chemins/hôtes distincts ; risque observé, pas conflit
  prouvé. Comparaison lexicale ; liens symboliques non résolus.
- Aucune nouvelle dépendance ni machine de workflow. Filtre de chemin volontairement
  simple : * et texte exact, pas regex/SQL arbitraire.

## Recherche externe ciblée — 2026-09-16

Baselines consultées : /Users/moi/.speckit/research/01-ai-agents-agentic-ai.md,
/Users/moi/.speckit/research/04-architectures-patterns.md,
/Users/moi/.speckit/research/08-testing-quality.md.
Pas de métrique commerciale utilisée pour justifier ces fonctionnalités.

- https://www.anthropic.com/engineering/building-effective-agents : mécanismes
  simples et composables ; ici un fait filtré, pas un agent évaluateur permanent.
- https://arxiv.org/abs/2503.13657 : terminaison et vérification sont des sources
  d'échecs multi-agents ; ici fin de tour explicitement distincte de réussite.
- https://modelcontextprotocol.io/specification/2025-06-18/server/tools : schémas
  d'entrée et erreurs explicites ; rester compatible avec MCP existant du projet.
- https://martinfowler.com/articles/practical-test-pyramid.html : tests de
  sérialisation et frontières réelles, pas seulement mocks de comportement.
- https://testing.googleblog.com/2015/04/just-say-no-to-more-end-to-end-tests.html :
  tests unitaires/integration ciblés et quelques parcours bout en bout.
- https://github.com/aannoo/hcom/blob/main/src/commands/events.rs et
  https://github.com/aannoo/hcom/blob/main/src/core/filters.rs : filtres composables
  et once. Inspiration fonctionnelle seulement, pas reprise de son moteur SQL.

DevKMS indisponible : command -v mem sans résultat. Connaissances conservées ici.
