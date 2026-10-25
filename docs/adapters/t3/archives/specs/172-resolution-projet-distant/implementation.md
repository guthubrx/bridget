# Archives techniques de l’adaptateur T3

Document d’origine : [specs/172-resolution-projet-distant/implementation.md](../../../../../../specs/172-resolution-projet-distant/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Développement sur Cartae, branche session-172-resolution-projet-distant.
Source initiale : c48c30fbbfaa62562ce4a9ff03e31b92444fb533.
Appel refusé : opus2d-cartae-lecture-bridget-20261018-9ecd0abe-r1.
Fil T3 : e4999bf4-8f05-401b-bdc2-a9bb32b0b9f7, position 33665.

<a id="extrait-2"></a>

## Extrait 2

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

<a id="extrait-3"></a>

## Extrait 3

Donnée réelle : le reçu worker de la deuxième recette Bridget passe de créé
à terminal en cinq secondes, sans permis ni fournisseur prêt. Le code réatteste
les droits T3 après Ready, mais le bootstrap expire après cinq secondes.
Référence relue : ThreadLaunchService.ts, préparation puis prepared-run.release.
Adaptation distante : fenêtre bornée de 60 s négociée ReleaseLeaseV2 ; ancien
worker conserve un permis de 5 s. Aucun relâchement des droits, de la révision,
de l'époque ou du contrôle du parent. Attestation HTTP : 10 s par requête ;
AutoDelegateV2 : budget client 600 s pour inclure la préparation du dépôt.
Les erreurs définitives connues de démarrage conservent maintenant leur motif.

<a id="extrait-4"></a>

## Extrait 4

Validation complémentaire : test HTTP réel avec réponse après 3,5 s PASS ;
3 tests spec172 PASS, 8 tests T3 MCP PASS, 2 tests récupération d'admission PASS.
Tests worker : première exécution parallèle 20 PASS / 1 verrou occupé.
Le test exact de reprise réussit seul, puis les 21 tests worker passent en série.
La cause de concurrence du verrou n'est pas corrigée ni déclarée résolue ici.
Protocole distant : 9 PASS. La nouvelle recette de bout en bout reste nécessaire.

<a id="extrait-5"></a>

## Extrait 5

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

<a id="extrait-6"></a>

## Extrait 6

Worker final 780d5df4ebfd5bb72f38fce509310b5ebb2523c9 compilé en 47,59 s,
activé à 08:06:07 UTC, PID3893226. Maître Mac a53b05c2 PID53473 conservé.
T3 637/1068 et pont898 conservés.
Revue r4 refusé avant reçu : expected_base_commit_mismatch.
Lecture source : fusion ac8f9295b89d1475f32af85f9e3e9d12d3f89008 à 10:04:57,
faite par une autre session pendant la recette. Aucun fichier Revue modifié ici.
La recette r5 utilise ce nouveau SHA exact, mêmes contraintes restantes.

<a id="extrait-7"></a>

## Extrait 7

## Complément d'autonomie après livraison
Le 18 octobre, nouvelle vérification : maître 172-a53b05c2 et worker PID3893226
conservés, sources 65506609. La charge du Mac est 20,54 pour 16 cœurs,
au-dessus du seuil normalisé100 ; aucun seuil ni processus n'est modifié.
Les appels MCP du tour automatique échouent avant admission avec :
t3_session_unavailable: permission_attestation_unavailable.
Ce refus apparaît deux fois, sur bridget_status ; aucun lancement ni identité
de substitution n'est tenté. La source T3 readMcpProviderPermissions refuse
un fait absent, périmé ou lié à un autre tour. Sa cause précise reste inconnue.

<a id="extrait-8"></a>

## Extrait 8

Ajout d'un test de bout en bout isolé, par le parcours natif Bridget :
spec172_parent_natif_auto_sans_t3_resout_projet_et_attend_ready.
Deux cas emploient un vrai maître, un vrai worker et un vrai wrapper, un parent
natif avec NativePermissionFact, AutoDelegateV2 sans project_id et un fournisseur
fermé simulé. Ils prouvent résolution, worktree au commit exact, délai Ready6s
toléré avec le bail60s, résultat remis une fois et même tâche au rejeu.
1 test / 2 cas PASS en25,59s. Pas de modèle externe ni d'appel T3 dans ce banc.

<a id="extrait-9"></a>

## Extrait 9

## Reprise du correctif : droits et parents distants
Les trois tours natifs de l’exécution T3 311 sont liés à la même tentative.
CodexAdapterV2 retirait le fait de droits au premier terminal natif, tandis
que goalHolds continuait la même exécution. Trois tests reproduisent le refus
puis passent : deux continuations et fin completed/interrupted/failed.
Choix de conception Bridget
Le checkout upstream possède seulement le cycle de session Codex, sans notre
attestation Bridget. On conserve l’autorité par exécution et on transfère
son propriétaire vers le contexte suivant, sans nouvelle révision ni droit.

<a id="extrait-10"></a>

## Extrait 10

Le relevé de piles du daemon montre handle_attested → reattest → attest_http
attendant T3. Le client accordait 10 s pour l’opération, contre trois échanges
HTTP de 10 s et une fermeture de 3 s autorisés au daemon. La récupération
d’admission conservait aussi un budget de 10 s. Le client réserve maintenant
ces 33 s, plus son budget de transport, avant l’éventuelle attente métier.
Aucun droit n’est mis en cache ou élargi ; les refus tardifs restent des refus.
Le JSONL reste borné à 16 Mio ; la réponse test dépasse le tampon de 8 Kio.

<a id="extrait-11"></a>

## Extrait 11

Le relevé de pile de T3 pendant une requête HTTP montre733/733échantillons
sur sa boucle principale dans StatementExecutionHelper::All. sessionIdentity
chargeait deux fois runs, messages, subagents, providerThreads, providerTurns,
runtimeRequests, contextTransfers et turnItems. Seuls thread et runs servent
aux contrôles.

<a id="extrait-12"></a>

## Extrait 12

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

<a id="extrait-13"></a>

## Extrait 13

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

<a id="extrait-14"></a>

## Extrait 14

Aucun autre serveur que Cartae testé. Les recettes actuelles partent du T3
ancien encore chargé et prouvent Bridget corrigé ; le nouveau T3f3ddfca2
reste installé/signé sur disque, non activé. La poursuite automatique de goal
avec ce nouveau T3 reste à recetter après l’accord de redémarrage demandé.
Les anciennes limites et échecs restent historiques, pas effacés ou comptés PASS.

<a id="extrait-15"></a>

## Extrait 15

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

<a id="extrait-16"></a>

## Extrait 16

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

<a id="extrait-17"></a>

## Extrait 17

La capture utilisateur concerne le fil17f50a6d-ec2f-494a-b9a3-0def5b14475b.
Les tours Claude169–172 ont échoué sur usage_limit/api_error_429, et non sur une
limite de taille du contexte. T3 a reçu un retour structuré13:20UTC/15:20Paris.
Le message binding_registration_refused est publié à11:56:05UTC dans le tour173,
qui avait repris avec Codex à11:52:21UTC. Ce refus distinct n'est pas attribué
au quota par simple corrélation. Le lien actuel est présent et occupé dans
l'inventaire ; une unique lecture de catalogue depuis ce fil a été demandée.
Son absence de réponse avant échéance ne prouve ni succès ni refus actuel.
Aucun redémarrage T3, aucune nouvelle admission métier à ce stade.

<a id="extrait-18"></a>

## Extrait 18

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

<a id="extrait-19"></a>

## Extrait 19

La nouvelle source reste sans activation pendant BUS453c75be-503b-4934-9311-e382733498aa.
Aucun redémarrage T3, aucune suppression de store ni intervention sur cette mission.
Ce banc ne remplace pas une délégation réelle sur le nouveau binaire activé.

<a id="extrait-20"></a>

## Extrait 20

Les sondes v1/v2 disposent elles aussi d’un refus ResourceLimit, avec leurs
identifiants d’origine et des ressources inconnues (None). Ce nouveau motif
fermé de CandidateProbeRefusal impose la même activation maître avant worker.
Retour arrière : worker/client/unité anciens avant ancien maître.
Aucune compatibilité ancien maître/nouveau worker n’est annoncée.
T3 ne possède pas l’équivalent de cette file distante assortie d’un délai RPC
de trois secondes. Sa file locale a été lue ; ce refus explicite est l’adaptation
nécessaire à la borne du transport Bridget, sans file non bornée ni délai muet.

<a id="extrait-21"></a>

## Extrait 21

Déploiement obligatoire : maître corrigé avant worker corrigé. Retour arrière :
worker/client/unité anciens avant maître ancien. Les nouveaux motifs fermés
ne garantissent pas un mélange ancien maître/nouveau worker. Les paquets1c5d8b6e
restent non activables. Aucune nouvelle compilation de livraison avant le verdict
de la revue r5. Aucun redémarrage T3, aucun nettoyage Cartae par ce lot.
La cause fine de la coupure historique837972ad reste non prouvée.

<a id="extrait-22"></a>

## Extrait 22

Validation r6 :29 tests distincts du démon PASS,2 tests du protocole PASS
conservés (protocole inchangé depuis r5), aucune suite complète.
La preuve durable liste les noms exacts et les empreintes des quatre fichiers
Rust compilés : validation/canal-worker172-tests.json. Les résumés complets
des16 filtres se trouvent dans /home/moi/.cache/bridget-channel172/tests-r6.json.
La revue indépendante r6 et les paquets restent à produire à ce checkpoint.
Aucun service changé, aucun redémarrage T3 ni nouvelle mission métier.

<a id="extrait-23"></a>

## Extrait 23

Ce mécanisme de récupération existant est pertinent pour l'admission
opus158-20261021-bus-arches-reprise-v1, encore chosen sans Task après le rejeu
strict de son propriétaire. Il n'autorise ni écriture manuelle du reçu ni
nouvelle clé. Sa récupération réelle après activation n'est pas encore prouvée.
Aucun service ni T3 redémarré à ce checkpoint. Paquets et contre-revue à suivre.

<a id="extrait-24"></a>

## Extrait 24

Sourcee5195b7de80341170196bfd92def8277b2f0df3a construite une fois par plateforme, signée sur Mac,
puis active sur maître3688 et worker589872.
Aucune mission native ni commande worker active au préflight ; T3[91019, 91105, 898]
préservés avec leurs dates de naissance. T044 close sur lecture MCP officielle
du propriétaire837 et recette réellee05fd619-83fd-4056-9bea-5a42615783b0, résultat lu/ACK,
rejeu strict même Task et même résultat. cleanup_done=true, aucune garde acquise.
L'admission figée bus-arches est soldée en refus de droits, sans Task.
Cette livraison ne prétend pas relancer une mission métier.
Reçu complet :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/172-resolution-projet-distant/validation/canal-worker172.md

<a id="extrait-25"></a>

## Extrait 25

Défaut établi sur le fil mcp:f5a66bd1-7cef-4d2c-888b-06dd7d8263d3 :
la preuve T3 v2 est valide, mais T3ThreadBindingFact ne retient que les UUID.
Le contrôle human_authority_guard échoue ensuite ; le catalogue et le statut
remontent permission_attestation_unavailable avant les contrôles métier.
La tâche façades 837972ad-ad58-4512-9a71-fd2843ab5be5 est déjà failed, sans
résultat, cleanup_done=true et guard_released=true ; elle appartient à chargement.
Son accès reste réservé au propriétaire, même après cette correction.

<a id="extrait-26"></a>

## Extrait 26

Checkpoint de livraison du 22 octobre : contre-revue r2 APPROVE, 7 tests PASS,
source 31f42307 fusionnée/poussée et build 172-31f42307 actif sur le maître.
T3/pont/worker préservés ; tâches distantes inchangées, zéro reprise.
Recette du contexte opaque encore attendue à ce checkpoint. Reçu :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/172-resolution-projet-distant/validation/identifiants-t3-20261022.md

<a id="extrait-27"></a>

## Extrait 27

Livraison worker vérifiée : build172-50b0673f actif Cartae, PID2037495.
Maître/pont/T3 préservés ; processus déjà actif vérifié avant/après par naissance
et session fournisseur. Recette réelle508c947f : deux rondes, même fournisseur, ACK,
rejeu identique et garde close/cleanup_done. Sept tests isolés PASS, revue r2 approuvée.
Anciennes instances conservées, aucun nouveau travail métier, maître non redémarré.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/172-resolution-projet-distant/validation/reconnexion-wrapper-20261022.md

<a id="extrait-28"></a>

## Extrait 28

Livraison du complément de diagnostic : source 222941e6 intégrée et poussée, maître Mac 172-222941e6 actif PID 39365, binaire signé SHA-256 0b0ad4d076554f81082773360193b70f926d66a276b5c10011bd5857d99bfd46. Contre-revue r2 APPROVE sans blocage. Quatre tests ciblés PASS ; refus MCP réel sur la tâche déjà close 508c947f, texte français transmis par le client existant, payload/résultat/ronde strictement conservés. Préflight sans wrapper distant ni commande active ; T3/pont/worker conservés. Pas de FULL ni de nouvelle mission métier. Source binaire distincte du commit documentaire final. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/172-resolution-projet-distant/validation/refus-reprise-20261022.json.

<a id="extrait-29"></a>

## Extrait 29

Revue 0043865a : le worker visait encore le principal. Correction de la sonde, du Prepare et de la projection du worktree isole pour utiliser le checkout reel, sous le meme depot commun et les droits existants. Recu avant extraction pour refuser les residus incomplets. T3 refreshCheckedOutBranch et WorktreeMcpService relus, differences distantes expliquees dans le recu. Qualification finale 18 Python + 4 Rust PASS ; RED/ GREEN meme fixture. Git Cartae 2.56 durable verifie dans SSH et shell de connexion neufs, sans PATH temporaire. Contre-revue et activation a suivre.

<a id="extrait-30"></a>

## Extrait 30

Source76b682d6 fusionnee/poussee, build172-76b682d6 actif Mac68244 et Cartae2219043. T3/pont28528/28591/898 preserves. 21 Python et 4 Rust affectes PASS, revue9fdb1dae APPROVE. Recette reelle fdd9c6e9 depuis le worktree Opus2D : hostname/package/cwd/HEAD ecef0730 conformes, marqueur emis, resultat lu/ACK et recette close. Premiere recette partielle conservee, critere shell ajoute a tort retire du contrat. Git2.56 durable pour SSH/shell/worker Cartae, 12 agents Opus2D informes, mandats T3 preserves. Trois produits temporaires Mac retires ; cinq worktrees Cartae propres/fusionnes conserves sur garde lsof incomplete. Recu : validation/worktrees-lies-20261022.md et JSON voisin.

<a id="extrait-31"></a>

## Extrait 31

- Livraison du correctif différentiel : source b8406f6c intégrée/poussée, maître 172-b8406f6c actif PID64762 ; worker2219043, pont898 et T3 inchangés. Contre-revue finale494dec67 APPROVE, 24 tests PASS, export réel7,61Mo/0,504s et vérification CartaePASS sans mission Notre-Dame. Résultat envoyé au propriétaire, aucune reprise métier lancée. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/172-resolution-projet-distant/validation/bundles-divergents-20261022.md.

<a id="extrait-32"></a>

## Extrait 32

Source a4640b463e943daf8f44f540f571a85dd3279918 intégrée et poussée ; 31 tests Python ciblés PASS, contre-revue finale a0b9b462 APPROVE. Maître 172-a4640b46 actif PID76289, signature et SHA-256 vérifiés. Le worker2219043, le pont898 et T3 28528/28591 sont préservés. La mission Notre-Dame28da9844 conserve instance/génération, wrapper2350358 et fournisseur2351171 pendant le remplacement ; son résultat est ensuite disponible sans erreur. Aucun rejeu métier par ce fil. La demande de qualification auprès du propriétaire a expiré sans retour et le checkout a avancé de fd4cad3b vers559cec75 : l’ancienne enveloppe n’est pas forcée, cette reprise fournisseur reste non vérifiée. Résultat envoyé au propriétaire et à l’intégrateur, reçus initiaux en remise en vol sans lecture confirmée. Archive source de compilation et prototype Mac retirés après lsof complet ; rollback, preuves, cache Cargo et worktrees Cartae conservés. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/172-resolution-projet-distant/validation/parent-worktree-20261022.md et JSON voisin.

<a id="extrait-33"></a>

## Extrait 33

Livraison réservation : source cbc034bd intégrée/poussée, build signé172-cbc034bd actif maître13340 ; 17tests ciblésPASS et revue74f5eb18 APPROVE. RED sur même fixture, GREEN avec réservation4s et résultat/rejeu uniques. Recette réelle9d13c023 Cartae réussie, constante15s lue, marqueur reçu, ACK/rejeu strict/fermeture confirmés. Worker2219043, T3 50681/50799 et pont898 préservés ; deux missions métier gardent les mêmes identités/processus et rendent leurs résultats sans erreur. Ancienne tâche e6283066 non relancée. Les réserves de boucle de rappel et de saturation prolongée sont conservées dans le reçu ; aucune causalité historique inventée. Archive source Mac nettoyée, rollback/preuves/cache Cargo préservés ; deux worktrees Cartae conservés sur inventaire lsof incomplet. Notifications envoyées à l’intégrateur et à Monuments, reçus initiaux en remise en vol. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/172-resolution-projet-distant/validation/reservation-budget-20261022.md et JSON voisin.

<a id="extrait-34"></a>

## Extrait 34

GREEN : cinq tests ciblés de renouvellement PASS ; deux gardes de liaison T3 et deux contrôles de ronde/preuve distante PASS. La preuve distante du test de nouvelle ronde est fournie par la fixture ; ce test ne simule pas un vrai redémarrage de Codex ou un fournisseur distant. Snapshot et enfant restent inchangés, un rejeu ne crée pas de seconde ronde. Revue avant activation.

<a id="extrait-35"></a>

## Extrait 35

Source dbec83e6 intégrée et build 172-dbec83e6 actif sur le seul maître Mac.
Neuf tests ciblés PASS et revue 9e5b4b1c APPROVE. Recette Cartae 492cf951 :
deux rondes sur le même enfant, résultats lus/ACK, rejeu strict identique puis
clôture. La rotation Codex réelle reste non forcée ; variante couverte par fixtures.
T3 et worker préservés, mission becec88c inchangée pendant activation.
Ancienne admission Monuments abandonnée : refus natif sans Task, aucun rejeu.
Le refus identity_not_found d'un Codex terminal direct reste un cas distinct.
Reçu : /home/moi/10.Scripts/64.bridget/specs/172-resolution-projet-distant/validation/parent-session-renouvelee-20261022.md.

<a id="extrait-36"></a>

## Extrait 36

Livraison : source a082fa39b17019e7b7b16bd2e584a08bc163eecb intégrée
et poussée sur main. Revue indépendante APPROVE (17c96357), résultat lu,
acquitté et clôturé. Builds release optimisés Linux 56,67 s et macOS 71,26 s,
signature macOS vérifiée. Maître et worker chargent 172-a082fa39.
Annonce worker_workspace_diagnostics_v1 lue sur le socket maître ; nouvelle
connexion worker acceptée. Trois identités de processus distants conservées ;
T3 et pont conservés. Aucun worktree applicatif supprimé, aucun BUS relancé.
La réponse métier R15 n'a pas été rejouée : aucun test MCP réel de ce refus
n'est revendiqué. La projection est couverte par les tests ciblés et la revue.
Le diagnostic distingue le plafond ; il ne libère pas de place.
