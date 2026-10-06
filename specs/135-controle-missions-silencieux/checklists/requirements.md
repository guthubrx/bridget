# Checklist qualité — Contrôle silencieux des missions

**But** : vérifier que la spécification est prête pour la planification.
**Créé le** : 2026-10-05
**Feature** : `specs/135-controle-missions-silencieux/spec.md`

## Qualité et complétude

- [x] Le problème utilisateur est décrit sans dépendre d'une technologie.
- [x] Les scénarios principaux ont un test indépendant.
- [x] Les délais et résultats sont mesurables.
- [x] Aucun marqueur de clarification ne reste.
- [x] Les cas d'inaction, d'escalade et de clôture sont couverts.
- [x] Le périmètre et les limites sont explicites.
- [x] La compatibilité des runs existants est définie.
- [x] La confidentialité et l'intégrité des verdicts sont protégées.

## Notes

Validation réussie au premier passage. Le rôle d'escalade est configurable. Son
absence produit une demande de décision au lieu d'un envoi ambigu.
