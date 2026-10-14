# Tasks SPEC-164 — Conversations T3 des tâches Bridget distantes

**Statut :** In Progress
**Progression :** 16/22
**Propriétaires :** DOC164 documents ; DEV_CARTAE164 code T3 isolé ; ROOT gates, Git, recette, paquet, installation et nettoyage.

Les tâches ouvertes restent non cochées jusqu’à l’acceptation de leur preuve par ROOT. Une preuve historique ou une approbation de plan ne prouve aucun code. Aucun test, install ou build ne démarre avant le permis ROOT applicable. Le développement reprend dans une ronde native durable après les gates, pas par message ordinaire hors mission.


- [x] T001 — Figer le plan et ouvrir le GO (ROOT, DOC164)

Couverture : US1–US4 ; FR-001/017 ; SC-006.
Sources/zone autorisées : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/spec.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/plan.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/reuse-audit.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/tasks.md`.
- Accepter les corrections de contre-revue R1 dans spec/plan/audit/contrat ; artefacts cohérents, analyse en deux lectures et checklist avant code.
- Fournir au développeur le contrat figé, la propriété des fichiers et les preuves de formes source ; autoriser la ronde code après clôture des objections.


- [x] T002 — Préparer le checkout et les contrôles adaptés (ROOT, dépend de T001)

Couverture : US4 ; FR-001/016/017 ; SC-005/006.
Sources/zone autorisées : `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/`.
- Attester la copie T3 Cartae dédiée164 et sa base 174a, les droits courants et l’absence de source Rust 164 ; conserver les fichiers étrangers.
- Choisir les contrôles Linux avec une toolchain privée Node 24.13.1/pnpm 11.10.0 nécessaire, ou les contrôles ciblés Mac autorisés. Aucun binaire Mac arm64 sur Linux, aucun install avant permis, aucune compilation simultanée dans la flotte.


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

Couverture : US1–US4 ; FR-001–017 ; SC-001–006.
Sources/zone autorisées : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/tests164.md`, `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/`.
- Exécuter les groupes projection+réducteur, web+Markdown inerte et mobile, format/lint/TypeScript concernés, diffcheck. Produire commandes, codes retour, hashes et limites.
- Rejouer seulement les tests affectés après correction. Ne pas exécuter un repo-wide ou une suite Rust164 ; les résultats 163 ne valent pas164.


- [x] T013 — Self-review et Converge (DEV, ROOT, dépend de T012)

Couverture : US1–US4 ; FR-001–017 ; SC-001–006.
Sources/zone autorisées : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/requirements-matrix.md`, `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/`.
- Relire tout le diff, supprimer les lignes sans effet utile, vérifier O(n+c), références des blocs clos, détails montés à l’ouverture, pureté, lecture seule et absence de couplage.
- Mapper17FR/6SC/4US sur les preuves réelles et conduire les lectures Converge requises sur source figée. Aucun cochage par simple affirmation du développeur.


- [x] T014 — Contre-revue code et audit ciblé (ROOT, dépend de T013)

Couverture : US1–US4 ; FR-001–017 ; SC-001–006.
Sources/zone autorisées : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/`, `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/`.
- Obtenir contre-revue indépendante et audit du périmètre réel : fidélité/rejeu/frontières, sécurité Markdown, limites, mobile, minimalisme et complexité.
- Fermer chaque objection sur preuve fraîche ; conserver limites de couverture. Une correction retourne à son test affecté et au contrôle nécessaire, pas à une suite globale inchangée.


- [ ] T015 — Intégrer après 163 et figer la source de livraison (ROOT, dépend de T014 et source T3 de163 validée/intégrée/poussée)

Couverture : US4 ; FR-017 ; SC-006.
Sources/zone autorisées : `/Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante`.
- Rapatrier les seules sources164 validées par T013/T014 et leurs preuves avec hashes ; intégrer la source T3 de163 validée/intégrée/poussée avant164, rapprocher les diff et préserver tous les travaux étrangers. T015 n’attend pas les gates produit G8/G9 de Bridget163.
- Commit source validée, merge et push vers le fork T3 ; vérifier références locales/distantes, tree et propreté. Aucun upstream, PR non demandée ou signature IA.


- [ ] T016 — Préparer le paquet signé (ROOT, dépend de T015)

Couverture : US4 ; FR-017 ; SC-006.
Sources/zone autorisées : `/Users/moi/.cache/bridget-delivery164-20261013`.
- Construire dans un stage Mac natif avec caches autorisés ; une seule compilation de flotte. Si164 a terminé ses gates source T013/T014/T015 à temps, paquet commun163+164 avec preuves et gates produit distinctes. Ne pas embarquer de code164 non validé. Sinon le paquet163 seul reste possible ; aucun cycle avec G8/G9 de Bridget163.
- Vérifier signatures, sources et validations de paquet existantes. Garder le rollback valide ; aucun remplacement installé tant que les gates requises ne sont pas satisfaites.


- [ ] T017 — Vérifier la présentation intégrée (ROOT, dépend de T015/T016)

Couverture : US1–US4 ; FR-001/002/006/014/015 ; SC-004/006.
Sources/zone autorisées : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/tests164.md`.
- Conduire S164-12 une fois dans un client isolé avec données synthétiques : lecture cohérente, activité repliée, détails accessibles, pages/gap/résultat/contrôles.
- Consigner captures et limites réelles web/desktop/mobile. Aucun nouveau serveur pointé sur données actives, aucun redémarrage T3 ; recette web ne vaut pas recette native mobile.


- [ ] T018 — Installer et fermer G8 (ROOT, dépend de T016/T017)

Couverture : US4 ; FR-001/017 ; SC-006.
Sources/zone autorisées : `/Applications/T3 Code (Local).app`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/`.
- L’activation du daemon/worker163 attend la clôture des tâches natives productrices, mission code164 incluse. Ne pas interrompre le développeur actif ; après son retour stable et la clôture, ROOT exécute les contrôles164 isolés directement sans enfant actif.
- Installer atomiquement le paquet signé et vérifier versions/hashes/signatures et état des processus existants. Préserver T3 sans relance.
- Attester livraison Git/disque, ressources et fonctionnement Bridget indépendant. Expliquer que l’UI active reste l’ancien code jusqu’à la prochaine relance autorisée.


- [ ] T019 — Nettoyer164 et fermer G9 (ROOT, dépend de T018)

Couverture : US4 ; FR-017 ; SC-006.
Sources/zone autorisées : `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/`, `/Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante`, `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission`.
- Fermer les producteurs propres, archiver les preuves ignorées avec SHA, vérifier lsof/inodes/cwd/propreté/merge/push ; retirer uniquement branches, worktrees, copie Cartae et TMP164 devenus inutiles.
- Le worktree163 distant reste justifié tant que le child164 y est lié. À la clôture retirer sa copie `.connector164`, puis163 si gardes satisfaites ; conserver rollback, DB, preuves, caches encore utiles et travaux étrangers. Aucune suppression forcée ni contournement d’une garde.


- [ ] T020 — Clore les métadonnées et le reçu (ROOT, dépend de T018/T019)

Couverture : US4 ; FR-017 ; SC-006.
Sources/zone autorisées : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante`.
- Synchroniser les preuves, progression20/20 seulement si chaque tâche est réellement satisfaite, statuts valides et limites conservées.
- Commit et push des métadonnées finales Bridget 164, vérifier reçus Git/disque/runtime/cleanup. Rapport final self-contained avec chemins absolus complets ; aucun travail testé/audité seul présenté comme livré.

## Convergence

Passage1 sur source R4 : NON CONVERGED, finding C164-01. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p1.md. Les vingt tâches initiales et leurs cases sont conservées.

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

R5b/P2 conservés comme checkpoints historiques. F1/F2/F3 confirmés, correction native R6 ouverte sans contrôle exécuté ni provenance finale. Rapport : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r1.md.

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

Les reçus restent historiques et byte-identiques : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p3.md` décrit le compteur2/22 au moment de sa lecture ; il ne sert pas de compteur courant. Preuves de source finale : `/Users/moi/.cache/bridget-delivery164-20261013/tests/validation164-r6-final.json` et `/Users/moi/.cache/bridget-delivery164-20261013/remote-code/source-transfer164-r6-final.json`. Contre-revue : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r2.md`. Audit : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/audits/2026-10-14/session-2026-10-14-spec-164-delta-r6-01/validator-receipt.json`.

Cette acceptation ne relance ni ne réatteste les tests. T015–T020 restent non cochées : intégration Git, paquet, recette UI, installation, nettoyage et reçu final. Aucun succès de livraison ou runtime n'est déduit.
