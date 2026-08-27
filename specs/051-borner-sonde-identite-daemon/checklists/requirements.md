# Checklist qualité — Borner la sonde d’identité du daemon

**But** : vérifier que la spécification distingue les observations avant le plan.
**Créée** : 2026-08-27
**Feature** : `specs/051-borner-sonde-identite-daemon/spec.md`

## Qualité du contenu

- [x] Le besoin et sa raison sont séparés de l’implémentation.
- [x] Les sections obligatoires sont remplies.
- [x] Aucun marqueur de clarification ne subsiste.

## Complétude des exigences

- [x] Les trois états sont nommés sans inclusion ni valeur par défaut trompeuse.
- [x] Chaque exigence est testable.
- [x] Les critères de succès sont mesurables.
- [x] Les scénarios positifs et négatifs sont définis.
- [x] Le périmètre et les cas limites sont bornés.
- [x] Les dépendances sont déclarées dans l’en-tête.

## Préparation

- [x] L’oracle rouge observe le vrai chemin réseau.
- [x] Le mutant causal vise la pose effective du délai.
- [x] Le scénario d’incertitude ne peut pas passer par une façade toujours vide.

