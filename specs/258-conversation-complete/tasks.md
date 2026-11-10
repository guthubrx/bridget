# Tâches 258 — Conversation complète

Toutes les tâches appartiennent au périmètre autorisé. Les chemins T3 sont sous
/Volumes/8TB2/50-repos-archives/11.Repositories/t3code-subagent-composer ; les
chemins Rust sous le worktree 258 Bridget. Aucun commit avant les gates finales.

## Fondations

- [x] T001 Écrire le cadrage, spec, plan, recherche, contrat, modèle et recette.
- [x] T002 Auditer chaque réutilisation dans reuse-audit.md et lever le gate.
- [x] T003 Exécuter Analyze sur spec/plan/tasks et corriger les écarts non ambigus.
- [x] T004 Implémenter le contrat humain de boîte fermé, la CLI stdin et les capacités ; tests de validation et compatibilité dans crates/bridget-transport et cli_lineage.rs.
- [x] T005 Raccorder lecture et envoi humains aux gardes/réceptions existantes dans daemon/native_lineage.rs ; tester isolation, idempotence, états et ronde.
- [x] T006 Raccorder modèle/effort et interruption au wrapper exact, local et distant ; tester confirmation, refus, perte de réponse et catalogue cible.

## US1 — Conversation commune

- [x] T007 Raccorder journal/état actif à useTimelineFollow et ThreadTimeline ; vérifier suivre, remonter, rester stable et revenir en bas.
- [x] T008 Vérifier regroupement, repli, code/copie, durée connue/inconnue, erreurs et lacunes via les composants existants et leurs tests.

## US2/US3 — Boîte commune

- [x] T009 Étendre les contrats/RPC, BridgetReader, BridgetLineage et client-runtime pour les opérations de boîte ; décodage strict et scopes testés.
- [x] T010 Monter le même ChatComposer et ses stores pour un enfant ; router l'envoi natif, préserver les brouillons et traiter refus/issue inconnue.
- [x] T011 Réutiliser les sélecteurs modèle/effort avec capacités cible, état en attente et sélection confirmée ; aucun changement de fournisseur implicite.
- [x] T012 Raccorder interruption, raccourcis et fonctions de contexte/pièces jointes selon capacités effectives ; vérifier les limites explicites et aucun chemin distant supposé local.

## US4 — Vérification et livraison

- [x] T013 Tester les états occupé/terminé/libéré/déconnecté/révoqué et les gardes ordinaires T3 ; vérifier les consommateurs web/desktop/mobile concernés.
- [x] T014 Exécuter tests Rust ciblés via build.py, tests T3, types/lint/build applicables ; traiter les échecs requis sans masquer les limites héritées.
- [x] T015 Faire la contre-revue du code, Converge et audit V15 ; corriger les manques avant clôture.
- [x] T016 Refaire le circuit UI réel en environnement isolé, enregistrer les clics et produire/vérifier le diaporama dans preuves-corrections.
- [ ] T017 Intégrer et pousser les deux dépôts ; distribuer/activer les changements Bridget par le parcours natif et vérifier actif/retour ; construire/signer/préparer l'installation T3 sans fermer l'application.
- [ ] T018 Nettoyer les ressources propres intégrées et sans usage ; conserver preuves, retour, travaux étrangers ; consigner SHA et exclusions dans validation.md.

- [ ] T019 Corriger le guidage du refus OAuth attesté dans la revue COPIL Thémys et la revue258 ; qualifier terminal+diagnostic exact, conserver api_error et les tâches, tester les citations ordinaires et erreurs historiques, publier dans le même lot.

## Dépendances

T004 → T005/T006 → T009 → T010/T011/T012. T007/T008 indépendantes après Analyze.
T013/T014 suivent les raccords. T015/T016 précèdent T017 ; T018 clôt la livraison.
Une preuve absente conserve la tâche ouverte et le statut In Progress.
