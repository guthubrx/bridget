<!-- SPECKIT START -->
Plan actif : specs/134-noms-humains-messages/plan.md.
Périmètre de ce tour : afficher le nom humain à côté de l'UUID dans les messages
Bridget et réparer les profils absents au prochain enregistrement. L'utilisateur
a autorisé le commit, la fusion, le déploiement, le push et le nettoyage le
2026-10-05.
Tous les essais utilisent un BRIDGET_HOME et une socket isolés.
Sélection SpecKit : SPECIFY_FEATURE=134-noms-humains-messages ; branche
session-134-noms-humains-messages.
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
