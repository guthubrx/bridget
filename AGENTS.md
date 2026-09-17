<!-- SPECKIT START -->
Plan actif : specs/101-abonnements-t3/plan.md (socle : sessions 098, 099 et 100).
Périmètre : communication inter-agents, observation et fédération SSH.
Ne pas lancer de daemon, wrapper ou script de déploiement avant configuration
explicite d'un home/socket isolé pour les tests. Les adoptions100 et101
sur l'installation existante sont autorisées par l'utilisateur le2026-09-16,
après tests, sauvegarde et vérification des processus. Pour101 : relancer
seulement Bridget et son pont, jamais T3 ni ses fournisseurs ; aucun commit.
Sélection SpecKit : SPECIFY_FEATURE=101-abonnements-t3 lorsque le script
exige un préfixe numérique ; branche session-101-abonnements-t3.
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
