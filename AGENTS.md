<!-- SPECKIT START -->
Plan actif : specs/102-fils-inter-agents/plan.md.
Branche : session-102-fils-inter-agents ; SPECIFY_FEATURE=102-fils-inter-agents.
Commande du 2026-09-16 : préparation détaillée UNIQUEMENT, sans implémentation,
commit, installation ni relance. Une nouvelle demande est nécessaire pour coder.
Lire specs/102-fils-inter-agents/quickstart.md avant reprise : la session 101
n'est pas intégrée dans cette base 1738a072 ; ne pas copier son code à la main.
Tests futurs : home/socket isolés obligatoires ; aucune conversation fournisseur
réelle ni aucun service de production ne doivent être interrompus.
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
