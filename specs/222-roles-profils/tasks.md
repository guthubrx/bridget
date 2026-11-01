# Tâches222 — rôles et spécialistes durables
Statut : In Progress. Périmètre complet US1–US7, sans report MVP.
Dépendances : ordre ci-dessous ; tests ciblés intégrés à la responsabilité modifiée.
Tous les chemins ci-dessous désignent le worktree serveur222. Les commandes Cargo passent par scripts/build.py.


## Préparation

- [x] T001 Conserver le cadrage validé, la spec et la checklist ; preuve : décisions1–17 et FR001–031. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/specs/222-roles-profils/spec.md`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/specs/222-roles-profils/checklists/requirements.md`.
- [x] T002 Fermer les objections de contrat du plan, tracer les arbitrages et vérifier le gate anti-doublon ; aucune nouvelle tâche tant qu'un contrat P1 reste indéfini. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/specs/222-roles-profils/plan.md`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/specs/222-roles-profils/contracts/closure-after-review.md`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/specs/222-roles-profils/reuse-audit.md`.

## Fondations

- [x] T003 Définir les requêtes fermées version1 et négocier SpecialistsV1 ; ancien pair refuse sans effet. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-transport/src/protocol.rs`.
- [x] T004 Étendre le domaine AgentProfileStore : profils réutilisables immuables, spécialistes liés aux agent_id, déclarations de rôles et grants ; migration idempotente, aucune altération des anciens profils. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/agent_profile.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/agent_profile/specialists.rs`.

## US1

- [x] T005 [US1] Déclarer/retirer ses rôles, rechercher avec périmètre, présence observée et sélection stable expliquée ; tests autre identité/absence/officiel refusé. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/daemon/specialists222.rs`.

## US2

- [x] T006 [US2] Créer/conserver/proposer un profil et une révision ; activer seulement sur décision humaine corrélée ; test concurrence de révision et anciennes consignes conservées. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/agent_profile/specialists.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/user_input.rs`.
- [x] T007 [US2] Réutiliser l'état appliqué de session et figer profil/révision/digest dans la soumission ; changer le profil ne modifie pas une mission admise. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/wrapper.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/execution_store.rs`.

## US3

- [x] T008 [US3] Définir mandats par défaut et budget absent par défaut ; créer/révoquer MandateGrant depuis contrôle humain, contrôler projets et autorisations natives. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/daemon/specialists222.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/fleet.rs`.
- [x] T009 [US3] Réserver attribution et soumission prepared dans une seule transaction SQLite ; canon divergent refusé ; contre-épreuves de coupure aux quatre stades. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/execution_store.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/agent_profile/specialists.rs`.
- [x] T010 [US3] Relier le réveil au cycle natif : grant vérifié sous la garde de réservation et avant RELEASE, session nommée sans reprise aveugle, arrêt inconnu bloquant ; test révocation entre réserve et remise. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/daemon/specialists222.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/daemon.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/lifecycle.rs`.
- [x] T011 [US3] Activer une soumission prepared corrélée une seule fois, réutiliser envoi/reçus natifs et restituer même responsable après reconnexion ; aucune file ou copie d'état nouvelle. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/execution_store.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/daemon/specialists222.rs`.
- [x] T012 [US3] Vérifier les prérequis de la cible, préserver les contraintes et rendre missing/unverified avec l'action précise ; préparation autorisée confiée comme mission, pas de commande arbitraire issue du profil. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/daemon/specialists222.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/registry.rs`.
- [x] T013 [US3] Rendre le spécialiste joignable occupé et ses demandes visibles ; utiliser les possibilités natives de remise et de délégation indépendante sans interruption silencieuse. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/daemon/specialists222.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/t3code.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/wrapper.rs`.

## US4

- [x] T014 [US4] Écrire des connaissances typées et sourcées ; vérifier l'identité et la portée de la source ; correction révisée et tombstone empêchant la réintroduction d'une source oubliée. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/agent_profile/specialists.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/daemon/specialists222.rs`.
- [x] T015 [US4] Rappeler les mémoires autorisées : pertinence lexicale, préférence projet à pertinence comparable, date/identifiant ; tests interprojets/refus/pagination sans limite globale de conservation. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/agent_profile/specialists.rs`.
- [x] T016 [US4] Accrocher la capture automatique aux résultats corrélés et observations attestées ; mêmes sources idempotentes, résultats rapportés distingués des faits prouvés. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/daemon/specialists222.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/daemon/native_delegation.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/daemon.rs`.
- [x] T017 [US4] Composer le contexte ciblé de mission avec profil figé et souvenirs sourcés ; la fenêtre de contexte ne supprime aucune donnée durable. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/wrapper.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/daemon/specialists222.rs`.

## US5

- [x] T018 [US5] Exposer les opérations conversation/MCP avec scopes de lecture, déclaration propre, demande et proposition ; un agent ne peut approuver sa propre durabilité. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/mcp.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/mcp_scopes.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/communication/client.rs`.
- [x] T019 [US5] Exposer l'administration et le catalogue en CLI via le contrôle humain existant ; mêmes révisions et reçus que les autres interfaces. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/cli.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/communication/client.rs`.
- [x] T020 [US5] Raccorder l'API humaine authentifiée et ses scopes ; consultation sans réveil, commande corrélée et refus explicites. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/mobile/http.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/daemon/mobile_operations.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/mobile/client.rs`.
- [x] T021 [US5] Ajouter le catalogue Android, fiche profil/mandat/budget, activation humaine et désactivation non annulante ; navigation et textes FR/EN, même état natif. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/android/app/src/main/java/app/cartae/bridget/SpecialistsScreen.kt`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/android/app/src/main/java/app/cartae/bridget/MainActivity.kt`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/android/app/src/main/java/app/cartae/bridget/BridgetClient.kt`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/android/app/src/main/res/values/strings.xml`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/android/app/src/main/res/values-fr/strings.xml`.
- [x] T022 [US5] Compléter mémoire/versions/missions et diagnostics du catalogue ; vérifier états vide, hors ligne, occupé, bloqué et perte de réponse. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/android/app/src/main/java/app/cartae/bridget/SpecialistsScreen.kt`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/android/app/src/test/java/app/cartae/bridget/SpecialistsTest.kt`.

## US6

- [x] T023 [US6] Exporter/importer uniquement une définition avec provenance ; aucune activation, compte, grant ou mémoire implicite ; conflits et champs inconnus refusés. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/agent_profile/specialists.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/daemon/specialists222.rs`.
- [x] T024 [US6] Convertir une conversation sélectionnée : même agent_id, anciennes tâches intactes, historique paginé/reprenable et connaissances rapportées avec source ; pas de scan général. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/daemon/specialists222.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/agent_profile/specialists.rs`.

## US7

- [x] T025 [US7] Conserver et utiliser les compositions facultatives avec livrables/dépendances, clés stables et admissions natives ; test issue partielle sans duplication ou arrêt d'un membre indépendant. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/agent_profile/specialists.rs`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/crates/bridget-daemon/src/daemon/specialists222.rs`.

## Finition

- [x] T026 Documenter les parcours FR/EN, migrations, sources de preuve, mandats et limites ; chaque refus indique portée et prochaine action utile. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/docs/delegation-native.fr.md`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/docs/delegation-native.md`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/docs/decisions/222-specialistes-durables.md`.

## Validation

- [x] T027 Exécuter les tests ciblés du domaine et des raccords avec build.py, puis les circuits natifs techniques du projet ; préserver les tâches métier et réutiliser les preuves valides. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/specs/222-roles-profils/validation.md`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/scripts/build.py`.
- [ ] T028 Exécuter Converge contre toutes les FR, ajouter seulement les manques ; faire contre-revoir l'implémentation et traiter les objections vérifiées. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/specs/222-roles-profils/tasks.md`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/specs/222-roles-profils/converge.md`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/specs/222-roles-profils/adversarial-review-implementation.md`.
- [x] T029 Exécuter l'audit applicable et fermer les défauts bloquants ; résultat et limites lisibles, aucun succès global inventé. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/specs/222-roles-profils/validation/audit`.

## Livraison

- [ ] T030 Construire les composants concernés Linux/Mac/Android, qualifier un retour compatible et intégrer/pousser le lot complet sans embarquer d'autres travaux. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/specs/222-roles-profils/validation.md`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/scripts/build.py`.
- [ ] T031 Activer via maintenance native, contrôler les images réellement chargées et le lien, puis vérifier le circuit visuel et publier ses preuves ; nommer tout client ancien. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/specs/222-roles-profils/validation.md`, `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/preuves-corrections/2026-11-01-222-specialistes/preuves.html`.

## Nettoyage

- [ ] T032 Contrôler fusion/push/propreté/usages et nettoyer seulement les ressources éligibles ; conserver les tâches, preuves, actif/retour et usages non attestés. Fichiers : `/home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/specs/222-roles-profils/validation.md`.

## Couverture
US1 T005 ; US2 T004/T006/T007 ; US3 T008–T013 ; US4 T014–T017 ; US5 T018–T022 ; US6 T023/T024 ; US7 T025.
FR001–003 T005 ; FR004–006 T004/T006/T007 ; FR007–016 T008–T013 ; FR017–021 T014–T017 ; FR022–024 T018–T022 ; FR025–027 T023/T024 ; FR028 T025 ; FR029 T026 ; FR030 T003/T027 ; FR031 T030–T032.

## Précisions de vérification après revue2
T009/T011/T027 couvrent les préparations sans exécution : aucun drain ni recovery ne promeut prepared ; la promotion exige une attribution222 corrélée et le même canon. T010 couvre révocation avant RELEASE. T016 ne capture que les deux sources attestées nommées au contrat.

## Preuve de code avant activation
13 contrôles daemon222,1 transport222 et1 autorité181 PASS sur la source intégrée ; neuf contrôles de modèle Android et assembleRelease PASS. T021/T022 attendent leur circuit authentifié réel ; T027 attend réveil/réponse/collecte en fonctionnement. T028 attend la fermeture de la porte de livraison FR031 ; T029 désigne l'audit de code FR001–030, pas une qualification runtime.

## Raccord observé en qualification Android
- [x] T033 Corriger la négociation mobile spécifique222 : accepter exactement MobileOperationsV1 + SpecialistsV1 pour cette opération, conserver la garde historique pour les autres ; test du vrai handler avec portée limitée, lecture autorisée, absence de remise et révocation, puis relire le catalogue Android réel.

## Preuves finales de qualification
T021/T022/T027/T033 vérifiées par les tests ciblés et le circuit réel décrit dans validation.md. T028/T031 restent ouverts pour le client MCP de CE fil. T030 attend Git/push effectifs ; T032 attend le bilan borné. Aucun nouveau code manquant FR001–030 identifié.
