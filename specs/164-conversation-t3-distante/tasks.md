# Tasks SPEC-164 — Conversations adaptateur de bureau des tâches Bridget distantes

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-1).

Les tâches ouvertes restent non cochées jusqu’à l’acceptation de leur preuve par ROOT. Une preuve historique ou une approbation de plan ne prouve aucun code. Aucun test, install ou build ne démarre avant le permis ROOT applicable. Le développement reprend dans une ronde native durable après les gates, pas par message ordinaire hors mission.


- [x] T001 — Figer le plan et ouvrir le GO (ROOT, DOC164)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-2).


- [x] T002 — Préparer le checkout et les contrôles adaptés (ROOT, dépend de T001)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-3).


- [x] T003 — Projeter le texte exact (DEV, dépend de T001/T002)

Couverture : US1 ; FR-003/004/016 ; SC-001/005.
Sources/zone autorisées : `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/packages/client-runtime/src/bridgetJournalPresentation.ts`, `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/packages/client-runtime/src/bridgetJournalPresentation.test.ts`.
- Ajouter le helper client-runtime commun de segments contigus, identités de livraison/session, clés stables et contenu exact sans séparateur inventé.
- Couvrir S164-01/02 avec payloads update/text/content réels synthétisés ; aucun rôle/message fournisseur fabriqué.


- [x] T004 — Fusionner sans perdre les conflits ni les coupes (DEV, dépend de T003)

Couverture : US2/US3 ; FR-003/011/012 ; SC-001/005.
Sources/zone autorisées : `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/packages/client-runtime/src/bridgetJournalPresentation.ts`, `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/web/src/components/BridgetTaskJournal.tsx`, `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/mobile/src/features/threads/BridgetTaskJournal.tsx`.
- Ajouter mergeJournalEvents pur commun avant Map des deux conteneurs, garder le premier événement et un témoin de conflit borné par séquence chargée ; rejeu identique sans doublon.
- Lever l’indication de tête coupée après une fusion dépassant 4 096 événements, conserver les références inchangées et couvrir S164-04/10.


- [x] T005 — Conserver toutes les frontières de lacune pertinentes (DEV, dépend de T003)

Couverture : US1/US2/US3 ; FR-005/012/016 ; SC-002/005.
Sources/zone autorisées : `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/packages/client-runtime/src/state/orchestration.ts`, `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/packages/client-runtime/src/state/orchestration.bridget149.test.ts`.
- Compléter le réducteur client existant avec gaps éphémères bornés64, gap legacy conservé et marqueurs sticky dédupliqués.
- Pruner seulement hors fenêtre ; sous pression préserver une coupe conservative qui interdit de fusionner à travers une frontière oubliée. Couvrir deux gaps et dépassement64 dans l’oracle runtime149, S164-03.


- [x] T006 — Présenter les activités et avis prouvés (DEV, dépend de T003)

Couverture : US1 ; FR-007/009 ; SC-002.
Sources/zone autorisées : `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/packages/client-runtime/src/bridgetJournalPresentation.ts`, `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/packages/client-runtime/src/bridgetJournalPresentation.test.ts`.
- Appliquer le tableau de formes figé : command started sans item_id, completed explicite, outil Claude appel seul, ACP tool_call_id, file et refusal attribué Bridget.
- Présenter entrées/sorties repliables et état neutre sans preuve ; aucune corrélation de proximité. Couvrir S164-05.


- [x] T007 — Garder les détails originaux consultables (DEV, dépend de T004/T005/T006)

Couverture : US3 ; FR-008/016 ; SC-002/005.
Sources/zone autorisées : `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/web/src/components/BridgetTaskJournal.tsx`, `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/mobile/src/features/threads/BridgetTaskJournal.tsx`.
- Ajouter le rendu technique sûr des formes inconnues, héritées, anomalies et sources de chaque bloc avec séquences.
- Monter le brut seulement à l’ouverture ; aucune copie profonde ou collection illimitée, aucun HTML exécuté. Couvrir S164-06.


- [x] T008 — Rendre le Markdown web inerte (DEV, dépend de T001/T002)

Couverture : US1/US4 ; FR-006/015 ; SC-004.
Sources/zone autorisées : `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/web/src/components/ChatMarkdown.tsx`, `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/web/src/components/ChatMarkdown.permissions.test.tsx`.
- Ajouter une prop opt-in inerte à ChatMarkdown, couper fichier/code-inline actif, médias, aperçus, shell et cartes actives ; conserver les usages ordinaires.
- Couvrir `/etc/passwd`, image locale, image HTTPS, HTML avec onerror et copie exacte, sans action ni chargement implicite. S164-07.


- [x] T009 — Intégrer la conversation web/desktop (DEV, dépend de T003–T008)

Couverture : US1/US2/US3/US4 ; FR-002/006/009/010/013/014/015 ; SC-002/003/004.
Sources/zone autorisées : `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/web/src/components/BridgetTaskJournal.tsx`, `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/web/src/components/BridgetTaskJournal149.test.tsx`.
- Remplacer les lignes JSON du Journal par blocs lisibles, Markdown inerte, activité secondaire, détails repliables et résultat distinct.
- Préserver actions, état/contact, permissions, sélection/copie, défilement volontaire et identité des blocs clos mémoïsés. Étendre les tests Journal149, S164-08/09/11.


- [x] T010 — Intégrer la conversation mobile (DEV, dépend de T003–T009)

Couverture : US1/US2/US3/US4 ; FR-002/006/009/010/013/014/015 ; SC-002/003/004.
Sources/zone autorisées : `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/mobile/src/features/threads/BridgetTaskJournal.tsx`, `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx`.
- Réemployer primitives Markdown natives, règles et styles dans un contexte inerte. Ne pas exporter AssistantMarkdownContent complet avec hooks thread/cartes actives et ne pas réduire au texte brut.
- Utiliser la même projection pour outils/détails/résultat ; conserver focus/foreground/abort/actions et lecture seule. Étendre le test mobile149, S164-07/08/09/11.


- [x] T011 — Valider pages, ressources et résultat durable (DEV, dépend de T009/T010)

Couverture : US2/US3 ; FR-005/010/011/012/013 ; SC-002/003/005.
Sources/zone autorisées : `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/web/src/components/BridgetTaskJournal.tsx`, `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/mobile/src/features/threads/BridgetTaskJournal.tsx`.
- Projeter chaque page historique seule avec début extrait sauf début attesté ; garder page suivante/retour suivi et curseurs existants.
- Préserver fenêtre de 4 096 événements/pages de 100 événements/résultat par fenêtres de 16 KiB et plafond de 256 KiB, résultat incomplet indiqué, copie du chargé exacte et zéro RPC par événement. Couvrir S164-03/08/10.


- [x] T012 — Exécuter les contrôles ciblés finaux (DEV, ROOT, dépend de T011)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-4).


- [x] T013 — Self-review et Converge (DEV, ROOT, dépend de T012)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-5).


- [x] T014 — Contre-revue code et audit ciblé (ROOT, dépend de T013)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-6).


[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-7).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-8).


- [x] T016 — Préparer le paquet signé (ROOT, dépend de T015)

Couverture : US4 ; FR-017 ; SC-006.
Sources/zone autorisées : `/Users/moi/.cache/bridget-delivery164-20261013`.
- Construire dans un stage Mac natif avec caches autorisés ; une seule compilation de flotte. Si164 a terminé ses gates source T013/T014/T015 à temps, paquet commun163+164 avec preuves et gates produit distinctes. Ne pas embarquer de code164 non validé. Sinon le paquet163 seul reste possible ; aucun cycle avec G8/G9 de Bridget163.
- Vérifier signatures, sources et validations de paquet existantes. Garder le rollback valide ; aucun remplacement installé tant que les gates requises ne sont pas satisfaites.


- [x] T017 — Vérifier la présentation intégrée (ROOT, dépend de T015/T016)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-9).


- [x] T018 — Installer et fermer G8 (ROOT, dépend de T016/T017)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-10).


- [x] T019 — Nettoyer164 et fermer G9 (ROOT, dépend de T018)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-11).


- [x] T020 — Clore les métadonnées et le reçu (ROOT, dépend de T018/T019)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-12).

## Convergence

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-13).

- [x] T021 — Rendre l’enveloppe originale et le témoin conflictuel complets (DEV, ROOT ; dépend de T012, corrective de T007 ; relecture T013 après preuve)

Couverture : US3 ; FR-008/011 ; SC-002.
Sources/zone autorisées :
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/web/src/components/BridgetTaskJournal.tsx
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/web/src/components/BridgetTaskJournal149.test.tsx
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/mobile/src/features/threads/BridgetTaskJournal.tsx
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx

- Réemployer les détails existants : JSON.stringify de chaque événement original complet et du témoin concurrent complet, version/date/session/séquence/type/livraison/payload inclus. Garder le premier événement, les références et les bornes ; aucun contenu actif. Monter le JSON seulement à l’ouverture, sans helper nouveau.
- Étendre les deux bancs Journal149 existants : avant ouverture aucun brut monté ; après ouverture v/ts consultables ; rejeu avec même seq et payload mais ts différent rend les deux enveloppes et conserve le premier. Aucun succès implicite ou donnée supprimée.
- ROOT exécute seulement les deux bancs affectés et les contrôles concernés autorisés, recueille hashes/transfert stable, puis relance Converge. Pas de replay des36 tests ou du Markdown inchangé. Contre-revue et audit T014 après convergence.

**Checkpoint de progression après ajout : In Progress, 2/21.** Le 2/20 en tête reste le checkpoint initial ; ROOT synchronise la progression après acceptation des preuves. Aucun cochage ajouté.

## Correction de contre-revue code R1

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-14).

- [x] T022 — Corriger classification inconnue, témoin rejoué et extrait de préfixe certain (DEV, ROOT ; dépend de T012 et revue R1, corrective de T005/T006/T007 ; nouvelle lecture T013/T014 après preuves)

Couverture : US1/US2/US3/US4 ; FR-005/008/011/012 ; SC-001/002/005.
Sources autorisées :
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/packages/client-runtime/src/bridgetJournalPresentation.ts
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/packages/client-runtime/src/bridgetJournalPresentation.test.ts
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/web/src/components/BridgetTaskJournal.tsx
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/web/src/components/BridgetTaskJournal149.test.tsx
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/mobile/src/features/threads/BridgetTaskJournal.tsx
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx

- Garder les kind hérités inconnus (__proto__/constructor/toString) dans le fallback technique sûr, par hasOwnProperty.call avant lookup ; aucun objet/fonction comme label React.
- Ne pas augmenter le compteur pour le même témoin rejoué ; présenter les autres réceptions comme occurrences, sans promettre un nombre de versions uniques ni créer un historique supplémentaire.
- Garder une indication Extrait pour une lacune explicite entièrement avant le premier événement retenu, via headCut certain dans les deux conteneurs. Aucun gap inventé depuis un saut seq.
- Réemployer les trois oracles/bancs existants des six fichiers. Producteur code-only sans compilation/test/lint ; ROOT tient permis, groupes affectés et transfert/provenance. Aucun correctif confort pour les warnings React admis.
- Attendre source R6 stable et résultats affectés avant nouvelle lecture Converge/audit delta ; P1/P2 et auditR5b historiques préservés, aucun P3 ni résultat futur présumé.

**Progression courante : In Progress,2/22.** Les21cases préexistantes conservent leur état ; T022 ouverte.

## Acceptation ROOT — source R6 validée, 2026-10-14

ROOT a accepté les preuves P3 CONVERGED, revue indépendante R2 APPROVE, audit incrémental et validateur canonique RC0. Sur cette instruction explicite, T003–T014 et T021/T022 sont cochées. T001/T002 étaient déjà acceptées. **Progression actuelle : In Progress,16/22.**

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-15).

Cette acceptation ne relance ni ne réatteste les tests. T015–T020 restent non cochées : intégration Git, paquet, recette UI, installation, nettoyage et reçu final. Aucun succès de livraison ou runtime n'est déduit.

## Acceptation ROOT — T015/T016/T017, 2026-10-14

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-16).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-17).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-18).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-19).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-20).

## Acceptation ROOT — G8/G9 attestés, 2026-10-14

**Progression actuelle : In Progress,21/22.** ROOT accepte G8 et G9 sur les reçus réels. T018/T019 sont cochées. T020 reste ouverte jusqu’au commit/push documentaire final par ROOT. Aucun push futur n’est attesté.

G8 opérationnel réel accepté sur source Bridget `3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5` : `/Users/moi/.cache/bridget-delivery163-20261013/g8-t058-operational-final.json`, SHA-256 `5b1c00923f2f307136b5831cc8d3328227b7284deacf44c793d71542e3ceb52c`. Résultat durable `RECETTE163_G8_OK`, un reçu ; OFF explicite prouvé puis replay ON de la même clé inchangé. Le replay ne constitue ni nouvelle tâche ni nouvel appel de décision. La recette HTTP de décision reste celle du laboratoire T040R4. Le provider réel prouvé par cette recette est codex-pro ; autres profils configurés sans nouvelle attestation quota/auth. ROOT atteste daemon31298/pont31420 et workerCartae1454489 actifs, trois routes autorisées Bridget/Roadie/Opus2D. Aucune inspection des capacités en mémoire ni mission de modèle distant supplémentaire déduite.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-21).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-22).

Conservations Cartae justifiées : `/Users/moi/.cache/bridget-delivery163-20261013/g9-cartae-final-usage-retentions.json`, SHA-256 `cb23362780df36d938f321c054c0fe626f0021194d65dbb78ed30f2e8852d359`. lsof RC0 mais incomplet à cause de FUSEportal `/run/user/1002/doc` ; aucun contournement. Sont conservés `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission`, sa copie `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164`, `/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target` et les cinq namespaces `/home/moi/.local/state/bridget-delivery164/tests/tmp-r2`, `/home/moi/.local/state/bridget-delivery164/tests/tmp-r3`, `/home/moi/.local/state/bridget-delivery164/tests/tmp-r4`, `/home/moi/.local/state/bridget-delivery164/tests/tmp-r5b`, `/home/moi/.local/state/bridget-delivery164/tests/tmp-r6`. Pas de retrait distant ni de gain physique revendiqué. Rollback, preuves, DB et travaux étrangers restent protégés.

S164-12 est une recette web réelle du paquet, avec données synthétiques : sept captures et archive66membres SHA vérifiés, archive `/Users/moi/.cache/bridget-delivery164-20261013/ui-recipe/ui-recipe164-evidence.tar.gz` SHA-256 `ef4fa8ed527f20ccbc1ba89d467bf02096b67112eae258c1ccd1173786c6cd99`. La taille390×844 prouve le web responsive, pas React Native natif ni Electron actif.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-23).

### Conservations G9 supplémentaires

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-24).

## Clôture documentaire acceptée par ROOT — 2026-10-14

**Statut canonique : Implemented. Progression :22/22. Pipeline : Livré, G8/G9 acceptés avec conservations justifiées.** T020 est cochée sur le push documentaire réel `c57cfd35efa1338cda5256a02c57d71258d8b8d4` vers github/main. Reçu : `/Users/moi/.cache/bridget-delivery163-20261013/git-final-doc-checkpoint-c57cfd35.json`, SHA-256 `56316fdcb4202865fbb57e7979a11d302bb968e2170453a9523070a490a30390` ; HEAD et référence distante égaux, racine propre à cette observation. Ce commit contient les documents de livraison G8/G9 avant le cochage final. Les cases fermées sont intégrées et poussées dans `af8998a7d7494913d4420e2dca7354a73bd8a4f5`, vérifié par ROOT sur github/main égal à HEAD et racine propre. Cartae main et worktree sont alignés sur ce commit ; archive25membres vérifiée byte-identique. Reçu : `/Users/moi/.cache/bridget-delivery163-20261013/git-final-cartae-af8998a7-command.json`. Les checkpoints21/22 précédents restent historiques.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/tasks.md#extrait-25).

La recette S164-12 prouve le client web construit avec données synthétiques, en taille desktop et mobileCSS ; elle ne vaut pas Electron actif ni React Native natif. Les tests et les audits ciblés restent qualifiés par leur provenance et couverture partielle. Aucun nouveau Converge, audit, test, build ou succès global GREEN n'est créé par cette clôture. Bridget est autonome.
