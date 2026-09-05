<!-- SPECKIT START -->
Plan actif : specs/089-communication-core/plan.md.
Périmètre : communication inter-agents, observation et fédération SSH.
Ne pas lancer de daemon, wrapper ou script de déploiement avant configuration
explicite d'un home/socket isolé. L'installation historique reste hors périmètre.
Sélection SpecKit : SPECIFY_FEATURE=089-communication-core lorsque le script
exige un préfixe numérique ; la branche Git reste session-089-communication-core.
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
