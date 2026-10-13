# Checklist de spécification 162

**Date :** 2026-10-13. Cette checklist mesure la qualité documentaire, pas le code ni le runtime.

- [x] Le problème utilisateur précède la solution.
- [x] Le périmètre 162 est distinct de 161 et 163.
- [x] Cinq scénarios priorisés ont une vérification indépendante.
- [x] Les exigences FR-001 à FR-030 sont testables.
- [x] Les critères SC-001 à SC-012 sont observables.
- [x] Pannes, permissions et identité sont explicites.
- [x] Les sources et les inconnues opérationnelles sont nommées.
- [x] Aucun choix automatique ou vue globale n’est imposé.
- [x] La compatibilité v1 et la différence entre fichiers et runtime T3 sont explicites.
- [x] Aucun droit ou secret parent n’est élargi ou copié implicitement.
- [x] Aucun succès de test, build ou livraison n’est prétendu.
- [x] Le plan couvre développement, tests, livraison et nettoyage.
- [x] Les routes sont désactivées par défaut et refusent les ambiguïtés.
- [x] Le résultat durable corrige un manque attesté avec store technique, blob immutable et dispositions explicites.
- [x] Les primitives SpecKit absentes et le fallback sont documentés.
- [x] Aucun tasks.md ne précède les gates de réemploi et de contre-revue.

Le choix de projets de production attend la réponse utilisateur. Cela bloque l’activation de la route concernée, pas les recettes isolées. L’audit R1 est PASS ; la contre-revue R1 demande des changements intégrés dans R2. L’actualisation de l’audit et la contre-revue R2 restent à recevoir ; ces cases ne les remplacent pas.

Checkpoint Tasks/Analyze : audit final PASS47/47 et corrections R2 validés par ROOT. 42 tâches non cochées, couverture30FR/12SC/5US ; deux passes Analyze, zéro constat actif après corrections. G8/G9 ouverts, aucune livraison162 prétendue.
