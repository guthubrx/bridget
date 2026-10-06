<!-- SPECKIT START -->
Plan actif : specs/136-messages-utiles/plan.md.
Périmètre : historique silencieux, actions courtes et remplacement explicite
dans les fils Bridget existants. Session autorisée le 2026-10-06.
Ne jamais déduire un remplacement du texte, ni effacer les messages historiques.
Préserver le travail non fusionné de la session 135.
Tous les essais utilisent un BRIDGET_HOME et une socket isolés.
Sélection SpecKit : SPECIFY_FEATURE=136-messages-utiles ; branche
session-136-messages-utiles.
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
