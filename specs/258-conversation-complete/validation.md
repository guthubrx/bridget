# Validation 258 — en cours

État : In Progress. Aucun commit, publication ou activation à ce stade. T3 reste ouvert.

## Preuves source et tests

- T004/T005 : contrat fermé, CLI stdin bornée, preuve privée humaine liée à racine/projet/tâche/ronde/instance, reçus réservés avant lecture fournisseur. Envoi normal existant réutilisé ; aucun thread T3 créé. Tests de contrat/CLI et gardes.
- T006 : catalogue interrogé sur le wrapper cible, sélection fournisseur confirmée, défaut natif seulement. Contrôle sur exécution existante. Reçu durable sans redispatch après issue inconnue. Worker distant identifié et fraîcheur vérifiée avant effet.
- 12 tests spec258 PASS, dont commandes locales/distantes, mauvais wrapper, ancien wrapper sans trame inconnue, changement d’instance pendant E/S, ronde/racine/enveloppe, révocations, fin/libération/déconnexion, perte d’accusé, écriture contrôle interrompue. Journal lineage élargi : 77 tests PASS (94,02 s).
- 244 tests T3 ciblés PASS dans 10 fichiers (21,42 s), incluant vue normale, page Bridget complète, suivi manuel/auto, projection d’outils/lacunes, contrats, Reader, brouillons, sélecteurs et hook. Ne pas additionner aux anciens passages recouvrants.
- Contrôle Codex avec sous-processus fournisseur fictif : catalogue et réglage du prochain tour sur le même fil PASS. Aucun fournisseur payant ni mission métier relancé.
- Typecheck web/server : aucune erreur dans les dernières sorties ; suggestions historiques de marque de classes présentes. Lint ciblé : avertissement préexistant de regex dans contrat149, aucun nouvel échec à la lecture.

## Réutilisation et limites

ChatComposer, TraitsPicker, draft store, ThreadTimeline et contrôleur de suivi partagés. Deux adaptations portent le contrat métier et sa traduction de présentation ; pas de second éditeur. Petite table de reçus dans DelegationStore existant nécessaire pour mémoriser la réponse fermée, distincte des reçus Cancel/Send/Control. Aucune nouvelle dépendance.

Catalogue modèle/effort actuellement attesté pour Codex App Server. Les autres pilotes rendent une capacité absente explicite tout en conservant l’envoi/interruption selon leurs capacités réelles. Ancien wrapper : aucune nouvelle trame tant qu’il ne l’annonce pas ; le journal et les messages autorisés restent utilisables. Aucun effort ni modèle remplacé implicitement. Pièces jointes/contextes non transportables refusés avec brouillon conservé. Les commandes sur enfant terminé/libéré sont désactivées ; aucune continuation implicite.

## À terminer

Contrôles finaux, preuves UI, revue indépendante, Converge, audit V15, intégration/push des deux dépôts, publication et activation des services Bridget, paquet signé T3 préparé sans arrêt, nettoyage sûr.

## Preuves visibles

Diaporama autonome T3 : preuves-corrections/2026-11-10-conversation-sous-agents/preuves.html (454059octets, 14étapes, 11images, 7clicsréels). Curseur/cercle premièreétape inspectés visuellement ; navigation suivante/dernière/retourpremière vérifiée. Éditeur, catalogue/effortconfirmé, send/steer, texteaccepté, suivi/suspension/retour et copiecode(Copied) observés dansAppRootisolé. Les captures et manifestes originaux sont conservés horsGit. Avantnoncapturé ; aucun usageenproduction prétendu. Le premier générateur a refusé les images1280avecviewport1400 : circuitrefait en1280x800sans modifierlescoordonnées ni lescaptures.

Contre-revueClaude : failed/api_erroravantverdict. Reçuworkerconfirmecommande/instance/génération etfin du producteur. Stderr0octet, journalseulementapi_error : causeinconnue. Pas de remplacement, modèle ou compte changé. Relecturelocale indépendante=false.

## Contrôles complémentaires

Client humain : le délai historique6s ne couvrait pas le budget composé3+4s. Compose dispose maintenant d’une borne absolue9s sous celle du Reader10s. Testavec socketUnixréelle, réponse après6100ms : PASS, une requête seulement et capacité dédiée. Totalcourant spec25813PASS (6,12s), horsrégressionlineage77déjàvalide. ContrôleT3 du plafondpermissions/plan absent et modèle indisponible désactivé : deux parcourscompletsPASS. Hook : génération de lignée relue et capacités absentes bloquées avantdispatch.

BuildwebPASS42,58s ; bundleserveurPASS1444ms. Les contrats de tâche ne permettent que codex_app_server/claude_stream_json : hypothèse d’attribution ACP réfutée par le schéma, aucun support supplémentaire inventé. RégressionOrchestrator/ThreadLaunchService52PASS : les commandesordinaires restent interdites surlesprojections.

## Incident OAuth confirmé (FR13/T019)

Signalement utilisateur depuis le fil COPIL Thémys : missionda788123, tâche4efd664f, instance48cc0e14/g2221. Sa commande native et le reçu de fin sont corrélés ; stderrvide. Le seq3 du journal dit exactement Failed to authenticate: OAuth session expired and could not be refreshed, suivi de api_error/is_errortrue. Même diagnostic dans la revue258e9277ecd. Lecture CLIactuelle authstatusjson : loggedInfalse, authMethodnone, code1. Aucun login/logout ni nouveau tour lancé.

Correction dans les puits existants : diagnosticfermé api_error:oauth_session_expired, code brutapi_error conservé dans provider_result ; guidage commun au statut et aux deux notifications, sans donnée de compte. Exige diagnosticexact et terminalerreur ; pour un diagnosticassistant, exige aussi le marqueurAPI ou le codeSDK authentication_failed. Les anciensapi_error restent inconnus. Trois tests transportPASS (vrais sous-processus fictifs, journalréel), dont faussecitation ; test daemonduguidage/propriétaire/immuabilitéPASS. Régressioncomplète du parseurClaude en cours au dernier relevé.

## Convergence fonctionnelle

Premier passage : CONVERGED pour les13exigences fonctionnelles/contrats et leurs tâches existantes. Aucun ajout de tâche de code. tasks.md est resté octet pour octet identique pendant le contrôle (SHA256a65a59b9e65929706a682a4f956563b3f71bcb4d5691fd72c83cbdf8f48949ff avant/après). Les preuves de publication, activation, paquetT3 et nettoyage restent dans T017/T018 ; cette convergence ne les déclare pas réalisées. Le lot resteInProgress.

Le dernier passage T3 porte sur299tests dans12fichiers, tousPASS(27,25s). Les processus de contrôle de types web/server/mobile ont tous terminé code0 ; suggestions de marques de classes héritées conservées. Lintde touslesfichiersTSmodifiés : zéroerreur, unavertissement149préexistant de regex. BuildwebfinalPASS13,06s, bundleserveurPASS1444ms. Macrelease0.8.25/258-native-composer-r1 inclutOAuth, buildPASS2m31. Linuxqualifie actuellementle même patchdc3b9857… dansunworktreedeconstructionisolé.

## Qualification finale avant intégration

Linux : tests natifs258, transport258 et contratPASS ; releasePASS234,50s, même patchsource queMac. MacClaude45PASS, daemon25814PASS, contrat12PASS, lineage77PASS et contrôle12PASS ; suites recouvrantes non additionnées. Les deux auditsV15 sont pass/validationtrue, avec notes paraxe et limites dansaudit.md. Publication et activation restentT017. Cause de la contre-revue maintenant confirméeOAuth, et non inconnue : journalcorrélé seq3, aucun nouvel essai fournisseur.
