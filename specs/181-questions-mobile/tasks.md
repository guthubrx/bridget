# Tâches181 — Questions humaines natives

Ordre : fondations -> US1/US5 -> US2/US4 -> US3 -> recette. Responsable unique ; la contre-revue est un livrable distinct en lecture seule. Aucune livraison MVP : toutes les stories sont requises.

## Préparation

- [x] T001 Cadrage, spec, plan, recherche et audit de réutilisation dans specs/181-questions-mobile ; autorité native Bridget et exclusions utilisateur vérifiables.
- [x] T002 Installer et démarrer l’émulateur Android ; conserver versions, commande et contrôle de boot dans le reçu privé181.

## Fondations et US1/US5 — Demande native, terminal et adaptateurs

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/tasks.md#extrait-1).

## US2/US4 — Formulaire et réponse libre persistante

- [x] T011 Écrire les tests JVM des réponses simple/multiple/libre/complément et du brouillon par liaison/révision dans android/app/src/test/java/app/cartae/bridget ; lancer le lot concerné.
- [x] T012 Extraire/étendre le formulaire Questions existant pour le fil et le panneau agrandi dans android/app/src/main/java/app/cartae/bridget/MainActivity.kt et le composant partagé ; texte, descriptions, compteur et navigation restent fidèles.
- [x] T013 Étendre LocalStore.kt/MobileViewModel.kt/BridgetClient.kt pour brouillon local, commande idempotente et états confirmés ; fermeture, réponse terminal et résultat inconnu ne perdent aucun texte.
- [x] T014 Ajouter la dictée système au champ libre du formulaire et les libellés FR/EN ; résultat éditable, annulation et moteur indisponible vérifiés ; aucun envoi automatique.

## US3 — Notifications

- [x] T015 Étendre BridgetMessagingService.kt pour relire les demandes authentifiées avant une notification privée avec choix courts/RemoteInput ; conserver poussée générique et heures silencieuses.
- [x] T016 Ajouter le traitement de réponse directe au même parcours de commande dans BridgetMessagingService.kt/AndroidManifest.xml ; identité de liaison, révision, action périmée et double clic testés.

## Vérifications des stories complètes

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/tasks.md#extrait-2).

## Dépendances et contrôle final

T003 précède T004 ; T004 précède T005–T010 ; T011 précède T012–T014 ; T010/T013 précèdent T015/T016 ; T017–T022 valident les stories intégrées. Exécutions outils indépendantes possibles, aucun partage concurrent d’écriture.

Converge, contre-revue adverse du code, audit, livraison Git/binaires et nettoyage suivent ces tâches et restent tracés dans plan.md/validation. Le statut Implemented n’est posé qu’à22/22 ; le statut Livré attend le déploiement et l’examen des candidats de nettoyage. Les tâches ajoutées par Converge seront append-only sous une nouvelle section Convergence.

## Correction issue de l’audit

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/tasks.md#extrait-3).

- [x] T024 Parcourir les pages des notifications natives pour ne pas ignorer une question au-delà des dix plus anciennes ; recette avec plus de10questions actives.

- [x] T025 Éviter la réécriture des tombstones déjà purgés lors des lectures ; index de rétention et test avant/après sans changement de suppression7jours.
