# Checklist de qualité — SPEC-161

**Date :** 2026-10-13. **Objet :** qualité des artefacts, pas validation du code. **Spec :** specs/161-client-cartae/spec.md.

## Qualité du contenu

- [x] Besoin et valeur utilisateur expliqués avant les moyens.
- [x] Exigences distinctes des détails de script exposés dans le plan.
- [x] Langue française et vocabulaire accessible.
- [x] Sections obligatoires complètes.

## Complétude

- [x] Aucun marqueur de clarification non résolu.
- [x] FR-001 à FR-014 testables et non ambigus.
- [x] SC-001 à SC-006 mesurables.
- [x] Critères décrivent les résultats observables ; les commandes restent au plan/quickstart.
- [x] Scénarios d'acceptation définis pour US1 à US3.
- [x] Cas d'échec et mutations concurrentes pris en compte.
- [x] Périmètre borné au client161 ; besoins162/163 conservés séparément.
- [x] Dépendances095/096/160 et cache offline non attesté nommés.

## Prêt pour planification

- [x] Exigences reliées aux recettes et aux preuves attendues.
- [x] Scénarios couvrent installation, continuité, retour arrière et nettoyage.
- [x] Aucun succès technique ou livraison prétendu.
- [x] Aucun nouveau composant runtime caché dans la spec.

## Validation documentaire

Passe du13octobre : relecture FR/SC/US, corrections avant publication. Les versions/champs de provenance nomment la cible produit, pas une prescription d'architecture. Réemploi PASS, contre-revue du plan traitée, 14 tâches générées et Analyze avec seconde passe effectués. Implémentation et livraison restent à ROOT. Aucune tâche d'implémentation cochée.
