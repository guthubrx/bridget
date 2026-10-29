# Validation208 — livraison réelle du29octobre2026

## Sources et contrôles

Source fonctionnelle finale7b34131bffa0fc28592435004272e6e0c3e53602, incluant f60daa75dbad40f4caa70cfc1c1ca1651f6f93e8. Intégrée à main et poussée.

- 3 tests208 PASS sur la tête finale : plafond équivalent après preuve fraîche, projection api_error, diagnostic de préparation interrompue par époque.
- 4 tests206 et5 tests172_continuation PASS sur le correctif de droits initial ; aucun changement de ces gardes dans le complément.
- Guides :32pages/522liens/13paires de langues PASS ; git diff --check PASS.
- Revue b23ee0f6-b1aa-4693-aa2c-bc3d413789b7 favorable sur f60daa75.
- Revue cb149ce8-51fc-418a-9054-1552d33bb16a favorable sur le complément7b34131b, sans objection concrète.
- Revues statiques, sans exécution indépendante des tests. Résultats lus, acquittés, missions clôturées nativement.
- Builds LinuxCartae, Linuxglibc2.36 et macOSARM64 PASS par scripts/build.py, deux jobs au maximum, cache réutilisé.

## Chargement effectif

Build208-attestation-7b34131b, version commerciale0.6.3 inchangée. Vérification par chemin de l’exécutable du processus et SHA256 après démarrage.

| Machine | Processus chargés | SHA256 |
|---|---|---|
| Cartae | maître1646199, worker1646513, mobile1646269 | 0827ea969f3305a25600b56e09256a6061a812588ca8f87b79c85acb6e75de58 |
| SYS | worker344039 | 6118ce20c53bd9a1c5577842a60762004e33f13f6008cd307f6c5d8aae4772f5 |
| KS | worker485841 | 6118ce20c53bd9a1c5577842a60762004e33f13f6008cd307f6c5d8aae4772f5 |
| Mac | worker70280, pontT370284 | 384d3171184903e1763540eecf45d6ad82f6e45b76e38eb4cd1939063b80d343 |

UpdaterCartae1646273 et Mac70286 actifs : processusPython normaux. Les lanceurs pointent sur le binaire208 installé. Un MCP déjà ouvert peut encore exécuter son ancien binaire jusqu’à reconnexion ; aucun chargement208 de tous les anciens clients n’est prétendu.

- Coupure du maître2,01s ; sauvegarde cohérente avant/après arrêt sous /home/moi/.local/share/bridget-maintenance/208-attestation-production-20261029T084141Z.
- Workers remplacés par leurs unités avec KillMode=process et SendSIGKILL=no. Aucun serviceOpus2D/NE02 modifié.
- WrappersCartae1162524/birth277006331 et2500061/birth270585460 préservés.
- WrapperKS476465/birth3118272951 préservé ; aucun reçu actifSYS/Mac lors du changement.
- Gel relu false ; aucune mission métier créée, remplacée ou relancée pour la maintenance.
- Reçus locaux sous /home/moi/.cache/bridget208 et /Users/moi/.cache/bridget208, avec les empreintes exactes.

## Qualification ClaudeCodeCartae et exécution après chargement

Mise à jour officielle2.1.251→2.1.312 autorisée par l’utilisateur. Détail et preuve historique dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/208-attestation-equivalente/claude-cartae.md.

- Première qualification réelle via207 : tâche2991e808-b9a3-4753-96f9-314aad6fd56f, résultatBRIDGET208_OPUS_CARTAE_OK.
- Qualification supplémentaire via maître ET worker208 : demandebridget208-opus-apres-chargement-20261029-02, tâche0b0de9e7-6766-4b9f-8623-09fbd130a556 ; connexionclaude-cartae, modèleclaude-opus-5-5 ; résultatBRIDGET208_LOADED_OPUS_OK, erreur nulle, lu et acquitté, clôture nativecleanup_done=true/guard_released=true.
- L’essai01 de ce dernier contrôle a été refusé sans tâche pour keep_alive30<minimum60. La valeur a été corrigée après lecture du catalogue. L’appel02 a d’abord reçu origin_proof_unavailable/not_recorded ; après réattestation du parent, son rejeu strict a admis la même clé. Cause plus précise de ce refus transitoire non établie, aucune révocation déduite.
- Les anciennes erreursAPI400 étaient dues à la versionCLI trop ancienne, pas à un compte/quota/durée établis.
- Comptes et modèles inchangés. Ancienne versionClaude conservée pour retour.

## Issue des tâches historiques et portée

d2f53478 etdc48d684 : cancelled/cleanup_done ; erreur historique revision conservée, aucune réanimation.5c44c784 également cancelled/cleanup_done. Le correctif rend les futures comparaisons tolérantes au seul renouvellement d’une preuveprovider_turn fraîche et équivalente ; native_wrapper et les changements de droits restent stricts.

28414216 était une autre cause : handler de préparation interrompu par redémarrage207, aucunchild_instance/launch/mission côtémaître, reçuGit et dossierKS déjàprésents. La projection208 explique ce cas sans inventer arrêt physique ni reprise. Le propriétaire a reçu le parcoursCancel→clôture→inspection dossier avant nouvelleclé. Dernière lecture native : cancelled, errornull, cleanup_done=true, child_instance et missionabsents ; guard_released=false n’est pas présenté comme libéré. Aucun Cancel de cette tâche par le mainteneur. Le dossierKS reste sous contrôle de son propriétaire.

## Information des propriétaires

Résultats transmis dans les filsT3existants :
- monuments32e1b8f8-938b-488c-a05e-67200732b973 : bridget208-monuments-livraison-complete-20261029-01, steeredrun531.
- Politique sol5d7b4e02-9cf7-42e7-96a5-71d77fbf4f77 : bridget208-politique-livraison-complete-20261029-01, steeredrun251.

NotificationBridgetmcp-50239-6ae3085c-1e acceptéein_flight ; ce reçu n’est pas compté comme une lecture.

## Nettoyage et conservations

- Versionactive208 et retour207 conservés. Exécutables201/183 encore utilisés par anciens wrappers ou clients conservés.
- Sources207/208, caches partagés et preuves utiles conservés ; aucun target copié.
- installed réussi surMac etSYS. SurCartae, appel effectué mais refus d’entretien : observation des processus incomplète. Aucun nettoyage forcé.
- Worktree d’intégration208, worktrees de revue/qualification et leurs branches conservés : lsof annonce une observation incomplète (espaces de noms et montages non accessibles). Les tâches de contrôle sont clôturées ; ce fait ne remplace pas le contrôle des usages pour retirer les dossiers.
- Les miroirs gérésSYS/KS restent des checkouts détachés natifs ; aucune réécriture de leurHEAD pour les aligner artificiellement. Les sources sont poussées et les objets208 disponibles par préparation native.
- Copies de transfert privées retirables après vérification de l’installation ; aucune donnée métier touchée.

Correction technique : pas de diaporama d’interface requis. Cette livraison n’affirme pas que tout incident historique ou toute connexion fournisseur est résolu.
