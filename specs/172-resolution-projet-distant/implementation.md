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


Worker final 780d5df4ebfd5bb72f38fce509310b5ebb2523c9 compilé en 47,59 s,
activé à 08:06:07 UTC, PID3893226. Maître Mac a53b05c2 PID53473 conservé.
T3 637/1068 et pont898 conservés.
Revue r4 refusé avant reçu : expected_base_commit_mismatch.
Lecture source : fusion ac8f9295b89d1475f32af85f9e3e9d12d3f89008 à 10:04:57,
faite par une autre session pendant la recette. Aucun fichier Revue modifié ici.
La recette r5 utilise ce nouveau SHA exact, mêmes contraintes restantes.

Recette Revue r5 réussie, task800dae46-f45a-4e7c-bc51-9d1e7dabba7d,
SHAac8f9295, revue0.1.0, marqueur exact, ACK et rejeu même tâche/résultat.
Deux tâches finales cleanup_done=true, un lancement chacune.
Recette supplémentaire parent natif local refusée resource_saturated avant
reçu/tâche ; aucune autre route. Limite conservée au reçu de livraison.


## Complément d'autonomie après livraison
Le 18 octobre, nouvelle vérification : maître 172-a53b05c2 et worker PID3893226
conservés, sources 65506609. La charge du Mac est 20,54 pour 16 cœurs,
au-dessus du seuil normalisé100 ; aucun seuil ni processus n'est modifié.
Les appels MCP du tour automatique échouent avant admission avec :
t3_session_unavailable: permission_attestation_unavailable.
Ce refus apparaît deux fois, sur bridget_status ; aucun lancement ni identité
de substitution n'est tenté. La source T3 readMcpProviderPermissions refuse
un fait absent, périmé ou lié à un autre tour. Sa cause précise reste inconnue.

Ajout d'un test de bout en bout isolé, sans T3 :
spec172_parent_natif_auto_sans_t3_resout_projet_et_attend_ready.
Deux cas emploient un vrai maître, un vrai worker et un vrai wrapper, un parent
natif avec NativePermissionFact, AutoDelegateV2 sans project_id et un fournisseur
fermé simulé. Ils prouvent résolution, worktree au commit exact, délai Ready6s
toléré avec le bail60s, résultat remis une fois et même tâche au rejeu.
1 test / 2 cas PASS en25,59s. Pas de modèle externe ni d'appel T3 dans ce banc.

Le relais ancien162 ne reconnaissait pas ReleaseLeaseV2. Il masque maintenant
cette capacité pour ses cas historiques ; le nouveau cas négocie explicitement
la capacité moderne. Contrôle ancien : 1 test / 4 cas PASS en39,36s,
dont expiration du permis5s et libération de capacité après échec.
Ce lot ne change que le banc et ses documents ; aucune compilation release ni
activation supplémentaire. La recette avec un vrai modèle parent natif reste
à réaliser après retour de la capacité et de l'attestation.


## Reprise du correctif : droits et parents distants
Les trois tours natifs de l’exécution T3 311 sont liés à la même tentative.
CodexAdapterV2 retirait le fait de droits au premier terminal natif, tandis
que goalHolds continuait la même exécution. Trois tests reproduisent le refus
puis passent : deux continuations et fin completed/interrupted/failed.
Référence T3 : goalHolds/registerRootTurn/emitRootTerminal dans le fork actif.
Le checkout upstream possède seulement le cycle de session Codex, sans notre
attestation Bridget. On conserve l’autorité par exécution et on transfère
son propriétaire vers le contexte suivant, sans nouvelle révision ni droit.

Recette parent natif réel Cartae : request recette172-parent-natif-cartae-20261018-r2,
task7ed49c85-2860-4ef3-ad31-8d63d1a3184b. Parent lancé ; enfant refusé
workspace_base_preflight_failed avant création. Résultat lu et acquitté.
Cause dans validate_routing_source : lecture locale inconditionnelle du chemin
/home/moi/10.Scripts/64.bridget par le maître Mac. La source est sur Cartae.
Référence T3 : préparation Git dans l’environnement de ThreadLaunchService ;
la différence Bridget est le transport SSH entre coordinateur et exécutant.
Correction : preuve de source en lecture sur le worker de la commande parent
attestée, avant admission et après décision. Capacité négociée, aucun repli
vers un ancien worker. Mapping source=destination dérivé du projet déjà
provisionné sur cette machine ; protections, SHA, révision et droits conservés.


## Retour de résultat et délai de l’hôte (18 octobre)
Une vraie tâche de codage Cartae a produit un test, dix tests Python PASS et
le marqueur BRIDGET172_CODAGE_CARTAE_OK dans son worktree isolé e125950f.
Task e125950f-9ceb-4dc3-92e6-2841e0566f05. Deux lectures MCP ont d’abord
expiré (« budget JSONL dépassé »). Le résultat complet est ensuite arrivé
par task_status ; ACK puis rejeu strict : même Task, même résultat, aucun doublon.
Ce succès différé n’a pas été présenté comme une correction de la panne.

Le relevé de piles du daemon montre handle_attested → reattest → attest_http
attendant T3. Le client accordait 10 s pour l’opération, contre trois échanges
HTTP de 10 s et une fermeture de 3 s autorisés au daemon. La récupération
d’admission conservait aussi un budget de 10 s. Le client réserve maintenant
ces 33 s, plus son budget de transport, avant l’éventuelle attente métier.
Aucun droit n’est mis en cache ou élargi ; les refus tardifs restent des refus.
Le JSONL reste borné à 16 Mio ; la réponse test dépasse le tampon de 8 Kio.

Autre défaut reproduit : daemon_supports_wait convertissait toute panne de
transport en false, donc en wait_unsupported. Les outils d’attente, ACK et
la sonde d’admission conservent maintenant les erreurs techniques. Un rapport
de capacités valide reste nécessaire pour annoncer l’absence d’une capacité.
Cinq nouveaux cas RED sur l’ancien comportement, puis neuf cas172 PASS,
y compris la variante d’admission ; 19 régressions du client PASS.

Référence T3 examinée : packages/effect-codex-app-server/src/protocol.ts,
request/Deferred et lecture JSONL ; réponse corrélée et erreur de transport
séparées. Le module upstream ne porte pas l’attestation HTTP Bridget : son
budget supplémentaire est une adaptation nécessaire. Le fork actif T3,
OrchestratorMcpService.sessionIdentity, vérifie le fait et le run avant/après I/O.
Ces contrôles sont conservés.

Les essais des vrais parents ont révélé une autre coupure, côté hôte Codex :
Revue r6 a expiré deux fois à 30 s, sans reçu. Aucun reçu Revue r6 en base au
contrôle suivant. Opus2D r2 a été admis sous Task b00ae037-ed93-4106-9e4b-68fbee5af3d2,
puis a terminé failed avec remote_start_failed:permission_attestation_unavailable.
Ces essais ne sont pas comptés comme réussis. Les deux profils Codex du Mac
configuraient réellement le serveur Bridget à 30 s.

Référence T3 examinée pour ce cas : provider/Drivers/CodexMcp.ts et
bridget/BridgetMcp.ts. Le montage officiel reprenait ce délai court tel quel.
L’adaptateur fournit désormais un minimum de 3700 s, comme le wrapper Codex
natif déjà présent dans Bridget ; un délai supérieur reste conservé. Les
missions gardent leurs propres échéances. Les serveurs étrangers, opt-out,
identités et configurations lues restent inchangés. Deux cas RED (absent/30 s),
puis les 42 tests du montage PASS ; lint des deux fichiers : zéro erreur/warning.
Les deux profils locaux utilisent le même fichier de configuration (le profil
cx-pro est lié au fichier Codex). Ce fichier commun a reçu ce seul changement,
avec sauvegarde privée et comparaison de tous les autres champs.

Régressions : 19 tests du client et 19 de délégation PASS. Les attentes MCP
ont donné16 PASS/1 ancien oracle : il attendait wait_unsupported sur une socket
absente. Cet oracle est aligné sur daemon_unreachable ; les cas anciens avec
rapport de capacités valide restent distincts. HTTP/attestation : huit PASS.


## Derniers défauts révélés par les vrais parents
Après installation40e4, Revue r6 reprise strictement a répondu en une seconde
expected_base_commit_mismatch : son HEAD avait changé depuis ac8f9295.
C’est le refus attendu pour une demande ancienne non admise ; aucun nouveau
SHA n’a été substitué dans sa clé. Opus2D r2 a été lu/ACK par son propriétaire.
Opus2D r3, SHAe3bb4066fbb4513d682cf156049445c658c52cd8, a été refusé
worker_unavailable pour dix candidats. Le service Cartae était actif et une
reconnexion apparaît deux secondes après le refus. Aucun résultat Opus2D final
n’est annoncé. Les rapports existent dans les demandes Bridget suivies ; deux
lectures parallèles depuis notre vieux MCP ont ensuite échoué au HTTP3s.

Le relevé de pile de T3 pendant une requête HTTP montre733/733échantillons
sur sa boucle principale dans StatementExecutionHelper::All. sessionIdentity
chargeait deux fois runs, messages, subagents, providerThreads, providerTurns,
runtimeRequests, contextTransfers et turnItems. Seuls thread et runs servent
aux contrôles. Référence T3 réutilisée : ThreadManagementService attend déjà
une exécution avec getProjectThreadRecords(...,[runs]), sans historique.
Le correctiff3ddfca24278ab1ebb689087e5c42c6bc00c6551 sélectionne runs
avant et après la vérification source. Identité, exécution, tentative, montage,
archivage et suppression restent contrôlés. Un test RED puis16PASS dont les
refus et dérives ; format/lint0/0. Paquet signé construit et installé sur disque,
ASAR4d57cee627d68079504e613cf93d532fa6399c7026a14c2b8f24fc26aa52a9b3.
T3 conserve ses anciens processus. Aucun bénéfice de latence runtime n’est
présumé avant son activation.

La sonde worker V2 accordait5s alors que les lectures Git individuelles sont
bornées à10s. Après expiration, son résultat valide tardif provoquait un
WorkerRejected puis une reconnexion. Le banc de vrai RPC avec réponse à6s
reproduit WorkerUnavailable et ce rejet, sans admission ni fournisseur.
Correction : budget de sonde60s avant toute réservation ; expiration nommée
probe_expired, différente d’un worker absent. Une réponse en lecture déjà
consommée/expirée est ignorée après contrôle d’époque/identité/forme. T3 applique
exactement cette règle dans effect-codex-app-server/protocol.ts resolvePending.
Les contrôles d’admission et les résultats d’autres époques restent fermés.
La déconnexion libère maintenant aussi pending_candidate_probes_v2 : test RED
puis PASS. Le banc retardé PASS couvre refus précis, doublon ignoré, expiration
courte, ancienne époque refusée et absence de Task/reçu.
L’observation de charge est prise une seule fois, en fin de sonde, pour rester
fraîche après les contrôles. Les admissions legacy/auto1/auto2 ont le même budget
de préparation600s ; les échéances métier et les seuils de ressources restent
inchangés. Un diagnostic du worker donne corrélation, nombre et durées des
candidats, sans chemins, politiques ou credentials. La recette Opus2D reste
nécessaire pour mesurer sa durée réelle avec ce diagnostic.

Régressions : client natif19PASS ; déconnexion1PASS ; worker20PASS/1Locked au
premier essai parallèle. Le Locked était déjà apparu une fois avant cette reprise.
Un descripteur F_DUPFD_CLOEXEC partageant la même description de fichier rend
le défaut déterministe : fermer le File propriétaire ne libérait pas flock tant
que ce descripteur restait ouvert. T3 ne possède pas de WorkerStore/flock ;
aucun équivalent direct de ce cas système n’a été trouvé. WorkerFileLock libère
maintenant explicitement LOCK_UN à son Drop, puis ferme le File. La protection
contre un propriétaire encore actif est conservée. RED déterministe puis PASS,
et module worker21PASS en parallèle. Le PID exact d’un héritier lors du premier
échec intermittent n’a pas été observé ; la reproduction établit le mécanisme.
Routage35cas :28PASS/7FAIL initiaux. Un oracle attendait encore le rejet d’une
ancienne corrélation ; il vérifie maintenant qu’elle est ignorée et ne satisfait
jamais la sonde courante. Les six autres échecs venaient du mutex Laya empoisonné
par cette assertion. Reprises : oracle1PASS et Laya7PASS. Aucun GREEN global ni
nouvelle FULL déduits. Protocole historique remote162 :9PASS.


## Activation et recettes finales réussies
Source124dc47a9fe2ad2717d06c53053b71d9f0f7dc26 intégrée et poussée.
Build Linux101,11s RC0 ; Mac142,30s RC0, signé. Maître81018 et worker4049619
actifs ; T3main637/serveur1068/pont898 inchangés. Aucun schéma changé.
Avant arrêt du maître51581, aucune mission native active/cleanup en cours.
Après redémarrage, les rejeux officiels de c5d2017b et e125950f ont rendu les
mêmes résultats/Tasks/ACK malgré le changement de HEAD. Quick_check=ok.
Le banc natif sans T3 sur le code courant est PASS :1test/2cas,25,57s.

Nouvelle mission réelle Bridget : a62d8fb1-9fef-4c92-9d80-0d1e1b475790,
SHA124dc47a, version0.4.1, résultat833octets, marqueur exact, ACK et rejeu
identiques. La première sonde réelle est mesurée à4,426s pour dix candidats.

Revue r7 : Task30a2525b-7d62-4be8-a3a6-b5b750382009, sans project_id,
SHAd9d29fe69c9969af913cb99ce70fd249fc723b67, revue0.1.0,
BRIDGET172_REVUE_FINAL_OK. Le parent Rapport a lu le résultat complet,
confirmé ACK et comparé le rejeu strict : mêmes Task/résultat. Le dépôt géré
initialement créé sans déclaration manuelle est actualisé au commit demandé.
L’agent de recette a seulement lu package.json et .git/HEAD détaché.

Opus2D r4 : Taskea8d9680-dd4e-4a71-a22d-1589ee028f9f, sans project_id,
SHAed82547ff65fd23307f63ee6683c9e9e7b22d420, opus2d0.1.0,
BRIDGET172_OPUS2D_CARTAE_OK. Le vrai parent trajets bus a lu le résultat,
confirmé ACK et comparé le rejeu strict : mêmes Task/résultat1007octets.
L’enfant a lu package.json, .git/HEAD et refs/heads/main sous le dossier Cartae.
Les sondes initiales Revue et Opus2D ont pris4,317s et4,303s ; ces mesures
n’attribuent pas rétrospectivement une durée précise à l’ancien refus r3.
Contrôle durable : six clés de recettes réussies, chacune une seule Task,
state=result_available, error=null, cleanup_done=true, acknowledged_at présent.

Aucun autre serveur que Cartae testé. Les recettes actuelles partent du T3
ancien encore chargé et prouvent Bridget corrigé ; le nouveau T3f3ddfca2
reste installé/signé sur disque, non activé. La poursuite automatique de goal
avec ce nouveau T3 reste à recetter après l’accord de redémarrage demandé.
Les anciennes limites et échecs restent historiques, pas effacés ou comptés PASS.

## Continuation distante après renouvellement du tour parent

Incident réel VOITURES : la tâche54569150 garde une attestation du tour850,
révision17. Les deux contrôles de Continue comparaient le fait entier avec celui
du tour courant. Un renouvellement valide pouvait donc produire
permission_not_inherited sans changement de droits.
Référence T3 : le dépôt amont /Users/moi/11.Repositories/t3code ne possède pas
ce port de continuation distante. Dans le fork actif, OrchestratorMcpService.sendToThread
relit le parent vivant puis contrôle les plafonds runtime/interaction, sans
exiger le numéro de son ancien tour. Bridget conserve une règle plus stricte :
seuls run_id et revision sont neutralisés pour une source provider_turn fraîche.
Session, fournisseur, pilote, cwd, modes et politique restent strictement égaux.
Les faits native_wrapper restent entièrement égaux. Les contrôles du même
propriétaire, de révocation, de sources et du worker restent inchangés.
Le helper partagé évite une divergence entre le contrôle avant RPC et le puits
de création de ronde. Coût O(B), copie bornée du fait ; aucun stockage ni schéma neuf.
Deux tests ont reproduit le refus avant correction. Après correction, deux PASS,
dont le franchissement du contrôle et le refus sans mutation en l’absence du worker.
Neuf mutations de droits/identité et le changement de mission native restent refusés.
Le test de RPC a d’abord utilisé un cwd fictif ; la garde source l’a refusé.
La fixture a été corrigée vers son vrai dossier, sans changer la garde.
Régressions de rondes :12PASS/2FAIL env_unfit lors de l’admission initiale.
Les deux échecs sont reproduits séparément sur la source de base6e250e63.
Ils ne concernent pas la continuation modifiée ; aucun GREEN global.
Régressions distantes162 :46PASS, y compris reprise et recontrôle des sources.
L’activation ne concerne que le maître Mac. Worker et wrapper disposent de leur
reconnexion existante ; les processus des deux missions seront relevés avant/après.
Aucune relance T3 ou worker demandée. La continuation réelle garde sa Task et sa clé.

## Sortie fatale du worker pendant une reconnexion

L’activation995 du maître a respecté les PID T3/pont. BUS a échoué api_error
à1792321458, avant le SIGTERM du maître vers1792321481. Cet échec est distinct.
Le worker4049619 a ensuite quitté sur worker_capability_unavailable, puis systemd
l’a relancé. Le processus gardé VOITURES est devenu terminal à1792321486.
La promesse de conservation de ce processus n’est donc pas prouvée et n’est pas
présentée comme réussie. Le résultat, le commit et le worktree restent conservés.
Cause : le point d’interrogation après probe_worker_capability faisait sortir
de run_worker sur une coupure réseau pendant sa sonde, au lieu de conserver
le runtime et ses enfants pendant la reconnexion. La garde du protocole reste
fermée : worker_capability_required et les refus de handshake restent fatals.
Référence T3 : packages/client-runtime/src/rpc/session.ts et client.ts séparent
les pannes de transport des refus métier. Bridget réessaie le même socket et
la même identité seulement pour les deux codes transport fermés, sans Hello ni
credential avant l’annonce de capacité. Attente bornée de1s entre connexions.
Un test de la vraie boucle reproduit exactement EOF puis sortie fatale avant
correction. Après correction il observe la deuxième connexion, sa sonde sans
secret, puis le Hello après capacité ; un refus de protocole arrête toujours.
Une fixture initiale avait un socket non privé : le refus de sécurité a été
observé, puis ses droits ont été corrigés à0600 sans modifier la garde.
Test nouveau1PASS. Module worker19PASS/3FAIL avec le mauvais Git du lanceur ;
les trois cas ont été repris seuls avec Git2.56 et ont tous passé. Aucun GREEN
global ni nouvelle FULL. L’installation worker doit encore être vérifiée.


## Complément quota du 18 octobre — cause réelle de BUS et de la reprise VOITURES

La Task BUS8dcb54b1 et la reprise VOITURESb9d8b49c ont échoué avec api_error.
Les journaux natifs Claude donnent la cause structurée : error=rate_limit,
isApiErrorMessage=true. Le texte annonce « resets 1:20pm (UTC) », soit15h20
heure de Paris ce18octobre. Ce n’est ni une panne SSH ni un défaut de droits.
La reprise b9d8b49c n’a pas produit de code ; son worktree propre reste conservé.

Référence T3 lue :
/Users/moi/11.Repositories/t3code/apps/server/src/provider/Layers/ClaudeAdapter.ts,
handleAssistantMessage vers3233 et handleResultMessage vers3268.
T3 conserve latestAssistantRateLimited à partir du code structuré rate_limit.
Une réponse ultérieure remplace cette preuve. Le résultat générique api_error
reçoit alors le diagnostic de limite d’usage. Bridget reprend ce mécanisme par tour.
Il garde la priorité du refus de permissions et de l’annulation. Le texte seul
ne déclenche pas cette classification. Une notice de quota n’est pas une réponse
métier réussie ; si le fournisseur se rétablit, la vraie réponse la remplace.

T3 ne possède pas le routage de flotte distante de Bridget. Adaptation nécessaire :
le fait de limite passe par le flux natif existant, fenêtre unknown si elle manque,
sans pourcentage, crédit ou date inventés depuis le texte. La remontée est liée à
l’enfant géré courant, sa Task distante préparée et son digest de définition.
Une seule connexion déclarée peut correspondre ; sinon le fait reste non lié.
La sonde distante relit ce fait avant la décision de routage. Le refus récent reste
restrictif après le nettoyage de l’enfant, pour la même connexion/définition/époque,
durant le seul TTL de la politique. Aucun crédit ou capacité positive d’une
instance morte n’est adopté. Sans autre cible, le port rend provider_quota_exhausted.
Le rejeu d’une décision admise ne migre pas. Les autres axes libres restent libres.

Limite explicite : aucune heure de retour n’est attestée par le code rate_limit seul.
Le texte humain n’est pas converti en fait machine et n’ouvre aucun hold160.
Une observation périmée redevient inconnue selon le TTL existant. L’authentification
et l’acceptation d’une nouvelle mission restent à prouver par l’essai réel.

Preuves ciblées privées, sous /home/moi/.cache/bridget-delivery172 :
quota-classification-red172.log (rate_limit rendu api_error avant correction),
quota-recovered-response-red172.log (notice incorrectement gardée comme réponse),
quota-routing-red172.log (fait perdu après sortie de l’enfant),
quota-remote-red172.log (sonde distante ignorant la limite), puis leurs GREEN.
34tests du pilote Claude,14tests de mémoire quota,13tests de routage,
1test de sonde distante,1test de liaison stricte et2tests de continuation PASS.
Le test de liaison refuse digest, instance, machine, gestionnaire ou compte ambigu.
Son cas artificiel qui tentait de remettre remote_prepared=false sur une Task déjà
préparée a été retiré : le magasin conserve ce fait de façon monotone. Donnée lue
avant retrait : valeur demandée false, valeur persistée true ; garde produit conservée.

Le défaut de fixture hérité env_unfit est résolu sans modifier le produit :
config-149 était créé0770 sous le masque réel du serveur, malgré le contrôle0700.
La fixture fixe maintenant0700. Les14tests native153_ronde_ passent.
Référence T3 lue : /Users/moi/11.Repositories/t3code/apps/server/src/provider/antigravityAuthSupport.ts
361–375 fixe aussi explicitement0700 après création ;
/Users/moi/11.Repositories/t3code/apps/server/src/provider/Drivers/ClaudeHome.ts20–36
isole le profil par CLAUDE_CONFIG_DIR.
Aucun compte ni profil utilisateur n’a été modifié. Aucune nouvelle FULL.

La relecture conserve les mêmes limites de stockage, d’identité et de fraîcheur.
Aucun nouveau schéma, magasin, outil, paramètre ou dépendance. Le test adversarial
ajoute des crédits synthétiques au même historique : ils restent absents du fait
rendu après la sortie de l’instance. Les crédits Codex ne traversent pas ce cas.


## Complément après les essais réels r4 et r5

r4 Task14c382ea-c831-4374-b6b8-c9f77fe456a7 et r5 Taskc9693bca-4304-4e72-abe2-442677bc8c14
ont toutes deux fini usage_limit, puis ont été lues et acquittées par leur propriétaire.
Aucun working confirmé, aucun code produit. La décision conservée prouve le choix
claude-cartae / claude-haiku-5-5 dans les deux demandes. La sonde r5 a lu ses faits
44 à47secondes après le refus r4 ; le TTL de30secondes avait déjà supprimé le refus.
Les autres modèles du même compte ont aussi des digests de mission différents.

Référence T3 courte : /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts, vers6455–6505 et7377–7387, conserve les fenêtres
rejected jusqu'au retour structuré ou à l'événement allowed. Le TTL des mesures
de charge ne constitue pas une preuve de rétablissement du compte. T3 ne couvre
pas le choix d'une flotte distante ; Bridget doit lier le refus à sa connexion.
Le nouveau test reproduit le refus oublié à137, après une observation à90 et un TTL30.
RED exact conservé dans /home/moi/.cache/bridget-delivery172/quota-delay-red172.log.

Correction en validation : empreinte privée de connexion issue du registre existant
(connexion, définition de base, révision de politique, identité/provisionnement worker).
Les modèles de la mission ne changent pas cette empreinte de compte. Un refus Claude
non levé reste restrictif après nettoyage et délai de décision, sans rajeunir son
observed_at. Les crédits et capacités positives restent soumis au digest complet,
à une instance vivante et au TTL. Une nouvelle réponse assistant sans erreur est
une preuve d'acceptation, sans prouver de crédit ou de pourcentage restant.
Une fenêtre structurée ne reçoit pas en plus un refus unknown sans date depuis
le code assistant. Aucun texte humain n'est transformé en date machine.
La mémoire reste volatile et bornée, sans schéma ni outil public supplémentaire.

La capture utilisateur concerne le fil17f50a6d-ec2f-494a-b9a3-0def5b14475b.
Les tours Claude169–172 ont échoué sur usage_limit/api_error_429, et non sur une
limite de taille du contexte. T3 a reçu un retour structuré13:20UTC/15:20Paris.
Le message binding_registration_refused est publié à11:56:05UTC dans le tour173,
qui avait repris avec Codex à11:52:21UTC. Ce refus distinct n'est pas attribué
au quota par simple corrélation. Le lien actuel est présent et occupé dans
l'inventaire ; une unique lecture de catalogue depuis ce fil a été demandée.
Son absence de réponse avant échéance ne prouve ni succès ni refus actuel.
Aucun redémarrage T3, aucune nouvelle admission métier à ce stade.


Relecture du complément : un refus sans date ne doit pas bloquer indéfiniment
un compte rétabli. Bridget conserve ce refus durant un délai de nouvelle tentative
de5minutes, distinct du TTL des métriques30secondes. Après ce délai, le fait
redevient inconnu, sans être présenté comme une capacité disponible. Une heure
structurée prime ce délai ; une vraie réponse acceptée lève le refus avant.
T3 n'a pas cette sélection distante automatique : cette borne de réévaluation
est l'adaptation nécessaire, sans parsing du texte et sans nouvelle planification.
Le test couvre137, la borne390, puis391 pour une observation à90 ; il garde
observed_at90 et resets_atnull, sans crédit. Une réponse vide ou une nouvelle
erreur d'authentification ne devient pas une preuve d'acceptation du compte.

Le refus de rattachement garde t3_session_unavailable. Il distingue désormais
primary_absent, credential_mismatch, instance_mismatch, primary_not_live et
connection_already_registered. Les gardes sont identiques et les preuves privées
ne sont jamais imprimées. Référence T3 : assertLiveCaller et bridgetSession dans
/Users/moi/11.Repositories/t3code-local/apps/server/src/mcp/OrchestratorMcpService.ts,
lecture de l'exécution vivante et contrôle du propriétaire de credential. T3 ne
possède pas la liaison auxiliaire locale du démon Bridget. Le complément rend
sa garde diagnostiquable ; il ne prétend pas avoir établi la cause historique
du refus de la capture. Le vieux processus MCP peut garder le libellé générique ;
le maître consigne alors la cause fermée, sans secret, dans son journal.


## Complément du refus remote_result_slot:StoreUnavailable

Le signalement métier r7 porte la clé
opus2d-voitures-cartae-sol61-reprise-pietons-20261018-r7.
La lecture seule du maître retrouve pourtant une admission durable :
Task350d5c53-3b0d-4b1e-90b2-150002b6f51a, puis failed/remote_start_failed,
cleanup_done=true et guard_released=true. Le premier appel ne rendait pas
cet identifiant. Le propriétaire a reçu cet identifiant pour relire l’existant ;
aucune nouvelle admission métier n’a été créée.

Le store réel contient 73 commandes, 103 slots et 17 reçus de préparation.
Ses fichiers occupent 612964 octets. Le calcul ancien compte 15788644 octets
avec les réservations ; ajouter un slot exige 2228224 octets et dépasse
le plafond inchangé de 16777216 octets. Les anciens reçus soldés continuent
de réserver chacun 64 Kio, même après collecte de leur blob. La collecte existe
déjà et garde les reçus sept jours : la supprimer ou vider le store aurait
détruit des preuves de rejeu sans corriger ce calcul.

Référence T3 lue pour ce défaut : les fonctions getAgentReplayStats/getReplayStats
dans /Users/moi/11.Repositories/t3code-local/apps/server/src/persistence/OrchestrationEventStore.ts
mesurent les octets réels avec octet_length(payload_json). T3 ne possède pas
de réservation équivalente pour un résultat de worker distant. Bridget conserve
donc sa réserve de publication de 2 Mio par résultat actif, sa réserve de croissance
pour les métadonnées en cours, et ses deux remplacements globaux sérialisés.
Une commande Terminal/Refused libère les 64 Kio de croissance ; ses 4 Kio
de drainage et de détails de sortie tardifs restent réservés. Un slot clôturé
dont le producteur est terminé libère cette croissance si son résultat est
définitif ou absent. Une préparation achevée et immutable la libère également.
Tous les fichiers, y compris les temporaires et les orphelins, gardent leur poids réel.
Les plafonds d’octets et d’entrées restent inchangés. Aucun reçu réel n’est supprimé.

Le worker distingue maintenant le refus typé WorkerStoreError::Quota :
WorkerRefusal::ResourceLimit, au lieu de StoreUnavailable. Les erreurs techniques
conservent leur refus fermé sans publier leur détail privé. Aucun nouveau champ
de protocole, schéma de base ou outil n’est ajouté.

Référence T3 lue pour la perte d’identifiant :
/Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/CommandReceiptStore.ts
conserve commandId, status et error, et retrouve ce reçu par sa clé.
Bridget réutilise ce principe après son admission durable. Une erreur de préparation,
de contrôle ou de démarrage rend le reçu de la même Task avec son erreur conservée.
Le statut reste queued/starting pendant la résolution du lancement ; il ne prouve
pas un travail métier. Les gardes de droits et de libération ne changent pas.
Le rejeu exact retrouve la Task ; une enveloppe différente reste refusée.
Un refus avant admission ne crée toujours aucune Task.

Trois régressions RED sont conservées avant correction : saturation au 106e reçu
terminé, quota masqué par StoreUnavailable, et perte de la réponse après admission.
Les premiers contrôles GREEN172 donnent 23 PASS. Le test du store conserve
120 résultats et reçus, vérifie leur relecture après réouverture et leur rejeu,
puis admet sept résultats actifs et refuse le huitième. Les séries de régression
complémentaires sont conservées dans les reçus de validation.

La nouvelle source reste sans activation pendant BUS453c75be-503b-4934-9311-e382733498aa.
Aucun redémarrage T3, aucune suppression de store ni intervention sur cette mission.
Ce banc ne remplace pas une délégation réelle sur le nouveau binaire activé.


## Activation après clôture BUS et contrôle réel

Le propriétaire BUS a terminé la ronde2 et fermé sa garde. La lecture seule du
maître confirme cleanup_done=true, guard_released=true et résultat conservé.
Le groupe320286 est absent. Aucun autre mandat natif actif ni admission auto
ouverte : le paquet préparé72a8e8f8 peut être activé sans interrompre un calcul.

Maître7825 actif depuis21:15:02CEST ; worker624845 actif, NRestarts0.
Source72a8e8f8823c5f23b463a45693bb9b27079e41f3, build172-72a8e8f8.
Les PID T330609/30660 et pont898 restent inchangés. Les anciens binaires
et l’unité worker sont conservés pour retour arrière. Aucun schéma modifié.

Le premier appel officiel après la relance du maître refuse le rattachement
avec le motif fermé primary_absent. Le pont existant se reconnecte puis réinscrit
les26fils ; les appels officiels suivants status et capabilities réussissent.
Aucun credential emprunté, aucune identité réécrite, aucune nouvelle route.
Ce cas transitoire observé ne prouve pas la cause historique de la capture Claude.
Le journal du nouveau maître conserve les avertissements avec RUST_LOG=warn.

Recette officielle bridget_auto_delegate :
request_id recette172-store72-cartae-live-20261018-r1,
Task3a825355-37b1-45e1-8bdf-9aef6de39dd4,
enfantb58e755e-920c-4be7-9a42-887c577079cf.
Cartae est imposé ; les autres axes sont choisis par Bridget :
claude/claude-haiku-5-5. Posturedevelopment pour conserver un enfant continuable,
mission explicitement limitée aux lectures. Worktree demandé et créé par Bridget,
sans project_id : /home/moi/10.Scripts/64.bridget/.worktrees/3a825355.
Le SHA lu est5c2b4635e3bdeda78f3687f1b0a04b8bd6831744 ; Cargo.toml donne0.4.1.
Marqueur BRIDGET172_STORE72_LIVE_OK ; résultat complet891octets lu puis ACK.
Le rejeu exact garde TaskID et résultat byte-identique.

La continuation officielle sur le même enfant, clé
recette172-store72-cartae-live-20261018-c2, ouvre la ronde2 et lit la borne
MAX_WORKER_STORE_BYTES=16*1024*1024. Marqueur BRIDGET172_STORE72_CONTINUE_OK,
résultat complet853octets lu puis ACK. Le rejeu de cette continuation conserve
la même Task, la ronde2 et le même résultat. Aucune nouvelle exécution au rejeu.
Ce contrôle utilise le tour parent courant ; le renouvellement automatique
d’un tour parent distinct n’est pas qualifié par ces deux rondes.

Après la première ronde, le store réel conserve74commandes,104slots et18reçus
de préparation :619207octets physiques,1184455octets comptés avec les réserves
du nouveau calcul. Plafond inchangé16777216octets. L’admission et la publication
réelles réussissent sans purge manuelle des anciens reçus.

La garde du test est ensuite fermée par bridget_task_cancel. Résultat conservé,
cleanup_done=true et guard_released=true ; commande terminale et groupe626253vide.
BUS conserve aussi son résultat, ses deux preuves de clôture et son worktree.
Le propriétaire a reçu l’information de levée du blocage, sans nouvelle recette
demandée. La reprise métier VOITURES lui appartient et n’est pas une livraison
de code revendiquée par cette recette de lecture.


## Incident du 19 octobre — plafond MCP invalide après échec fournisseur

Trajets bus a conservé le refus de bridget_task_status sur la Task
62363500-c0cf-409b-b669-5549f33cfa7b. L’appel exact est daté du18octobre à
23:45:35.364UTC, refus à23:45:35.604UTC. Le constat a ensuite été consigné à
23:46:17.781388UTC dans le reçu métier externe. Les outils send et status du
même fil ont aussi rencontré ce refus. Aucune nouvelle admission n’a été créée.

La lecture seule du maître retrouve un événement durable provider_failed,
non acquitté, à22:48:29UTC, pour l’instance du parent trajets bus. Sa Task est
déjà failed/terminal Codex:failed, cleanup_done=true et guard_released=true.
L’erreur de plafond est distincte de cet échec fournisseur, dont la cause fine
n’est pas attribuée à un quota ou au contexte sans autre preuve.

Le serveur rejoue les avis DelegatedRuntimeEvent après tout Registered,
y compris RegisterAuxiliary. Le client MCP ouvre une connexion auxiliaire pour
CallerScope et attend un objet fermé version/caller_read_only. Il reçoit donc
cet avis d’incident avant la réponse à sa requête et refuse le protocole.
La présence du fait durable et le code du rejeu expliquent le défaut réel.

Référence T3 lue pour ce cas : le runtime
/Users/moi/11.Repositories/t3code-local/packages/provider-acp/src/server/AcpSessionRuntime.ts
expose request via acp.raw.request, notify via acp.raw.notify et getEvents via
sa file dédiée. Le test buffers root metadata while session startup is still pending
dans /Users/moi/11.Repositories/t3code-local/apps/server/src/provider/acp/AcpJsonRpcConnection.test.ts
vérifie aussi la coexistence des événements et de la réponse de démarrage.
T3 ne possède pas la paire primaire/auxiliaire Unix de Bridget ; le mécanisme
réutilisable est la séparation des notifications et des réponses de requête.

Le correctif reprend la garde déjà utilisée pour la remise d’incident en direct :
seule la connexion primaire reçoit le rejeu ; l’auxiliaire MCP n’en reçoit aucun.
Il ne supprime ni n’acquitte l’avis durable, ne saute aucun contrôle de droits,
et ne change aucun protocole ou schéma. La modification concerne le maître seul.

Le nouveau test utilise le véritable handle_connection avec des sockets Unix et
une inscription auxiliaire attestée. Avant correction il reçoit bien
DelegatedRuntimeEvent au lieu de CallerScope (RED). Après correction il vérifie
le plafond exact pour Warning et Failed, puis la remise et l’ACK par le parent
primaire. L’avis reste en attente tant que ce parent ne l’acquitte pas.
Reçu RED : /home/moi/.cache/bridget-delivery172/auxiliary-scope172-red.log.


## Erreurs de préparation distante lisibles — 21 octobre 2026

Demande utilisateur : laisser le nettoyage Cartae à son responsable et rendre le
refus de préparation compréhensible. Aucun fichier de cache distant supprimé par
ce correctif ; aucun dossier métier, reçu ou résultat modifié.

Cas réel : Task ba369015-da43-45c4-b70a-dae9509ea2db, reçue durablement avant
l’échec `remote_workspace_prepare:StoreUnavailable`. Son reçu de préparation
contient record=null ; aucun worktree ni enfant n’existe. À l’observation,
Cartae offrait environ 1,1 Go pour une extraction de 7 981 663 707 octets.
L’ancien code avait jeté le diagnostic Git ; cette erreur historique ne sera
pas réécrite en un nouveau diagnostic présenté comme observé.

Référence T3 lue avant modification :
/Users/moi/11.Repositories/t3code-local/apps/server/src/vcs/GitVcsDriverCore.ts,
classifyGitFailure et executeGit : classifier les lignes de diagnostic Git en
raisons fermées, sans transmettre stderr ni ses chemins/URL éventuels.
GitManager délègue createWorktree à ce service. T3 ne possède pas le reçu
WorkerStore distant ; Bridget conserve cette différence nécessaire pour son rejeu.

Adaptation : Git est lancé avec LC_ALL=C. Les diagnostics fatals/error de
l’extraction distinguent espace insuffisant et accès disque refusé. Le worker
transmet seulement DiskFull, FilesystemAccessDenied, WorkspacePrepareFailed ou
WorkspacePrepareIncomplete. Les erreurs OS ENOSPC/EDQUOT sont distinguées des
plafonds de réservation ResourceLimit. Les erreurs de stockage non classées
restent StoreUnavailable ; aucune cause n’est inventée depuis ce seul code.

La réponse native ajoute error_message en français pour un échec de préparation.
Le champ error, l’identifiant de tâche, l’état, la clé et le rejeu restent
inchangés. Les vieux refus StoreUnavailable reçoivent une explication qui dit
explicitement que la cause précise manque. Aucun stderr brut n’est transmis.

Tests et activation : voir le reçu de livraison et les journaux privés sous
/home/moi/.cache/bridget-message-errors172. Le travail s’effectue dans le
worktree existant de la session172 ; les services restent inchangés pendant
les tests et la préparation.

Validation ciblée : 10 tests distincts PASS, aucune suite complète. La simulation
Git ENOSPC passe par le vrai exécuteur Git et le nettoyage conditionnel de branche.
Les tests du worker couvrent les refus OS, les codes sérialisés et les détails
privés exclus. Le test de reçu maître vérifie les messages, le même TaskID et
le rejeu strict sans nouveau lancement. Les régressions d’extraction, de
préparation/rejeu d’un vrai worktree, d’admission et de réserve du store passent.
Un filtre vide a été exclu du compte, puis remplacé par les deux tests du store.


Complément après revue indépendante r1 : l’import local du commit utilise lui
aussi la classification Git. Les motifs doivent terminer la ligne de diagnostic,
ce qui évite de prendre un nom de fichier pour une erreur système. Le test ENOSPC
crée maintenant réellement sa branche avant l’échec et prouve son retrait.

Références T3 supplémentaires lues : GitVcsDriverCore.ts collecte stdout et
stderr séparément (lignes982–1024) ; Bridget reprend cette séparation avec un
maximum de 1 Mio par flux et drainage du surplus. Orchestrator.ts, autour de
Workspace preparation failed (lignes8275–8334), présente une préparation échouée
comme telle. Bridget projette désormais work_state=failed lorsqu’une préparation
a échoué avant toute création d’enfant, sans modifier status ni le reçu durable.
error_recovery précise que le rejeu ne relance rien et qu’il faut corriger la
cause puis annuler la tâche avant une nouvelle demande. Le reçu incomplet invite
à vérifier le disque et le dossier. Aucun nettoyage ou redémarrage implicite.

14 tests distincts PASS après ce complément. La première exécution d’un nouveau
test de flux a refusé une forme status incomplète ; le test emploie maintenant
la forme déjà autorisée. La liste blanche Git n’a pas changé. Les paquets161df06a
préparés avant cette revue ne seront pas activés. Les nouveaux motifs du protocole
fermé imposent d’activer le maître corrigé avant le worker corrigé ; le retour
arrière doit suivre l’ordre inverse. Les anciens workers restent lisibles par le
nouveau maître. Aucune compatibilité ancien maître/nouveau worker n’est annoncée.
