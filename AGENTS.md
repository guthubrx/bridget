<!-- SPECKIT START -->
Plan actif : specs/099-fiabilite-communications/plan.md (socle : session 089).
Périmètre : communication inter-agents, observation et fédération SSH.
Ne pas lancer de daemon, wrapper ou script de déploiement avant configuration
explicite d'un home/socket isolé. L'installation historique reste hors périmètre.
Sélection SpecKit : SPECIFY_FEATURE=099-fiabilite-communications lorsque le script
exige un préfixe numérique ; branche session-099-fiabilite-communications.
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
