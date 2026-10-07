<!-- SPECKIT START -->
Plan actif : specs/140-bulles-compactes/plan.md.
Périmètre : bulles Bridget compactes dans T3 et titre réel des fils à la remise.
Session140 autorisée le 2026-10-07 par « my-specify-all go ».
Extension du titre du fil autorisée explicitement le 2026-10-07.
Commit, fusion, push et installation autorisés explicitement par l'utilisateur
le 2026-10-07. Livraison avec sauvegarde et vérification après installation.
Préserver les textes, conversations, agents, relances et verdicts des missions.
Essais frontend et transports isolés ; préserver les données actives pendant
la livraison. Aucun cleanup d'autres travaux sans autorisation.
Sélection SpecKit : SPECIFY_FEATURE=140-bulles-compactes ; branche
session-140-bulles-compactes.
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
