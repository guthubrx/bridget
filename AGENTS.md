<!-- SPECKIT START -->
Plan actif : specs/135-controle-missions-silencieux/plan.md.
Périmètre de ce tour : rendre agent-loop silencieux sans changement utile et
contraindre le suivi des missions par prise en charge, progrès et escalade.
Tous les essais utilisent des runs agent-loop isolés. La boucle Politique ne
sera migrée qu'après validation complète.
Sélection SpecKit : SPECIFY_FEATURE=135-controle-missions-silencieux ; branche
session-135-controle-missions-silencieux.
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
