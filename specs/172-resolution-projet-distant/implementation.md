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
La préparation automatique des nouveaux projets demandée ensuite reste à réaliser.
