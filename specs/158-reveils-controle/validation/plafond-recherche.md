# Recherche du plafond158 — mesure avant raccord MCP

4096 lignes privées, corps8192 caractères,500 appels par cas. Index native_delegations_child déjà présent. Aucun index/cache ajouté. O(log n+B) pour la recherche et le décodage ; ceci ne mesure pas le coût d'ouverture Rust ni un appel MCP de bout en bout.

{
  "child-4095": {
    "count": 500,
    "p50_ms": 0.030625,
    "p95_ms": 0.037042,
    "max_ms": 0.637584
  },
  "parent-absent": {
    "count": 500,
    "p50_ms": 0.008625,
    "p95_ms": 0.009542,
    "max_ms": 0.017
  }
}

Le plan SQLite utilise l'index pour présence et absence. Preuve : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/specs/158-reveils-controle/validation/plafond-recherche.json.
