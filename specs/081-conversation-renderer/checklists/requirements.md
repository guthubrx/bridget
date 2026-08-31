# Checklist de qualité - SPEC-081 Conversation structurée et rendu technique sûr

**But** : vérifier que la spécification est complète avant planification.
**Créée** : 2026-08-31
**Feature** : `specs/081-conversation-renderer/spec.md`

## Qualité du contenu

- [x] Aucun détail d'implémentation ne conditionne la valeur utilisateur.
- [x] Le problème, la valeur et les limites sont exprimés pour un opérateur.
- [x] Toutes les sections obligatoires sont complétées.
- [x] La réutilisation MIT est encadrée sans transformer la spécification en plan technique.

## Complétude des exigences

- [x] Aucun marqueur de clarification ne subsiste.
- [x] Les exigences sont testables et sans ambiguïté fonctionnelle.
- [x] Les critères de succès sont mesurables et vérifiables.
- [x] Les scénarios principaux, les échecs et les dégradations sont définis.
- [x] Le périmètre interdit explicitement l'exécution depuis un contenu de conversation.
- [x] Les dépendances avec les SPEC-069, 074, 076 et 080 sont identifiées.

## Préparation à la planification

- [x] Chaque récit utilisateur apporte une valeur autonome.
- [x] Le comportement sûr par défaut et l'exception demandée par l'opérateur sont distingués.
- [x] La portée locale des préférences est explicite.
- [x] Aucun besoin d'arbitrage utilisateur supplémentaire n'est nécessaire avant le plan.

## Notes

Un document Desktop V1 valide de l'opérateur actuel migre vers les trois
autorisations locales actives. Une nouvelle installation, un reset, une
corruption ou une version inconnue restent à trois autorisations désactivées.
