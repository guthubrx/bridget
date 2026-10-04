<!-- SPECKIT START -->
Plan actif : specs/133-relais-sous-agents/plan.md.
Périmètre de ce tour : relais MCP borné des sous-agents internes sous l'autorité
de leur parent Bridget. L'utilisateur a autorisé le commit, la fusion, le
déploiement des services installés et le nettoyage le 2026-10-04.
Tous les essais utilisent un BRIDGET_HOME et une socket isolés.
Sélection SpecKit : SPECIFY_FEATURE=133-relais-sous-agents ; branche
session-133-relais-sous-agents.
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
