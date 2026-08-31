# Checklist de qualité - SPEC-076 Espaces projets et coordinateur initial

**But**: vérifier que la spécification est complète, testable et ne devance pas les autorités de SPEC-065 à SPEC-075.
**Créée**: 2026-08-30
**Feature**: [spec.md](../spec.md)

## Qualité du contenu

- [x] La valeur utilisateur et le problème sont exprimés sans imposer une architecture.
- [x] Le projet visible est défini comme dossier réel, sans second registre visible.
- [x] Les parcours créer, importer, reconnecter, retirer et réactiver sont séparés.
- [x] La configuration distingue outil agent, fournisseur ou upstream, modèle, effort et permissions.
- [x] L'analyse initiale est bornée et son absence de mutation est explicite.

## Complétude des exigences

- [x] Aucun marqueur de clarification ne reste.
- [x] Les exigences fonctionnelles sont observables et testables.
- [x] Les critères de succès sont mesurables et indépendants de la technologie.
- [x] Les cas limites de doublon, chemin déplacé, lien symbolique, dépôt modifié, configuration absente et incompatibilité sont couverts.
- [x] Les dépendances 065, 066, 067, 072, 074 et 075 sont déclarées.
- [x] Les limites de SPEC-066 et SPEC-067 sont explicitement préservées.

## Préparation de la planification

- [x] Les autorités à réutiliser devront être vérifiées dans un reuse-audit avant les tâches.
- [x] L'implémentation est explicitement bloquée tant que SPEC-066 et SPEC-067 ne sont pas livrées et validées.
- [x] Aucune approbation de secret ou de profil n'est attribuée à l'interface.
- [x] Aucun agent complémentaire n'est créé automatiquement.
- [x] Le relais UI reste lié à loopback et Bridget Desktop passe par le tunnel SSH de SPEC-074.
- [x] La réactivation est une mutation durable à implémenter, pas une création de projet déguisée.

## Notes

La SPEC est prête pour le plan et l'audit de l'existant. La planification doit
vérifier les surfaces UI réellement présentes, le contrat de racines déjà livré
par SPEC-065, le registre de fournisseurs SPEC-072 et l'impact exact des
worktrees SPEC-074 et SPEC-075.
