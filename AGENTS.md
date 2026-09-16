<!-- SPECKIT START -->
Plan actif : specs/100-observation-partage/plan.md (socle : sessions 089 et 099).
Périmètre : communication inter-agents, observation et fédération SSH.
Ne pas lancer de daemon, wrapper ou script de déploiement avant configuration
explicite d'un home/socket isolé pour les tests. L'adoption de la session 100
sur l'installation existante est autorisée par l'utilisateur le 2026-09-16,
après validation, sauvegarde et vérification des processus concernés.
Sélection SpecKit : SPECIFY_FEATURE=100-observation-partage lorsque le script
exige un préfixe numérique ; branche session-100-observation-partage.
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
