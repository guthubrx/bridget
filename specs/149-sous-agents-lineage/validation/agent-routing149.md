# Routage des agents - session 149

Date de validation : 2026-10-10. Politique approuvée par l'utilisateur.

| Type de tâche | Agent | Modèle |
|---|---|---|
| Tests existants, lecture de logs, docs | claude | Haiku 5.5 medium |
| Revue ciblée, tests simples | claude | Haiku 5.5 high |
| Cas oubliés, permissions, diagnostics d'interop | claude | Sonnet 5.5 high |
| Développement complexe | Codex | gpt-6.1-sol high |

- Les agents GLM (reviewers et testeurs) sont remplacés depuis le 2026-10-10.
- Les preuves GLM historiques restent en place. Elles ne sont pas réécrites.
- Les modèles GLM 5.3 flash et Codex restent requis pour l'interop produit. Ils ne sont pas des testeurs.
- Pas d'approbation humaine supplémentaire pour les permissions pendant la session 149.
- Pas de reroutage d'agents actifs sans besoin.
- Ce document sert de référence pour les prochaines écritures de docs.
- Hors périmètre : le plan contrat T037, les modèles natifs, les autres rôles.
