<!-- SPECKIT START -->
Plan actif : specs/139-entetes-complets/plan.md.
Périmètre : tous les en-têtes Bridget discrets dans T3.
Session139 autorisée le 2026-10-07 par « go ».
Commit, fusion, déploiement et push autorisés explicitement le 2026-10-07.
Préserver les textes, conversations, agents, relances et verdicts des missions.
Essais frontend isolés ; livraison avec sauvegarde et retour arrière du paquet.
Sélection SpecKit : SPECIFY_FEATURE=139-entetes-complets ; branche
session-139-entetes-complets.
<!-- SPECKIT END -->

<!-- SPECKIT-USER START -->
# Couche utilisateur SpecKit

- Source de verite utilisateur: `~/.speckit/`.
- Avant une feature/refactor structurel: executer ou appliquer `/speckit.sync`.
- Charger `.specify/memory/constitution.md` pour les gates projet.
- Charger `.specify/memory/standards.md` pour retrouver les refs et baselines research.
- Ne pas modifier directement `.agents/skills/speckit-*`, `.specify/templates/*`, `.specify/scripts/*` ou `.specify/workflows/*`.
- Les commandes utilisateur canoniques vivent dans `~/.speckit/commands/` et sont publiees vers Codex/Claude/Gemini par `~/.speckit/scripts/sync-agent-adapters.py`.
<!-- SPECKIT-USER END -->
