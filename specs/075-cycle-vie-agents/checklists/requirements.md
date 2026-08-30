# Checklist qualité - SPEC-075

## Qualité de la spécification

- [x] Aucun détail d'implémentation n'est utilisé comme exigence utilisateur.
- [x] Les trois verbes arrêt, relance et décommissionnement ont des effets
  distincts et testables.
- [x] Les exigences remplacées de SPEC-073 sont identifiées.
- [x] Chaque user story dispose d'un test indépendant.
- [x] Les erreurs, courses, redémarrages et états transitoires sont couverts.
- [x] La conservation de l'historique est explicite.
- [x] Les agents non gérés et TMUX ont un comportement fermé.
- [x] Les critères de succès sont mesurables.
- [x] Le hors périmètre empêche la purge, les actions en masse et le changement
  de runtime de dériver dans cette session.

## Validation des besoins

- [x] L'arrêt reste réversible et visible.
- [x] La relance conserve l'identité logique.
- [x] Le décommissionnement retire réellement la flotte.
- [x] Un échec partiel ne produit pas de faux succès.
- [x] La sémantique survit au redémarrage du daemon.
- [x] La migration des agents arrêtés hérités est traitée sans heuristique de
  nom.
- [x] CLI, HTTP et UI convergent vers une seule autorité daemon.
- [x] Les fonctionnalités connexes pertinentes ont été intégrées: matrice
  d'éligibilité, états transitoires, corrélation, audit et migration.

## Points volontairement non ajoutés

- [x] Suppression définitive de données: nécessite une spécification dédiée.
- [x] Pause ou drain d'un tour: relève du contrôle d'exécution, pas du cycle de
  vie du processus.
- [x] Renommer, cloner ou changer de provider: opérations de configuration
  distinctes.
- [x] Actions de masse: conséquences trop larges pour cette première surface.
