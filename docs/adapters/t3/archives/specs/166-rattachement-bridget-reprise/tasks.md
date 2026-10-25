# Archives techniques de l’adaptateur T3

Document d’origine : [specs/166-rattachement-bridget-reprise/tasks.md](../../../../../../specs/166-rattachement-bridget-reprise/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [x] T003 Ajouter l’oracle RED qui monte A, tourne vers B et exige `thread/resume` avec B avant `turn/start` dans `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts`
- [x] T004 Conserver la sortie du test RED et prouver que le code courant saute la reprise dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/specs/166-rattachement-bridget-reprise/validation/red.md`

<a id="extrait-2"></a>

## Extrait 2

- [x] T005 [US1] Centraliser la requête native de reprise sans changer le traitement des fils archivés dans `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts`
- [x] T006 [US1] Comparer la preuve montée à la preuve courante et reprendre le même fil natif avant le tour uniquement si elles diffèrent dans `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts`
- [x] T007 [US1] Faire passer l’oracle de rotation et prouver qu’une preuve stable n’ajoute aucune reprise dans `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts`

<a id="extrait-3"></a>

## Extrait 3

- [x] T008 [US2] Étendre le test de rotation pour vérifier la publication des droits sur B et le refus d’A dans `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts`
- [x] T009 [US2] Rejouer les scénarios de session partagée, reprise et bifurcation sans traversée d’identité dans `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts`

<a id="extrait-4"></a>

## Extrait 4

- [x] T010 [US3] Vérifier dans le rejeu le même identifiant natif, `excludeTurns: true`, un seul `turn/start` et l’échec sûr si la reprise échoue dans `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts`

<a id="extrait-5"></a>

## Extrait 5

- [x] T011 Documenter la décision et vérifier que le diff ne crée ni service, ni stockage, ni dépendance dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/docs/decisions/166-rattachement-bridget-reprise.md`
- [x] T012 Exécuter format, tests ciblés de l’adaptateur, du gestionnaire de sessions et de l’attestation, typecheck et contrôles proportionnés ; consigner les commandes et codes retour dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/specs/166-rattachement-bridget-reprise/validation/tests.md`
- [x] T013 Intégrer et pousser les sources validées, construire et signer T3, préparer le rollback puis installer l’application sans effacer de donnée ; consigner les versions dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/specs/166-rattachement-bridget-reprise/validation/delivery.md`
- [x] T014 Relancer T3 après contrôle des processus, valider un fil repris sans réinstallation Bridget, finaliser le reçu et nettoyer sans force dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/166-rattachement-bridget-reprise/validation/livraison166.md`
