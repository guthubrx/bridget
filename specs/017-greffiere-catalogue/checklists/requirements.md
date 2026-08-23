# Checklist qualité — Spec 017 greffière du catalogue

**But** : vérifier que la spécification est complète avant les artefacts de
conception détaillée.
**Créée** : 2026-08-23
**Feature** : `specs/017-greffiere-catalogue/spec.md`

## Qualité du contenu

- [x] Aucun détail d'implémentation imposé dans les exigences produit.
- [x] Valeur utilisateur et frontière de responsabilité explicites.
- [x] Rédaction accessible à un lecteur non technique.
- [x] Sections obligatoires complètes.

## Complétude des exigences

- [x] Aucun marqueur de clarification ne subsiste.
- [x] Chaque exigence est vérifiable.
- [x] Les critères de succès sont mesurables.
- [x] Les critères sont indépendants d'un choix de framework.
- [x] Les scénarios d'acceptation couvrent le noyau et les refus.
- [x] Les cas limites couvrent migration, récurrence, clôture et homonymie.
- [x] Périmètre et dépendances sont explicites.

## Prêt pour conception

- [x] Le format fermé, la migration, les transitions et la vue sont tracés.
- [x] Les interdits adverses sont tous repris : pas de plan hôte, déduplication,
      score ni état planifié.
- [x] Les trois filets de découvrabilité sont mesurables.
- [x] Toute création de travail est bornée à une délégation durable.
- [x] Le lien constat-remède est un fait de délégation déclaré, jamais une déduction.
- [x] Le journal, son idempotence et son append atomique ont une autorité unique.
- [x] La perte d'événement est rattrapée par l'état durable attesté, sans inférence.
- [x] Les collisions d'identifiant, l'horodatage de tri et les writers concurrents sont testables.

## Notes

La migration est volontairement conservatrice : toute information absente de la
prose doit rester explicitement non classée ou être complétée par un humain ;
elle ne peut pas être inférée par Maicie.
