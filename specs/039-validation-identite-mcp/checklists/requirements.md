# Checklist qualité — Valider l'identité MCP

**But** : valider la complétude de la spécification avant implémentation.

**Feature** : `specs/039-validation-identite-mcp/spec.md`

## Qualité du contenu

- [x] Le problème durable précède le geste technique.
- [x] La comparaison couvre chaque classe de caractères et les longueurs.
- [x] Les scénarios couvrent le refus avant effet et le contrôle sain.
- [x] Toutes les sections obligatoires sont renseignées.

## Complétude des exigences

- [x] Aucun marqueur de clarification ne subsiste.
- [x] Les exigences sont testables et non ambiguës.
- [x] Les critères de succès sont mesurables.
- [x] L'ensemble inverse accepté-canonique/refusé-MCP est établi vide.
- [x] Le destinataire et le domaine sont explicitement bornés hors scope.
- [x] Les dépendances SPEC-006, SPEC-010 et SPEC-026 sont déclarées.

## Préparation de la feature

- [x] L'inventaire daté du parc interdit une régression de disponibilité.
- [x] L'oracle invalide exige zéro exécution, pas seulement un refus.
- [x] Le contrôle positif interdit un détecteur toujours fermé.
- [x] Le mutant vise la grammaire réellement empruntée par `read_name`.
- [x] Aucun changement de CLI, daemon central, destinataire ou domaine n'est
  autorisé.
