# Implémentation 172

Développement sur Cartae, branche session-172-resolution-projet-distant.
Source initiale : c48c30fbbfaa62562ce4a9ff03e31b92444fb533.
Appel refusé : opus2d-cartae-lecture-bridget-20261018-9ecd0abe-r1.
Fil T3 : e4999bf4-8f05-401b-bdc2-a9bb32b0b9f7, position 33665.

Hypothèse confirmée : aucune sonde distante lorsque project_id est absent,
car la boucle fait continue avant la lecture du mapping existant.
Correction : résolution unique depuis le registre, refus explicites,
identifiant conservé dans RoutingCandidateObservation puis Delegate.
Tests : en cours. Activation et recette réelle : non effectuées.

Validation : 2 nouveaux tests PASS, 35 tests spec168 PASS et 1 test incident171 PASS.
Premier lancement de régression : 26 PASS/9 FAIL avec le vieux Git système ;
reprise avec le Git 2.56.0 du worker : 35 PASS. Aucun changement de test pour cela.
Référence T3 complémentaire : OrchestratorMcpService.delegateTask rattache le parent ;
ThreadLaunchService prépare fetch/checkout/setup-script avant prepared-run.release.
Extension demandée ensuite implémentée dans project_preparation.rs et scripts/prepare-project.py.
Le helper utilise la fédération SSH validée (reçu et empreintes), retrouve le
registre du worker et prépare un dépôt Git standard avant la sonde existante.
Neuf tests Python utilisent de vrais dépôts et un transport SSH simulé, sans T3 :
nouveau projet non publié, rejeu, transfert incrémental, conservation du checkout,
interruption avant publication, protections et registre séparé du tunnel.
9 PASS après désactivation de la maintenance Git automatique (une première
exécution avait rencontré une course de nettoyage dans pack). 2 tests Rust172
et 35 régressions168 PASS. Pas de suite globale.
Sonde SSH réelle lecture seule de Revue : dépôt absent, needs_bundle=true.
Aucune correspondance Revue ni destination préparée manuellement.
Activation et recettes modèles restent à effectuer.


## Recettes du 18 octobre et correction des délais

Source 9c705d7a21e6b314f2c436485ab40ba10ed464e2 activée sur le maître Mac.
Le worker est encore en 171. T3 et son pont sont conservés.
Bridget connu : tâche 060d93d7-278c-4bcc-884b-003769c19a18, résultat reçu.
Revue neuf : tâche c0a1ea43-6dec-408e-b148-ca812ef752e9, dépôt et correspondance
créés automatiquement, lancement et résultat reçus. Aucun project_id fourni.
Les deux briefs demandaient par erreur un shell en posture discovery : le refus
d'outil est correct, ces résultats ne valident pas la lecture attendue du dépôt.
Deuxième recette Bridget : 4553683e-ff47-4935-9bfd-0d2759354bf2,
remote_start_failed, aucune mission émise, nettoyage terminé.
Deuxième recette Revue : permission_attestation_unavailable, aucune tâche.
Plusieurs appels MCP exposent attestation_transport_unavailable / HTTP 3 s.

Donnée réelle : le reçu worker de la deuxième recette Bridget passe de créé
à terminal en cinq secondes, sans permis ni fournisseur prêt. Le code réatteste
les droits T3 après Ready, mais le bootstrap expire après cinq secondes.
Référence relue : ThreadLaunchService.ts, préparation puis prepared-run.release.
Adaptation distante : fenêtre bornée de 60 s négociée ReleaseLeaseV2 ; ancien
worker conserve un permis de 5 s. Aucun relâchement des droits, de la révision,
de l'époque ou du contrôle du parent. Attestation HTTP : 10 s par requête ;
AutoDelegateV2 : budget client 600 s pour inclure la préparation du dépôt.
Les erreurs définitives connues de démarrage conservent maintenant leur motif.

Validation complémentaire : test HTTP réel avec réponse après 3,5 s PASS ;
3 tests spec172 PASS, 8 tests T3 MCP PASS, 2 tests récupération d'admission PASS.
Tests worker : première exécution parallèle 20 PASS / 1 verrou occupé.
Le test exact de reprise réussit seul, puis les 21 tests worker passent en série.
La cause de concurrence du verrou n'est pas corrigée ni déclarée résolue ici.
Protocole distant : 9 PASS. La nouvelle recette de bout en bout reste nécessaire.

Négociation ancienne/nouvelle capacité et refus des doublons : 1 PASS.
Le premier ajout au test réutilisait une connexion déjà authentifiée ;
la garde StaleConnection l'a refusée. Le scénario utilise désormais une
nouvelle connexion, comme lors d'un remplacement réel du worker.


## Enregistrement distant après libération
Bridget r3 réussit : 68f69ce3-62bb-4b17-8bbc-853399a6346f, SHA a53b05c2,
version 0.4.1, marqueur reçu ; rejeu strict même tâche et résultat, ACK effectué.
Le permis durable confirme 60 s, native_ready=true et nettoyage terminé.
Revue r2 ancien est clôturé au redémarrage (07:59:07), avant son rejeu.
Revue r3 : 9cbc539d-264f-4ac6-a753-bfb91553003e, permis 60 s effectivement
accordé, puis negotiation_failed. Le stderr privé contient exactement
Resource temporarily unavailable (os error 11). Le maître enregistre l'enfant
à 08:01:37, au moment où le wrapper quitte ; aucun manque de mémoire ou de PID.
La lecture de Registered dans connect_and_register_at n'accorde qu'une seconde.
Reproduction : réponse Registered retardée de 1,5 s -> même os error 11 à 1,01 s.
Correction : 10 s pour ce premier accusé, retour à 1 s après Registered.
Le test vérifie aussi le maintien de la réponse suivante dans le lecteur.
T3 n'a pas ce Register à travers notre tunnel SSH. Référence examinée :
ClaudeMcp.ts préparation bornée à 6 s, distincte de la boucle de session.
L'adaptation Bridget garde une attente initiale bornée sans modifier les droits.
Ce changement de wrapper est utilisé sur Cartae ; seule sa version sera
réactivée pour la recette, le maître a53b05c2 reste suffisant et inchangé.

Validation : 69 tests de reconnexion PASS, 1 ignoré ; reproduction RED puis GREEN.
