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
| Webhook/tâches | réutiliser/étendre | crates/bridget-daemon/src/wake_schedules.rs:13 | Route signée à ajouter dans le service HTTP existant ; aucune écoute supplémentaire. Aucune route webhook actuelle n'est attestée. |
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

## Réemploi C26 après tests
RÉUTILISER idempotency et DelegationStore : recherches reserve/replay/envelope_mismatch/admit_auto_task/transaction puis lecture des handlers et tests. Les92tests distincts passent, aucun écart justifiant du code neuf. Pas d'AutoAdmissionStore parallèle. Le contrat de perte de réponse est précisé dans contracts/common.md, les preuves dans validation/c26.md.

## Gate C27 — projection des refus de sonde
RÉUTILISER project_probe_constraints, failure, routing_probe_failure et routing_probe_error dans daemon/native_delegation.rs. Recherches capability_unavailable/probe/reasons/refusal et lecture du parseur fermé existant. guidance.rs porte les interventions interactives156, pas une responsabilité de catalogue d'erreurs. Aucun nouveau framework, store, enum partagé ou protocole prévu pour cette correction de projection. La cause précise d'un CapabilityUnavailable ancien reste inconnue ; les contrôles manquants doivent être explicités, pas déduits du code grossier.

## Clôture source C27
REUTILISER confirmé : project_probe_constraints/failure et les deux ports initial/relecture.35motifs fermés sans champ libre ; BTreeSetstandard pour déduplication, pas de dépendance ou format de reçu nouveau. Septtests spécifiques et14régressions C27 passent ; deux194 déjà comptés enC26. Aucun item logiciel partagé neuf. Les nouveaux fichiers validation/c26.md et c27.md sont des preuves, pas des composants d'exécution. Audit/source et activation restent distincts ; trois revues contradictoires documentées.

## Compléments T177/T178 — recherche avant code
WorkspaceRequest, les deux parseurs MCP, le schéma partagé, InvalidParams et son mapping -32602 sont réutilisés. Un parseur commun porte la même règle de forme pour deux appels réels ; aucun changement du type ou du protocole. La sélection registry applique déjà activation, politique et cinq filtres : seul son résultat vide reçoit un suffixe fermé dans le builder existant. failure projette ce suffixe sans parser de texte libre. Aucun nouveau module, service, store, dépendance ou effet. Les refus après sonde ne reçoivent pas ce diagnostic local.


## Gate C16 — 30octobre, après fixture A/B
RÉUTILISER les mappings à clés distinctes, AgentRegistryFile racine tolérante, routing_project_mapping, select_mapping et le champ project_id des opérations de préparation existantes. Pas de sidecar, historique de permissions, nouveau store, commande ou endpoint. La section racine opaque optionnelle ne sert qu'à choisir une future admission ; sa validation tardive ne prive pas A de ses droits. Test A+B : A conserve rapport/permission/recheck et réservation, B n'est encore qu'une déclaration. Mutation brute A→B : refus avant Start/Release et nouveau slot. Les propositions de conception sont des hypothèses ; le sidecar fondé sur un registre globalement fermé est rejeté, la racine est tolérante. Publication physique et configuration existantes conservées ; aucune nouvelle autorité humaine inventée.


Gate C16 complément corrélation : recherche observe_routing_candidates et auto_admission_id avant écriture. La variante avec clone changeait l'identifiant technique de sonde malgré le reçu inchangé (RED194atteint). Réutiliser la même implémentation d'observation avec une clé de projet séparée du canon. Une façade interne conserve les appelants initiaux ; le coeur est unique, sans store/type/protocole neuf. Cette séparation est nécessaire pour maintenir la corrélation du canon et épingler le mapping déjà observé ; pas un helper décoratif.

### C02–C05 — complément de présentation des faits220
Revue2ceacc8c lue. Réutiliser CachedCatalogue, project/ingest220 et les trois
ports de lecture actuels. received_at est le fait de réception manquant ; le
reste est dérivé au moment de lire. Aucun modèle ni compte n'est ajouté ou
réactivé. Un index de modèles déclarés par connexion remplace le parcours
répété ; complexité O(d log d + m log d), d/m bornés256. Aucun magasin nouveau.
C06 : refuser les homes partagées casserait les profils normal/Terra existants
sans besoin prouvé ; décision retenue : ne pas introduire ce refus.
La découverte Claude n'est pas déclarée couverte par l'observation Codex.

Relecture C03 : l'hypothèse d'un manque vient de la lecture d'une ligne détachée
de son objet connexion. La forme existante relie déjà modèle/compte/serveur.
Dupliquer machine_id/connection_id par modèle pourrait amplifier128connexions
x256modèles au-delà de la trame16Mio. Ne pas le faire ; tester les deux contextes
complets. La projection gardera le lien existant et son ancien format imbriqué.



## C19 — contrôle des lecteurs
RÉUTILISER les ports candidats, le validateur installé et la borne de commande de209. Ne pas étendre son Report à deny_unknown_fields. Le module storage_compatibility porte uniquement le contrat de formats, lié à un nonce et au SHA du candidat, et le plancher du lecteur installé. Aucun store ni scan global. Le plancher conservateur couvre les formats écrivables, y compris en racine custom ; aucun inventaire absent déduit vide. Le passage initial requiert un lecteur de vue et un retour qualifiés avant activation des écritures. Aucune nouvelle garantie de lecteur avant les types et tests correspondants.

### C19 : preuves privées de l'export (poursuite du noyau)
Les helpers privés du worker écrivent des reçus de 64 Kio ; ils ne peuvent pas
porter le manifeste complet borné à 8 Mio. Le même inventaire de WorkerStore
compte déjà tous les fichiers privés, y compris orphelins. Réutiliser ce budget
et son verrou de transition ; ne pas créer un second quota ou un second store.
Le module workspace_commit_view conserve seulement les règles de l'export :
intention privée fermée, publication exclusive, lecture bornée et audit exact.
Ses descripteurs openat/nofollow sont réutilisés pour les fichiers de preuve.
Le raccordement futur doit réserver les octets avant toute écriture et rattacher
ces fichiers à WorkspacePreparationReceipt. Aucun appelant natif à ce stade.

### C19 : processus d'export
Réemploi de ManagedLaunch, ManagedStatusReporter, du bootstrap et du marqueur
durci avant RELEASE. Les enfants Git restent dans le groupe du helper ; aucun
nouveau service, ordonnanceur ou transport. workspace_commit_view_job sépare
seulement la possession du sous-processus et le protocole privé de fin de
l'algorithme d'export. Le canal de statut existant porte un code fermé.
Un verrou de fichier privé par tuple empêche deux lancements concurrents.
Le raccordement au budget, à Cancel et à la reprise du worker reste requis.


### C19 : relecture d'une vue conservée
REUTILISER WorkerReadView::recheck_at_with_workspace, le reçu existant et
WorkspaceRecord.commit_view ; étendre leur branche de rapport, sans second
contrôleur de droits. La fonction remap_prepared_worktree suppose un dossier
sous l'ancienne racine Git et ne peut servir telle quelle pour un export
extérieur sans .git. CREER la dérivation pure ViewIntent::execution_paths :
elle impose la cohérence des décalages source/mapping/manifeste avant de
remplacer la racine, une règle de sécurité spécifique. La relecture native
et ses contre-exemples de sous-dossiers utilisent la même dérivation.
Aucun nouveau store, outil public ou autorité de lancement.


### C19 : fichier omis sous processus possédé
REUTILISER ViewJob et materialize, après recherche des lecteurs de blobs,
ManagedLaunch, stop_group et des marqueurs. Étendre le même contrôleur avec
une requête fermée Export/Materialize ; pas de second gestionnaire de
sous-processus ni d'exécuteur de commandes. Le tuple commande complète,
séquence, chemin et binding impose la corrélation avant RELEASE.
Les deux entrées CLI internes séparent leurs formes admises et exigent le
canal géré avant toute lecture de chemin. Pas de nouvel outil MCP à ce stade.


### C19 : propriétaire runtime de la matérialisation
REUTILISER la collection ViewSourceProcess et le même ViewJob pour l'export
et les fichiers omis. Ajouter l'identité de résultat/requête au propriétaire,
pas une collection ou Fleet de plus. RÉUTILISER le corps existant de
inspect_candidate comme lecture de définition/droits, après sa garde
d'admission inchangée : le rappel avant RELEASE doit relire ces faits pour
l'opération enregistrée. Le corps partagé n'admet rien et ne crée aucun token.
Les compteurs de signal, la récolte et Cancel gardent les groupes de fichiers.

### C19 — libération du verrou local de vue
Constat réel : export_busy au démarrage de la matérialisation après un export
terminé, reproduit pendant la suite Mac r14 ; test ciblé Linux avec dup rouge.
Recherche : lease C19 est le seul flock de ce tuple ; ManagedMarkerStore garde
séparément le processus non arrêté. Réutiliser ce même fichier/verrou, sans
second créneau, attente, retry ou serveur. ViewLease ajoute seulement LOCK_UN
au Drop du propriétaire : la fermeture d'un FD seul ne libère pas flock tant
qu'une description dupliquée subsiste (cas fork avant exec). CLOEXEC conservé.
Le marqueur non soldé reste bloquant, même après libération du verrou local.
Pas de nouvelle dépendance. Garde minimale justifiée par ce cas de possession.

### C19 — lecture en cours sans attente dans le worker
Recherche ciblée : runtime.materialize_view_file attendait en boucle la fin
du helper pendant jusqu'à son échéance. Réutiliser la même collection de
processus et le même reçu de matérialisation, sans thread, queue, ordonnanceur
ni store neufs. begin_view_file rend un jeton privé sans manifeste ;
poll_view_file fait un seul pas. Seul l'adaptateur de test attend en boucle.
Le collecteur existant observe l'annulation et le slot de mission même si le
lecteur ne revient pas. Abort garde une cause de preuve indisponible distincte
de Cancel et n'envoie pas un nouveau signal à chaque tick. Les audits finaux
restent bornés et synchrones : le raccordement devra encore mesurer leur coût.

### C19 — lancement avant READY et preuves pour le retrait
RED réel : c19-unready-bootstrap-red.log, Child possédé encore vivant,
confirm_view_sources_stopped=Ok(true) sans marqueur. Réutiliser le même reçu :
un bit de tentative durable avant fork, puis le marqueur et l'arrêt existants.
Pas de PID supplémentaire, moniteur, retry ou table. Le contrôleur courant
durcit un arrêt après observation de son Child ; un lecteur retrouvé reste
conservateur. Nouveau raccourci de spawn seulement en cfg(test).
check_view_native_closure réutilise inventory, drainage_proven, le fence198 et
les dispositions162. Le pin de collecte protège ces preuves jusqu'au futur
tombstone ; ne pas produire une réussite à partir de leur disparition.
Autre RED réel : c19-settled-pid-reuse-red.log. Une source déjà arrêtée ne doit
pas reprendre le groupe numérique d'un autre processus pour sa propre activité.
La reprise conserve le stop durci ; aucun signal au processus de fixture.
Ces éléments restent privés et sans admission C19 activée.

### C19 — conservation sans privilège et référence de manifeste
Choix utilisateur confirmé : conserver sur inventaire incomplet, pas de contrôle
administratif du worker. Réutiliser la carte exportée, la publication privée,
SHA256 et le reçu existants. Le grand manifeste est un document de données ;
la petite référence privée en garde l'autorité par son empreinte canonique.
Figer les écritures avant réduction ; pas d'archive opaque, second store,
compression/dépendance nouvelle, élargissement de quota ou droit ajouté.
Le compteur conserve les compagnons et orphelins réellement présents.
La référence ne donne jamais autorité au contenu modifiable de la carte.
L'implémentation et les tests de tous les états restent requis.

### C19 — boucle source non bloquante
Réutiliser begin/poll du port de fichiers, le même ViewJob, le même BTreeMap
view_sources et le reçu de préparation. ViewSourceRead conserve l'identité
minimale ; source_failure préserve uniquement la cause fermée de fin du helper.
Pas de second exécutant, file ou état de réussite fictif. Les états de mission
restent ceux du runtime. Préparation publique toujours fermée avant ses gates.

### C19 — reprise native198
Réutiliser original_workspace_at, BindProof198 et le reçu Released/slot exact.
Aucun registre d'alias ajouté. Le contrôle d'ancre est partagé par début/fin de
matérialisation ; il porte une règle d'autorité, pas un passthrough. Le helper
n'accepte pas une Task différente. La chaîne privée, pas la valeur command_id
seule, autorise une autre commande. Cancel doit garder l'identité de cette
opération afin qu'une reprise ne soit pas protégée par l'ancien stop_once.


### C19 — demande native de conservation
Réutiliser ViewBinding, le reçu WorkspacePreparationReceipt, check_view_native_closure et compact_closed_view. Le protocole existant n'a aucune opération de clôture d'espace : Cancel ne vise qu'une commande/slot et Reconcile est une lecture, donc aucun ne peut porter cette nouvelle décision propriétaire sans ambiguïté. Ajouter ConserveCommitView et son accusé fermés, empreinte seule ; WorkspaceViewV1 négociée avant émission. Pas de store, worker ou ordre de signal neuf. Ancres privées et preuves finales restent les seules autorités ; l'ACK de données ne crée pas de clôture. La garde publique de création reste fermée.


### C19 r55 — réservation et audit
Réutilisations : WorkspacePreparationReceipt, ManagedLaunch/ViewJob,
WorkerRuntime et WorkerStore.inventory. Aucun store de tâches, autorité,
dépendance ou pool nouveau. Le thread lit seulement les fichiers ; il ne
reçoit ni WorkerStore, ni commande de mutation, ni programme fournisseur.
Le worker effectue seul le CAS final. Il garde la lecture après Cancel
jusqu'à sa fin. Cela sort l'audit de 512 Mio de la boucle des commandes.
Les rapports publics restent entièrement audités. Le même UID peut modifier
des fichiers : aucune immutabilité OS n'est promise.
La conservation non publiée réutilise le port natif ; elle ne permet aucune
suppression physique. Réparation sans tentative : même graine, canon et délai.

### C19 — audit avant Start et Release
REUTILISER WorkerReadView::recheck_at_with_workspace pour toutes les autorités,
la définition et les droits ; ajouter seulement un rapport de vue obtenu par
un lecteur privé possédé au lieu de refaire sa lecture lourde dans le runtime.
Le rapport est lié à l'ancre complète et consommé pour un seul contrôle.
Aucun cache réutilisable de permissions, aucun store écrivain dans le lecteur.
Le contexte de préparation ne convient pas : ces contrôles portent une identité
d'exécution active et, pour Release, un permis et une échéance déjà figés.
Leur contexte borné réutilise l'identité/corrélation existante, sans opération de
protocole nouvelle ni autre ordonnanceur. Une annulation attend la récolte du
lecteur avant libération de son créneau, mais ne publie ni ne libère de bootstrap.


### Relecture C20–C50 — 31 octobre
La contre-revue d6c7e06b est une lecture partielle des sources et artefacts, sans
test ni shell. Sa contradiction sur la route webhook a été vérifiée : aucun
récepteur actuel n'est attesté, la ligne d'audit qui le présentait comme existant
est corrigée. Réutiliser le serveur HTTP et les magasins wake/idempotency reste
la décision ; la source signée est à créer selon le contrat avant ses tâches.
Les autres arbitrages signalés comme absents restent à rapprocher de D1–D11 :
leur absence ne peut pas être conclue sans lire ces décisions. Aucune catégorie
supprimée, aucun résultat de parité déduit d'une recherche de symbole négative.

### C19 — fichier omis vers le client
REUTILISER les méthodes privées begin_view_file/poll_view_file, NativeDelegationRequest,
can_own_task, live_connection_identity, remote_command_identity, les slots
d'exécution et la façade delegation_mcp. linked_task de native_child_requests
n'est pas adapté : il exige child_requests=forward et Codex, alors que cette
lecture doit fonctionner aussi avec Claude et child_requests=deny. Ne pas
modifier cette autorité155. La nouvelle opération transporte seulement Task
facultative et chemin relatif ; maître et worker dérivent les preuves exactes.
Ne pas détourner read_artifact : ses références portent des artefacts publiés,
pas la copie privée d'un blob dans une vue. Aucun second magasin de fichiers.

### C19 — audit final d'une matérialisation
La lecture de finish_materialization révèle un file_available/recover complet
dans le pas du runtime, même hors verrou. Réutiliser ce CAS en séparant arrêt
durci, lecture possédée et publication. Le thread ne reçoit que ViewIntent,
chemin et racine de preuves, jamais WorkerStore. Le collecteur et le poll
réutilisent ce même lecteur ; pas de deuxième audit parallèle par opération.
Le wrapper synchrone reste un adaptateur privé de tests/erreurs sans audit.

### Raccord lecture de fichier — R103/R107
REUTILISER la forme native commit_file_refusal, WorkerRefusal et view_failure_action. La table commit_file_worker_refusal adapte uniquement les codes fermés au contexte lecture : project_probe_constraints est lié à une admission et changerait sa portée, son canon et son guidage. Deux usages réels : refus du roundtrip et réponse fichier sans sous-cause observée. Aucun nouveau store, service, type public ou droit. Les métadonnées d'exécution restent dérivées du maître, jamais du chemin demandé. La factorisation commune des trois pending est écartée : autorités, deadlines et propriétaires différents ; aucune divergence observée à corriger par une abstraction nouvelle.

### Compatibilité depuis l'installateur autonome
REUTILISER formats report et formats validate-report : l'installateur ne maintient aucune liste de formats ni de minimum. La forme autonome shell ne peut importer scripts/update.py du dépôt ; elle appelle les mêmes ports natifs fermés et lie nonce/image/contrôleur installé, avec sortie fichier bornée et hash après contrôle. Contrôle après préparation de service en mémoire, avant tout programme fournisseur, profil ou remplacement du binaire. Les ports natifs de hash restent bornés à10s ; ce changement n'ajoute pas un processus fournisseur ou un service. Le paquet historique0.6.3 sans port n'est conservé qu'en première installation, jamais au-dessus d'un contrôleur courant.

### Constat de fichier terminé, publication indisponible
CREER seulement ViewFileAuditState, état privé du propriétaire déjà existant ViewFileAudit : Reading conserve son JoinHandle, Ready conserve le résultat et l'intention jusqu'au CAS. La recherche de ViewFileAudit/view_file_audits/collect_view_file_audits n'a trouvé qu'un JoinHandle sans état de résultat retenu. Une réinsertion du JoinHandle après join est impossible. Réemploi du collecteur runtime, mêmes bornes128, réessai d'écriture au plus une fois par seconde ; aucun nouveau lecteur, helper ou store, aucune nouvelle échéance de mission. Une clôture exacte rend le constat obsolète sans affirmer sa disponibilité.

La reprise après arrêt du worker réutilise begin_view_file_audit depuis les seuls reçus materialization.stopped && available=None, au démarrage. Aucun nouveau helper, appel fournisseur ou requête client. Une panne de publication n'est pas un motif de recopie. Reprise bornée par l'inventaire existant et sa validation ; un arrêt non attesté reste exclu. Une panne de vraie écriture peut garder la quarantaine du store, jamais levée automatiquement.
