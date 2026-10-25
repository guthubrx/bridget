# Tâches 177 — Bridget Mobile et pilotage fédéré

Statut : In Progress
Date : 2026-10-24
Branche : session-177-bridget-mobile

Le périmètre complet des huit scénarios est conservé. Les cases ne sont cochées qu'après vérification. Aucun lot pilote ne clôt le pipeline. Les tâches bloquées par un accès externe restent ouvertes avec leur précondition nommée.

## Fondations

- [x] T001 Inventorier l'autorité courante, les chemins et les attestations dans specs/177-bridget-mobile/validation/preflight.md ; relever les préconditions réelles sans modifier le runtime. (FR-016, FR-017, FR-018)
- [x] T002 Préciser les contrats d'autorité et de migration dans specs/177-bridget-mobile/contracts/mobile-v1.md et enregistrer l'ADR dans docs/decisions/ ; la preuve d'origine ne doit pas être remplacée par un libellé. (FR-002, FR-003, FR-018, FR-026)
- [x] T003 Choisir et verrouiller le build Android et le serveur HTTP dans specs/177-bridget-mobile/research.md et reuse-audit.md ; installer les outils nécessaires dans un emplacement réutilisé. (FR-021, FR-022, FR-025, FR-027)

## US1 — Association et révocation

- [x] T004 [US1] Écrire les tests d'association, d'expiration, de rejeu après réponse perdue, de portée et de révocation dans crates/bridget-daemon/src/mobile/store.rs. (FR-001, FR-002, FR-003, FR-020)
- [x] T005 [US1] Implémenter le magasin privé des appareils, associations et renouvellements dans crates/bridget-daemon/src/mobile/store.rs ; aucun secret brut persistant. (FR-001, FR-002, FR-020)
- [x] T006 [US1] Ajouter l'administration locale et le QR dans crates/bridget-daemon/src/mobile.rs et cli.rs ; refuser l'émission distante non autorisée. (FR-001, FR-020, FR-021)
- [x] T007 [US1] Implémenter l'authentification et le contrôle de portée dans crates/bridget-daemon/src/mobile/http.rs ; tester lecture, mutation, abonnement et limite de ressources. (FR-003, FR-013, FR-020, FR-025)

## US2 — Annuaire fédéré

- [x] T008 [US2] Réutiliser les lectures natives dans crates/bridget-daemon/src/mobile/client.rs pour machines, projets et agents ; autoriser avant pagination. (FR-004, FR-005, FR-006)
- [x] T009 [US2] Tester les filtres, les identités voisines, les machines indisponibles et les projets hors portée dans crates/bridget-daemon/src/daemon/mobile_operations.rs. (FR-003, FR-004, FR-005, FR-006)

## US3 — Messages et résultats

- [x] T010 [US3] Adapter l'envoi humain natif et la lecture de reçu dans crates/bridget-daemon/src/mobile/client.rs ; conserver clé, date, enveloppe et refus natifs. (FR-008, FR-009, FR-010, FR-014)
- [x] T011 [US3] Exposer les pages de journal et le suivi borné dans crates/bridget-daemon/src/mobile/client.rs et http.rs ; conserver lacunes, séquences et révocation. (FR-007, FR-015)
- [x] T012 [US3] Adapter les artefacts natifs dans crates/bridget-daemon/src/mobile/client.rs ; aucune lecture par chemin libre, limiter types et tailles. (FR-003, FR-011, FR-025)
- [x] T013 [US3] Prouver sur socket réelle la perte de réponse et le rejeu strict dans crates/bridget-daemon/src/daemon/mobile_operations.rs ; une commande et une remise. (FR-008, FR-009, FR-010, FR-014)

## US4 — Tâches et questions

- [x] T014 [US4] Adapter la demande de travail vers un agent enregistré par l'envoi humain natif ; annoncer la création directe d'agent comme indisponible et ne jamais emprunter l'instance d'un agent. (FR-002, FR-003, FR-012)
- [x] T015 [US4] Réutiliser les projections, réponses, continuations, pauses et annulations natives dans crates/bridget-daemon/src/mobile/client.rs et daemon.rs ; exposer les seules actions autorisées. (FR-012, FR-013, FR-014)
- [x] T016 [US4] Relier les questions au guichet existant dans crates/bridget-daemon/src/mobile/client.rs ; vérifier options, portée et acquittement. (FR-003, FR-012, FR-013)
- [x] T017 [US4] Tester le refus de droits, la pause, l'annulation tardive et les commandes inconnues dans crates/bridget-daemon/src/daemon/mobile_operations.rs et crates/bridget-daemon/src/mobile/http.rs, au sein des fixtures natives existantes. (FR-003, FR-010, FR-012, FR-013)

## US5 — Application et reprise

- [x] T018 [US5] Créer le module Android dans android/ avec versions verrouillées, manifest et APK de test ; aucun secret dans les fichiers suivis. (FR-022, FR-023, FR-026)
- [x] T019 [US5] Implémenter le stockage protégé et la file d'envoi durable dans android/app/src/main/java/app/cartae/bridget/LocalStore.kt ; clé persistée avant envoi. (FR-002, FR-009, FR-011, FR-020)
- [ ] T020 [US5] Implémenter le client HTTPS et la réconciliation dans android/app/src/main/java/app/cartae/bridget/BridgetClient.kt ; aucune nouvelle clé automatique sur réponse perdue. (FR-009, FR-010, FR-014, FR-015)
- [x] T021 [US5] Créer association QR et gestion des appareils dans android/app/src/main/java/app/cartae/bridget/MainActivity.kt avec chaînes françaises et anglaises. (FR-001, FR-020, FR-023)
- [ ] T022 [US5] Créer annuaire, filtres mémorisés et favoris dans android/app/src/main/java/app/cartae/bridget/MainActivity.kt ; afficher machine, projet, état et fraîcheur. (FR-004, FR-005, FR-006, FR-023)
- [ ] T023 [US5] Créer conversation, brouillons, pièces jointes et lecture des résultats dans android/app/src/main/java/app/cartae/bridget/MainActivity.kt ; actions accessibles au clavier et avec grands textes. (FR-007, FR-008, FR-010, FR-011, FR-023)
- [ ] T024 [US5] Créer le suivi et les actions de tâches dans android/app/src/main/java/app/cartae/bridget/MainActivity.kt ; ne proposer que les capacités annoncées. (FR-012, FR-013, FR-014, FR-023)
- [ ] T025 [US5] Tester fermeture de processus, changement de réseau, reçu perdu et renouvellement dans android/app/src/test/ ; exécuter le build et les tests Android. (FR-009, FR-010, FR-015, FR-020, FR-022)

## US6 — Serveur permanent et domaine

- [x] T026 [US6] Implémenter l'adaptation d'attestation liée à la machine d'origine puis prouver sa portabilité et la préservation des autorités dans une fédération isolée ; ajouter les tests dans crates/bridget-daemon/src/origin_proof.rs et crates/bridget-daemon/src/daemon/origin_proof.rs et les faits dans validation/migration.md. (FR-016, FR-017, FR-018)
- [x] T027 [US6] Préparer la migration et le retour arrière dans scripts/federate-ssh.sh ou le mécanisme existant retenu, sans seconde autorité active ni copie de base ouverte. (FR-016, FR-017, FR-018, FR-021)
- [x] T028 [US6] Préparer puis appliquer la route bridget.cartae.app dans l'installation existante ; vérifier DNS, certificat, écoute loopback et absence de changement des autres routes dans validation/network.md. (FR-003, FR-021, FR-025)
- [x] T029 [US6] Effectuer la bascule autorisée après préflight ; vérifier identités, reçus, tâches et raccordement des machines dans validation/migration.md. (FR-016, FR-017, FR-018)
- [ ] T030 [US6] Vérifier depuis le téléphone le pilotage serveur Mac éteint et le retour du Mac ; consigner la preuve réelle dans validation/acceptance.md. (FR-006, FR-016, FR-017, FR-018)

## US7 — Notifications

- [x] T031 [US7] Adapter le canal existant pour les notifications minimales dans crates/bridget-daemon/src/mobile/notifications.rs ; configuration privée et absence de texte métier. (FR-019, FR-025)
- [x] T032 [US7] Ajouter le service Android et les préférences de notification dans android/app/src/main/java/app/cartae/bridget/BridgetMessagingService.kt et les ressources FR/EN. (FR-019, FR-020, FR-023)
- [ ] T033 [US7] Configurer le service de notification autorisé, vérifier appareil en arrière-plan et refus de permission ; consigner la recette dans validation/notifications.md. (FR-019, FR-025)

## US8 — Livraison complète

- [x] T034 [US8] Réutiliser l'installation pour le composant serveur et le raccordement des machines ; documenter commandes et prérequis dans docs/mobile.md et docs/mobile.fr.md. (FR-021, FR-024, FR-027)
- [x] T035 [US8] Signer l'APK avec une clé privée sauvegardée et ajouter la vérification de mise à jour dans android/ et le canal de publication existant ; préserver données locales et compatibilité. (FR-020, FR-022)
- [ ] T036 [US8] Exécuter les parcours QR, Wi-Fi/4G, erreurs, fichiers et mise à jour sur téléphone ; produire les preuves UI réelles et le bilan dans validation/acceptance.md. (FR-001, FR-003, FR-010, FR-011, FR-013, FR-015, FR-022, FR-023)
- [x] T037 [US8] Terminer les documentations publiques EN/FR dans docs/mobile.md, docs/mobile.fr.md, docs/README.md et README.md ; vérifier les liens et l'absence de secret. (FR-024, FR-025, FR-026)
- [ ] T038 Exécuter Converge après la dernière implémentation, sans modifier le code ; tracer correspondance FR/code/test dans specs/177-bridget-mobile/validation/convergence.md. (FR-001, FR-027)
- [ ] T039 Faire la contre-revue d'implémentation si admise et l'audit applicable ; lire la décision et les blocages dans specs/177-bridget-mobile/validation/audit.md. (FR-003, FR-025, FR-026)
- [ ] T040 Intégrer et pousser les sources et artefacts après les gates ; vérifier le SHA distant et les paquets installés dans specs/177-bridget-mobile/validation/delivery.md. (FR-021, FR-022, FR-024, FR-027)
- [ ] T041 Traiter les worktrees, branches et caches du périmètre avec les outils existants ; préserver preuves et version de retour, puis relever les conservations dans validation/delivery.md. (FR-027)

## Corrections issues de la recette physique

- [x] T042 Rendre la conversation compacte, ouvrir sur les messages récents, ajouter le retour aux derniers messages et un formatage lisible, avec FR/EN et preuve Pixel. (FR-007, FR-023)
- [ ] T043 Afficher les états d’envoi et de suivi mesurés, relire après envoi et permettre la reconnexion explicite sans nouvelle clé. (FR-009, FR-010, FR-015)
- [x] T044 Corriger le guidage d’une instruction humaine : répondre dans le fil, ne pas envoyer à human ; conserver canons, droits et messages interagents, avec tests. (FR-002, FR-008, FR-009)
- [ ] T045 Qualifier la réponse humaine et le suivi sur le parcours réel après mise à jour signée, sans perte de l’association. (FR-007, FR-010, FR-022)

## Dépendances et exécution

T001 → T002 → T003 précèdent les adaptations d'autorité. US1 précède toutes les données privées et mutations. US2 et US3 précèdent l'interface connectée. US4 préserve les politiques et puits natifs. L'application peut être construite contre le contrat stabilisé avant la bascule du maître. T026–T030 exigent le succès du préflight et les accès de déploiement ; T033 exige la configuration de notification ; T030 et T036 exigent le téléphone de recette. Les tâches d'un même fichier sont séquentielles. Aucune délégation de fabrication n'est lancée par défaut.

## Vérifications par scénario

US1 : association à usage unique, réponse perdue, expiration et révocation. US2 : filtres et refus hors portée. US3 : journal borné, artefacts et socket réelle avec reçu perdu. US4 : droits et états natifs. US5 : stockage durable, processus arrêté et réseau. US6 : migration isolée puis réelle et Mac éteint. US7 : notification physique sans contenu métier. US8 : APK signé, mise à jour, sources poussées et nettoyage.

## Statut initial des préconditions externes

- Contre-revue de plan : demandée mais refusée avant démarrage, aucun verdict acquis ; voir reuse-audit.md.
- DNS : enregistrement A du sous-domaine créé et relu ; route HTTPS active, backend permanent r6 actif.
- Notifications : projet Spark et application créés, FCM V1 activé, compte serveur dédié privé ; réception réelle encore ouverte.
- Téléphone : Pixel 10a/Android 17 associé et utilisé réellement. USB absent au dernier contrôle r10b ; mise à jour finale, recette réseau et notification physique ouvertes.
- Migration : attestation et préparation d’origine prouvées dans la fédération isolée ; services permanents et bascule r6 vérifiés ; recette physique Mac éteint encore ouverte.


## Retour utilisateur depuis le téléphone (24 octobre, 20:38)

- [ ] T046 Afficher des noms courts et repères de projet, distinguer les homonymes sans modifier leur identité native.
- [ ] T047 Ouvrir sur une page récente ; charger les pages antérieures explicitement, en conservant les curseurs et les limites.
- [ ] T048 Garder les derniers messages visibles à l’ouverture du clavier, sans arracher la lecture d’un ancien message.
- [ ] T049 Rétablir le suivi vide après intégration des correctifs natifs concurrents ; conserver d5af84f8 au prochain binaire et qualifier le guidage QR.

## Derniers retours Pixel — vocabulaire et suivi

- [ ] T050 Corriger le nom `.git`, le faux état de reconnexion, la fin d’un long message et le reçu lu sans renvoi ; qualifier le clavier et la reconnexion réelle.
- [ ] T051 Ajouter « Transmettre maintenant » avec la même clé et un repli en file, une autorité mobile limitée au message et les contrôles de portée.
- [ ] T052 Afficher les activités disponibles dans le journal natif, en lignes compactes, sans nouvelle dépendance envers une interface de bureau.

## Point de livraison r10b

Voir validation/delivery.md. T050–T052 sont implémentées et contrôlées par les
tests ciblés. Leur recette physique finale reste ouverte, donc leurs cases ne
sont pas cochées. Les deux revues indépendantes sont traitées ; aucun audit
global nouveau n’est revendiqué. Les tâches réseau, notifications, autonomie
Mac éteint, pièces jointes et tâches métier restent suivies ci-dessus.

## Retours de recette physique r10c

- [x] T053 Compacter l’annuaire, conserver seulement l’étoile de favori et placer un repère de projet coloré à gauche ; vérifier sur le Pixel.
- [x] T054 Installer le vrai logo Bridget dans l’icône Android et vérifier son rendu installé.
- [x] T055 Transmettre et afficher commandes, paramètres et sorties des outils du fil principal ; qualifier le dépliage réel et signaler les limites de transport.
- [x] T056 Corriger la réutilisation d’un jeton FCM après réassociation et vérifier la réception réelle sur l’association courante.

APK r10c installé sans désinstallation. Les 67 contrôles natifs passent ; la
réassociation a une preuve RED puis GREEN. L’annuaire et le détail complet des
outils ont été vus sur le Pixel. La notification réelle est enregistrée par
Android et sa réception est confirmée par l’utilisateur. Son ouverture par
toucher, les recettes 4G/Mac éteint et les autres tâches ouvertes ci-dessus ne
sont pas considérées comme validées.

## Lecture riche demandée sur le Pixel

- [x] T057 Remplacer le Markdown partiel par le rendu natif, ajouter la coloration du code et qualifier les principaux langages.
- [x] T058 Ouvrir les blocs HTML et fichiers partagés dans un aperçu isolé ; préserver les droits et vérifier le circuit réel sur le Pixel.


## Recette de reprise — 25 octobre 2026

- [x] T059 Charger les fichiers à l’ouverture, conserver le bouton Actualiser et
  aligner les formats FR/EN. RED observé puis GREEN sur le Pixel après mise à jour.

T057/T058 : Markdown et Python coloré vus dans cette conversation ; aperçu de
bloc HTML ouvert puis fermé. Import HTML via le sélecteur Android, aperçu du
fichier partagé et export vérifié par SHA-256. Les tests natifs couvrent aussi
un artefact Html produit par le service canonique, son type avec charset et le
refus de lecture depuis une autre conversation. 69 PASS, 1 recette d’origine
ignorée ; Android 14 PASS, lintRelease/assembleRelease PASS.

Le client HTTPS, l’annuaire, les tâches et les notifications sont implémentés.
Les cases de recette plus larges restent ouvertes : la présente reprise ne
remplace pas une instruction et son résultat sur chaque réseau ni le test Mac
réellement éteint. Le détail par tâche et la prochaine action figurent dans
validation/r10e.md. Statut global : In Progress.
