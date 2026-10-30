# Audit de reutilisation de l'existant — capacités complètes221

## Decision

Statut: PASS
Date: 2026-10-30
Feature dir: /home/moi/10.Scripts/64.bridget/.worktrees/221-capacites-completes/specs/221-capacites-completes

Conclusion courte:24items du plan audités. Aucun second orchestrateur, moteur d’admission, store de tâches ou serviceHTTP. L’édition de texte existe déjà côté humain et doit être réutilisée. Les surfaces nouvelles ont une responsabilité propre et les inconnues d’architecture reçoivent les décisionsD1–D10 ; leurs preuves de fonctionnement restent ouvertes. Ce PASSautorise tasks, pas l’implémentation sans contrats détaillés ni une déclaration d’égalité.

## Synthese

| Metrique | Valeur |
| --- | ---: |
| Items extraits du plan |24|
| Items audites |24|
| Réutilisations déjà prévues |19|
| Existants potentiellement pertinents |5|
| Duplications evidentes non arbitrées |0|
| Règles/mémoires applicables |7|
| Specs existantes applicables |24|

## Reutilisations correctement identifiees

| Item du plan | Existant réutilisé ou écart | Preuve | Commentaire |
| --- | --- | --- | --- |
| Observation de compte/modèle | réutiliser/étendre | crates/bridget-daemon/src/provider_catalogue220.rs:28 | Observation locale220, pas un second collecteur |
| Authentification isolée | réutiliser/étendre | crates/bridget-daemon/src/agent_profile.rs:9 | Profil isolé ; parcours humain nouveau, aucune copiecredentials |
| Préparation guidée/miseàjour | réutiliser/étendre | scripts/federate-ssh.sh:368 | Réutiliser install/update/federation, aucun service neuf |
| Versions chargées | réutiliser/étendre | crates/bridget-daemon/src/build_info.rs:2 | Distinguer disque/runtime avec observations existantes |
| Projet/prérequis | réutiliser/étendre | crates/bridget-daemon/src/project_preparation.rs:24 | Sondes closes depuis manifestbase, pas exécution script libre |
| Lecture de commit immuable | créer justifié | crates/bridget-daemon/src/remote_worker.rs:2720 | Vue exportée immuable ; shared_refresh avance HEAD et bloque les descendants, donc inadapté |
| Switch/fork/import/checkpoint | réutiliser/étendre | crates/bridget-transport/src/codex_app_server.rs:3186 | Forktransportexistant ; manifests/passation complémentaires, pas second protocole |
| Drainage | réutiliser/corriger | crates/bridget-daemon/src/managed_process.rs:90 | UID et récolte séparés, même subreaper ; zéro service neuf |
| Diagnostic commun | réutiliser/étendre | crates/bridget-daemon/src/guidance.rs:5 | Pas de catalogue refus parallèle |
| File éditable | réutiliser | crates/bridget-daemon/src/daemon/mobile_queue.rs:112 | Édition déjà présente ; pas de varianteQueueEditdupliquée |
| Questions | réutiliser | crates/bridget-daemon/src/user_input.rs:34 | Questions181etdécisionshuman_inbox, aucun nouveau store |
| Secret éphémère | créer justifié | crates/bridget-daemon/src/user_input.rs:512 | Questiongénérique interditsecrets ; canalvaleurdistinct nécessaire, métadonnéesréutilisées |
| Artefacts | réutiliser/étendre | crates/bridget-daemon/src/artifact_service.rs:1 | Même blobstore/droits/isolation pour nouveauxclients |
| Client web humain | créer client / réutiliser service | crates/bridget-daemon/src/mobile/http.rs:290 | Pas d’interfacewebproduit existante ; même passerelle/auth/store/HTTPloopback |
| Navigateur | créer justifié | crates/bridget-transport/src/pty.rs:35 | PTYn’estpasCDP ; contrôlehumain/session/arrêt requis, nouveau moduleworkermais pas daemon |
| Appareils | créer justifié | crates/bridget-daemon/src/mobile/store.rs:16 | Appareilappariéhumain≠simulateurpiloté ; utiliser outils officiels et possessionworker |
| PR/diff/abonnement | réutiliser/étendre | crates/bridget-daemon/src/wake_schedules.rs:13 | Sourcesévénementsdifférentes, mêmes occurrences et reçus ; Git pourdiff |
| Terminal | réutiliser/étendre | crates/bridget-transport/src/pty.rs:35 | PTYpossédé réutilisé ; droit humain/sessionprojet à étendre, pas shell viaMCPlecture |
| Organisation | réutiliser/étendre | crates/bridget-daemon/src/threads.rs:97 | Même fils/annuaire/execution_store ; recherche/pin/archive sans secondregistre |
| Webhook/tâches | réutiliser/étendre | crates/bridget-daemon/src/wake_schedules.rs:13 | Route signée dansHTTPdéjàprésent ; aucune écoute supplémentaire |
| Usage/coût | réutiliser/étendre | crates/bridget-daemon/src/execution_store.rs:168 | Événementsusage/quotas existent ; estimation et présentation seulement nouvelles |
| Mobile création/horsligne | réutiliser/étendre | crates/bridget-daemon/src/daemon/mobile_operations.rs:15 | Même admission/receipts, nouvelleorigine humaine explicite |
| Moteurs | réutiliser/étendre | crates/bridget-transport/src/managed_session.rs:284 | ACP/portsnatifs et observationversionnée ; pas frameworkautonome |
| Windows | réutiliser le client web commun | crates/bridget-daemon/src/mobile/http.rs:350 | Choix utilisateur : interface versMac/Linux, aucun port du moteurUnix |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
| --- | --- | --- | --- |
| Édition de file | remplacement humain existant | daemon/mobile_queue.rs:112 ; mobile_operations.rs:1939 | Réutilisation décidée dansD7, aucun autre moteur d’édition |
| Fork | bootstrapCodexexistant | codex_app_server.rs:3186,3259 | Étendre parcours/receipts, conserver le transport |
| Secret | US8prévue, pas livrée ; questionsrefusentsecret | specs/158-reveils-controle/spec.md:181 ; user_input.rs:512 | Canalvaleurdistinct ; pas recréation d’une questionsécrète générale |

## Duplications evidentes

Aucune duplication de code créée. La proposition initiale d’étendre QueueEditAction pour éditer un texte est retirée au profit du puits humain existant. Les nouveaux clients utilisent la même passerelle ; les nouveaux déclencheurs utilisent wake/idempotency.

## Memoires et regles applicables

| Source | Règle | Impact |
| --- | --- | --- |
| AGENTS.md | corriger et activer les incidents | gateparunité cohérente, pas d’accumulation binaireinactive |
| AGENTS.md | préserver les métiers | fixturesprivées, aucune mission ancienne relancée |
| ConstitutionXVI | worktreeisolé |221surCartae,maininitialpropre |
| ConstitutionXIX/XX | réutilisation/minimalisme | pas de wrapperpasserelle ni storeparallèle |
| ConstitutionXXII | mesureavantgain | acceptance50cas, NONÉTABLI si référenceabsente |
| my-specify-all | pasMVPfinal | toutes lescatégories et tâches restent visibles jusqu’à100% |
| .specify/memory et standards | primitives/contrats | protocoles des skillslus ; scriptsabsents remplacésmanuellement |

## Specs livrees applicables

153/158/159/168/172/174/177/180/181/183/187/194/198/201/207/208/209/211/214/215/216/217/219/220. Réemployer respectivement cycle/gardes/réveils/file/routage/SHA/admission/artefacts/édition/questions/mobile/rondes/rejeux/fence/reconnexion/compact/droits/support/annuaire/convertisseur/Node/modèle/inventaire/catalogue. Les labels historiques de statut ne sont pas des preuves fraîches.

## Journal de recherche

| Requête | Portée | Résultat |
| --- | --- | --- |
| noms et responsabilités des24items duplan | crates/scripts/docs/specs | ancrages listés, fichiersrelus ciblés |
| secret/SecretRef/US8/isSecret | user_input/human_inbox/mobile/specs158 | secretsnonimplémentés, questionsdenied |
| Fork/Resume/CodexThreadBootstrap | managed_session/codex_app_server | transportForkexistant |
| QueueEditAction/MessageQueueChange/replace | protocol/mobile_queue/mobile_operations | éditionhumaineexistante, test180 |
| route/auth/Router | mobile/http.rs | HTTPloopbacketBearer, extensionnavigateur àprotégerCSRF |
| browser/device/terminal/PTY | cratesetdocs | PTYexistant ; pas de brokerCDP/simulateurnatif trouvé |
| manifests | Cargo.toml et Cargo.lock | axum/tokio/tungstenite/ring/minreqdéjà présents ; aucune dépendanceajoutée |

## Arbitrages

| Sujet | Décision | Justification | Date |
| --- | --- | --- | --- |
| File éditable | réutiliser | même responsabilité déjà implémentée ; D7 |2026-10-30|
| Sessionsoutils | créer sous contrôleworker | responsabilitépossession/bornes/sécurité absente du PTYsimple ; D3 |2026-10-30|
| Secret | créer seulement canalvaleur | questionspersistantesrefusentsecrets ; D2 ; métadonnéesréutilisées |2026-10-30|
| Vueimmutable | créer dansworkspaceexistant | shared_refresh modifieHEAD, donc ne couvre pas le résultatlecture ; D1 |2026-10-30|
| ClientwebetWindows | réutiliserautorités/extraireOSnécessaire | exigencesplateformes explicites et importsUnixinconditionnels ; D4 |2026-10-30|

## Gate avant tasks

- [x] Aucune duplication évidente non arbitrée.
- [x] Chaque item extrait du plan a une ligne d’audit.
- [x] Les règles projet applicables ont été lues.
- [x] Les specs existantes proches ont été vérifiées sur les responsabilités concernées.
- [x] Le plan précise la réutilisation et justifie les créations nécessaires.

La contre-revue initiale BLOCKEDa servi à corriger le plan ; ses objections retenues/réfutées sont documentées. Les qualifications absentes restent des tâches, pas une exception cachée aux critères.

## Arbitrage local C25 — diagnostic après attente bornée

RÉUTILISER SessionJournal::new/append et le journal distant existant, après la destruction de ses writers. Un retour forcé de Drop pourrait orpheliner un descendant vivant et perdre sa filiation ; il est rejeté. Après5s, publier une seule erreur fermée/guidée dans le journal natif puis garder le superviseur vivant pour récolter ses propres enfants. Ce n’est pas une promesse de Dropborné : c’est un résultat d’attente borné et une supervision résiduelle explicite. Le seul helper de projection porte cette règle de sécurité/lifecycle, aucun nouveau store/protocole. La recette de non-fuite et l’absence de preuve d’arrêt anticipée sont nécessaires avant gateL1a.

## Extension C25 avant barrière

Le test réel avant barrière échoue sur le candidatpost-barrière : aucun événement pendant15s, malgré writer actif ; journalrawvide et propriétaireChildencore détenu. REUTILISER JournalWriter::enqueue, pas SessionJournalpendant un writeractif. Étendre ManagedSessionavec un getter de writer facultatif (défautNone), implémenté par les deux adaptateurs natifs déjà détenteurs de ce writer. Il porte une règle d’observabilité sûre et évite un second écrivain ; aucune autoritépublique nouvelle. Le signaler publie un diagnostic par phase via le writer actif ; après barrière, SessionJournalreste le seulfallback. Aucun nouveau store, dépendance ou canal réseau.

## Précisions de réutilisation après la ronde2

- NavigateurC33 : PtyTransport::from_master envoie sur un terminal possédé ; il ne fournit ni DOM/snapshots/CDP, ni contrôle humain de navigation. Le module nouveau porte la propriété exclusive et les limites de session navigateur. Aucun autre shell d’agent recréé.
- AppareilsC34 : mobile/store.rs gère les téléphones humains appariés et leurs autorisations ; ce ne sont pas les simulateurs pilotés sur un worker. L’inventaire/outillage officiel reste derrière le worker existant, aucun second dispositif d’authentification.
- TerminalC37 : réutiliser PTYetattach pour flux/backpressure ; une session shell du projet possède une identité et des droits distincts du dialogue attach avec un agent. C’est cet écart d’autorité qui exige un port humain contrôlé.
- PR/webhook : store/wake_schedules.rs conserve payloadTEXTetles occurrences, FrozenWakeporte une source/version et un canon. Étendre le décodage pour une version2fermée, sans réécriture des canonsv1 ; mêmes transactions/reçus/occurrences, pas de table de tâchesPR. Le testv1/v2requis doit prouver lecture/rejeu/quotas avant toute source nouvelle. Pas de stockage d’une signature comme corps de commande d’autorité.
- MobileC44 : MobileOperation inclut liste/show/cancel mais aucun TaskCreate ; ajouter une origine humaine attestée au moteur d’admission existant, pas une identitéagent forgée. Les autorisations d’appareil existantes ne reçoivent pas ce nouveau droit sans accord humain explicite.
- HorsligneC46 : MessageQueue et testsAndroid QueueStore/ProcessRecreation ainsi que mobile180couvrent déjà une partie des cas. Réutiliser, tester puis n’ajouter que le parcourswebmanquant ; pas de second journal natif de remises.

Convergence C25/T176 : session du journal différente de l’instance wrapper. Relecture des activate_journal des adaptateurs et SessionJournal::new : Codex écrit thread_id, Claude écrit connection_id. Réutiliser le getter ajouté pour transporter cette identité avec le writer ; aucun second registre ni règle devinée depuis le nom de fournisseur dans le wrapper. Recette RED effective avec identités distinctes conservée.
