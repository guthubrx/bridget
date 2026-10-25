# Tâches181 — Questions humaines natives

Ordre : fondations -> US1/US5 -> US2/US4 -> US3 -> recette. Responsable unique ; la contre-revue est un livrable distinct en lecture seule. Aucune livraison MVP : toutes les stories sont requises.

## Préparation

- [x] T001 Cadrage, spec, plan, recherche et audit de réutilisation dans specs/181-questions-mobile ; indépendance T3 et exclusions utilisateur vérifiables.
- [x] T002 Installer et démarrer l’émulateur Android ; conserver versions, commande et contrôle de boot dans le reçu privé181.

## Fondations et US1/US5 — Demande native, terminal et adaptateurs

- [x] T003 Écrire les tests de validation, propriété, révision, course et rejeu du contrat dans crates/bridget-daemon/src/user_input.rs et les lancer avant la réalisation complète ; états et refus observables.
- [x] T004 Implémenter le magasin natif et ses transitions bornées dans crates/bridget-daemon/src/user_input.rs, store.rs, protocol.rs et daemon.rs ; les tests T003 passent sans affaiblir087/155.
- [x] T005 Exposer l’outil MCP natif et la commande humaine terminal dans mcp.rs, mcp_scopes.rs, cli.rs et communication/client.rs ; test d’identité négatif et réponse reçue sans T3.
- [x] T006 Réemployer le callback Codex natif pour les agents principaux dans wrapper.rs/codex_app_server.rs et daemon/native_child_requests.rs ; le routage155 reste couvert et inchangé.
- [x] T007 Traiter AskUserQuestion dans claude_stream_json.rs via le même contrat ; conserver le refus des autres permissions et vérifier corrélation/réponse native dans les tests du transport.
- [x] T008 Lire les demandes T3 dans t3code_contract_v2.rs et conserver la forme/questions originales ; tests de demande résolue/non reprenable/étrangère et borne.
- [x] T009 Brancher publication, réponse et réconciliation de l’adaptateur dans t3code.rs/t3code_contract.rs ; commandId stable, clôture externe et aucune réinjection de prompt testés.
- [x] T010 Étendre les opérations Questions/HTTP dans daemon/mobile_questions.rs et mobile/http.rs ; tests Rust autorisé/refusé, entrée périmée et origine appareil.

## US2/US4 — Formulaire et réponse libre persistante

- [x] T011 Écrire les tests JVM des réponses simple/multiple/libre/complément et du brouillon par liaison/révision dans android/app/src/test/java/app/cartae/bridget ; lancer le lot concerné.
- [x] T012 Extraire/étendre le formulaire Questions existant pour le fil et le panneau agrandi dans android/app/src/main/java/app/cartae/bridget/MainActivity.kt et le composant partagé ; texte, descriptions, compteur et navigation restent fidèles.
- [x] T013 Étendre LocalStore.kt/MobileViewModel.kt/BridgetClient.kt pour brouillon local, commande idempotente et états confirmés ; fermeture, réponse terminal et résultat inconnu ne perdent aucun texte.
- [x] T014 Ajouter la dictée système au champ libre du formulaire et les libellés FR/EN ; résultat éditable, annulation et moteur indisponible vérifiés ; aucun envoi automatique.

## US3 — Notifications

- [x] T015 Étendre BridgetMessagingService.kt pour relire les demandes authentifiées avant une notification privée avec choix courts/RemoteInput ; conserver poussée générique et heures silencieuses.
- [x] T016 Ajouter le traitement de réponse directe au même parcours de commande dans BridgetMessagingService.kt/AndroidManifest.xml ; identité de liaison, révision, action périmée et double clic testés.

## Vérifications des stories complètes

- [x] T017 Ajouter et lancer les tests instrumentés Android du formulaire, reprise, agrandissement, résultat vocal contrôlé et notification ; rapports conservés hors Git.
- [x] T018 Recette réelle sur émulateur avec producteur/commandant terminal natif, sans T3 : choix/libre/complément, fermeture et réponse reçue une seule fois ; reçus181 privés.
- [x] T019 Recette réelle notification en arrière-plan et dictée ; qualifier le moteur vocal présent et les limites éventuelles, sans présenter une injection de résultat comme reconnaissance réelle.
- [x] T020 Recette contrôlée de l’adaptateur T3 et modes terminal ; préserver les vrais fils, aucun redémarrage fournisseur/T3 pour répondre ; modes indisponibles affichés avec guidage.
- [x] T021 Lancer les tests ciblés Rust/JVM, lint et builds concernés ; fault injection réseau/reprise, révocation et course entre clients ; aucune régression nouvelle bloquante.
- [x] T022 Produire et vérifier le diaporama des circuits réels dans preuves-corrections/2026-10-25-questions-mobile/preuves.html ; rapports, manifestes, captures et reçus privés protégés.

## Dépendances et contrôle final

T003 précède T004 ; T004 précède T005–T010 ; T011 précède T012–T014 ; T010/T013 précèdent T015/T016 ; T017–T022 valident les stories intégrées. Exécutions outils indépendantes possibles, aucun partage concurrent d’écriture.

Converge, contre-revue adverse du code, audit, livraison Git/binaires et nettoyage suivent ces tâches et restent tracés dans plan.md/validation. Le statut Implemented n’est posé qu’à22/22 ; le statut Livré attend le déploiement et l’examen des candidats de nettoyage. Les tâches ajoutées par Converge seront append-only sous une nouvelle section Convergence.

## Correction issue de l’audit

- [x] T023 Préserver les reçus existants à capacité32 dans l’adaptateur T3 ; test avant/après et fermeture explicite.

- [x] T024 Parcourir les pages des notifications natives pour ne pas ignorer une question au-delà des dix plus anciennes ; recette avec plus de10questions actives.

- [x] T025 Éviter la réécriture des tombstones déjà purgés lors des lectures ; index de rétention et test avant/après sans changement de suppression7jours.
