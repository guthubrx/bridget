# Validation250 — résultat natif et refus manuel

## Incident exact

Signalement de l’enfantN2 ab18b152, envoi manuel mcp-3114084-6af0ab47-4 à1794157383 en réponse à7f6ceb97. La clé est Prepared, sanssend_delivery. Le maître a journalisé remote_result_requires_worker à17:03:03 et17:03:24UTC. Le Nack produit était hors des réponses attendues par SendIdempotent ; la façade affichait outcome_unknown/accusé perdu. Le rejeu exact rencontrait le même refus. Aucune cause réseau ou disque n’en est déduite.

Un circuit distinct a ensuite fonctionné : tâche d467f589-d7cd-4977-aae9-591a66a786b8 result_available à1794157448, résultat durable4017octets ; demande d’origine answered à1794157447. ReçuWorkerResult cartae/instanceec376610/génération2079 accepté sans conflit, result_sent=true. Ni le contenu métier du résultat ni le fichier de production ne sont copiés dans ce rapport. Le parent a reçu les identifiants pour lire sa tâche existante. Aucun envoi métier, rejeu ou relance par le mainteneur.

## Correction

Dans le seul chemin SendIdempotent, remote_result_requires_worker est rendu par reject_idempotent_send : refus durable corrélé IdempotencyResult::Rejected. Le prochain rejeu exact d’une ancienne réservationPrepared retrouve le refus, sans capture et sans nouvelle remise. Les erreurs techniques inconnues restent inchangées. Le contrat WorkerResult n’est pas assoupli et le résultat canonique n’est pas modifié.

Le guidage demande de donner la réponse finale normale, afin que le worker conserve et publie le résultat ; il indique au parent de lire le même task_id. Le refus porte sur cet envoi manuel, pas sur la mission ni les autres travaux.

## Vérifications

Le test250 couvre clé neuve et réservationPrepared historique, chaque cas rejoué deux fois. Réponses identiques, aucun contrôle de remise émis, aucunsend_delivery, Task inchangée et demande toujours ouverte. Les contrôles de publication native et de refus legacy sont réutilisés. Trois contrôles ciblés passent surMac etLinux, dans une seule invocation finale par plateforme :spec250, publication canonique162 et double voie idempotente/legacy162. Le premier contrôle historique attendait encoreNack sur la voie send_as désormais idempotente ; son assertion a été alignée, tandis que Send legacy garde son Nack. Ce seul delta est un test, commita3dbea6b74747fba13d47ef01a88b41b88eeaa2d. Le code exécutable reste celui du candidat27b402ebd03a813ef0bd1378d4f468820d6833d9.

Revue3880e68c terminée, lue etACK, aucunP1/P2 concret trouvé. Revue statique sans test/build indépendant. Aucun nouveau modèleClaude ni campagne générale. Aucune nouvelle revue requise pour la seule assertion de test.

Les diagnostics au parent et àl’enfant ont des reçus conservés. Ils indiquent le résultat natif disponible et ne demandent ni rejeu ni nouvelle mission. Le bilan d’activation et de publication est consigné après vérification.

T3 ne doit pas être arrêté. Preuves privées : `/Users/moi/.cache/bridget250-resultat-natif/`.

## Activation et preuve en fonctionnement

Source finale a3dbea6b74747fba13d47ef01a88b41b88eeaa2d, version0.8.16/250-resultat-natif-r1. Sept composants chargés sur quatre hôtes et leursSHA vérifiés :Mac8c4f7ffd202c3047b5b7f127f6b6dbc2e4c5078315c08573e711f1b5f6b42906 ; Linux23adedb2f378016930f9af20a5fe7e6f4cf7fefb773ef60b53c1e1f82c5e70d4. Coupure du maître1,80s. Retour249-r1 compatible et qualifié dans les deux sens ; pointeurs atomiques des quatre hôtes vérifiés, ancien245 refusé comme lecteur.

La façade MCP existante a servi un test délibéré sur notre propre revue distante terminée3880e68c, sans utiliser une missionParix : identifiant bridget250-validation-refus-20261108-01, issued_at1794158574. Réponse effectivement reçue status=remote_result_requires_worker, avec le texte de résolution. En base, résultat terminal/rejected, aucune send_delivery, propre revue toujours ronde1/result_available. Ce contrôle ne crée aucun tour, aucune ronde ni résultat. Il n’a pas renvoyé le messageN2.

Le résultatN2 demeure result_available/4017octets/result_sent=true après passage. Son ancienne clé manuelle reste conservée sans réécriture forcée ; le correctif sait clôturer une réservationPrepared au prochain rejeu strict, mais aucun rejeu de cette clé n’a été fait par le mainteneur. Le résultat natif ne dépend pas de cette clôture.

Tous les wrappers actifsCartae ont conservé leurs PID et dates de naissance pendant le redémarrage du maître puis du worker :3162719,2995359 et3113948. Les quatre processus principauxT3 ont conservé PID et date de naissance ; le renderer2669 a disparu entre les observations, cause non attribuée. Aucune commande de maintenance n’a arrêté l’applicationT3. RegistreMac et septpayloads protégés strictement inchangés. Session nativeV2 rétablie. Les anciennes façades ouvertes et les images encore utilisées restent conservées.

## Publication, avis et nettoyage

Release publiquev0.8.16, six actifs téléchargés anonymement/SHA et tailles vérifiés. APK8016, certificat0c43c621254d9a6e0dbe148466f453f1aaba1a62bf23bd7332c61086f16a6588 et lignée8015 vérifiés. Le delta final n’est qu’une assertion de testRust ; aucune seconde compilationAndroid nécessaire.

Diagnostic puis bilan final envoyés au parent et àl’enfantN2 vivants, avec identifiants et reçus conservés. Une remise en vol ne prouve pas lecture du message. Aucune réponse d’accusé exigée et aucun rappel périodique. Aucun fichier ou commitParix modifié, aucune mission ou changement de moteur déclenché.

Source finale intégrée/poussée et alignée sur quatre hôtes avant nettoyage. Propres worktreesMac250, compilationSYS250 et AndroidCartae250 retirés après intégration et absence d’usage ; branche250Mac supprimée, main seul. Les870fichiers de buildAndroid identifiés sont retirés avec leur worktree. Copies de transfert et bundles propres retirés ; détails dans cleanup-transfers.json privé. Caches partagés sous budget :7,65GioMac et3,14GioSYS au dernier passageCargo. Actif250 et retour249 conservés ; travaux, vues de revue, reçus et images étrangères utilisées ou non observables ne sont pas supprimés. Le bilan documentaire final est poussé et aligné àmain sur les quatre hôtes en fin de livraison.
