# Tâches — SPEC-166 Rattachement Bridget après reprise T3

**Statut :** In Progress  
**Progression :** 13/14  
**Pipeline :** Application installée ; activation et recette réelle en attente.

Chaque tâche reste ouverte jusqu’à sa preuve. Le test de rotation précède le correctif.

## Phase 1 — Préparation

- [x] T001 Figer le diagnostic expurgé et les identifiants de version dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/specs/166-rattachement-bridget-reprise/validation/runtime-before.md`
- [x] T002 Vérifier l’absence de duplication et les règles applicables dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/specs/166-rattachement-bridget-reprise/reuse-audit.md`

## Phase 2 — Fondations

- [x] T003 Ajouter l’oracle RED qui monte A, tourne vers B et exige `thread/resume` avec B avant `turn/start` dans `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts`
- [x] T004 Conserver la sortie du test RED et prouver que le code courant saute la reprise dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/specs/166-rattachement-bridget-reprise/validation/red.md`

## Phase 3 — US1 Reprendre un fil sans réparer Bridget

**But :** le premier tour d’un fil ancien recharge sa preuve privée courante.  
**Test indépendant :** A est montée, B devient courante, puis la transcription observe la reprise avec B avant le tour.

- [x] T005 [US1] Centraliser la requête native de reprise sans changer le traitement des fils archivés dans `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts`
- [x] T006 [US1] Comparer la preuve montée à la preuve courante et reprendre le même fil natif avant le tour uniquement si elles diffèrent dans `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts`
- [x] T007 [US1] Faire passer l’oracle de rotation et prouver qu’une preuve stable n’ajoute aucune reprise dans `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts`

## Phase 4 — US2 Préserver les refus de sécurité

**But :** la réparation ne prête jamais l’identité d’un autre fil.  
**Test indépendant :** B reçoit les droits du run ; A et la preuve d’un fil voisin ne les reçoivent pas.

- [x] T008 [US2] Étendre le test de rotation pour vérifier la publication des droits sur B et le refus d’A dans `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts`
- [x] T009 [US2] Rejouer les scénarios de session partagée, reprise et bifurcation sans traversée d’identité dans `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts`

## Phase 5 — US3 Conserver la continuité des conversations

**But :** la réparation garde le même fil natif et son historique.  
**Test indépendant :** la reprise utilise le même identifiant avec `excludeTurns: true`, puis le tour démarre une fois.

- [x] T010 [US3] Vérifier dans le rejeu le même identifiant natif, `excludeTurns: true`, un seul `turn/start` et l’échec sûr si la reprise échoue dans `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts`

## Phase 6 — Qualité et livraison

- [x] T011 Documenter la décision et vérifier que le diff ne crée ni service, ni stockage, ni dépendance dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/docs/decisions/166-rattachement-bridget-reprise.md`
- [x] T012 Exécuter format, tests ciblés de l’adaptateur, du gestionnaire de sessions et de l’attestation, typecheck et contrôles proportionnés ; consigner les commandes et codes retour dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/specs/166-rattachement-bridget-reprise/validation/tests.md`
- [x] T013 Intégrer et pousser les sources validées, construire et signer T3, préparer le rollback puis installer l’application sans effacer de donnée ; consigner les versions dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/specs/166-rattachement-bridget-reprise/validation/delivery.md`
- [ ] T014 Relancer T3 après contrôle des processus, valider un fil repris sans réinstallation Bridget, finaliser le reçu et nettoyer sans force dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/166-rattachement-bridget-reprise/validation/livraison166.md`

## Dépendances

`T001 → T002 → T003 → T004 → T005 → T006 → T007 → T008 → T009 → T010 → T011 → T012 → T013 → T014`

## Possibilités de travail parallèle

Le changement de code et son rejeu utilisent les mêmes fichiers. Ils restent séquentiels pour éviter une course. La documentation T011 peut être relue pendant T012, mais sa validation finale dépend du diff complet.

## Stratégie d’implémentation

Le minimum livrable est US1 avec l’oracle RED puis GREEN. US2 et US3 verrouillent ensuite la sécurité et la continuité avant toute installation. T005 centralise un comportement déjà présent. Cette tâche réduit la maintenance future. Aucune tâche n’ajoute une couche ou une dépendance.
