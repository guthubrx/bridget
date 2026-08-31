# État d'implémentation - SPEC-081

Date : 2026-08-31
Branche : session-081-flotte-globale-sources-ui
Worktree : /home/moi/bridget-referent/.worktrees/session-081-flotte-globale-sources-ui
État : implémentée et validée sur Linux, non livrée.

## Réalisé

- Flotte globale agrégée depuis toutes les sources connectées, avec identité composée source_id:agent_name.
- Colonne Sources qui pose des filtres visibles et indépendants pour source et projet.
- Création et import de projet dirigés vers une source explicitement choisie.
- Gestionnaire de serveurs accessible depuis la coque Desktop.
- Tris composables, ordonnables, inversables, regroupements imbriqués et groupes repliables persistés localement.
- Coordinateur pré-épinglé uniquement sur rôle explicite, avec désépinglage local durable.
- Découverte locale non persistée, libellée exactement Cet ordinateur, uniquement lorsqu'un relais local existe réellement.
- Panneau distant compact conservant conversation et onboarding sans cacher la coque Desktop.

## Preuves

- validation ciblée et workspace : evidence/validation.md
- parcours manuel et limite native : evidence/manual-validation.md
- analyse de cohérence : analysis-report.md
- audit de réutilisation : reuse-audit.md
- contre-revue : adversarial-review.md
- audit final : audit.md

## Limite avant livraison

Un contrôle réel sur macOS reste nécessaire pour le rendu WebView et l'interaction de la fenêtre native. Aucune opération de livraison n'a été engagée.
