# Livraison 172 — préparation autonome des projets distants

État courant du 18 octobre 2026, après continuation, reconnexion et quota : **In Progress**.
Le complément corrige aussi les défauts rencontrés lors des vraies délégations.
T3 est installé sur disque ; son activation reste à faire selon la consigne utilisateur.

## Versions actives après le complément quota

- Source commune1e5ddd076b3d610c864e5d37dbe66d28dc780edc, intégrée et poussée.
  Build172-1e5ddd07 sur Mac et Cartae. Aucun tag ni release créé par ce complément.
- Maître Mac50706 actif depuis13:56:22 CEST. Binaire signé, vérifié strict :
  /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget.
  SHA256d585bb22d60265b59133adcb451fa12ff5a9c20e687fd80744212d8537a38dd4.
  Build254,41s, RC0 ; artefact issu d’une extraction privée du commit source.
- Worker Cartae13183 actif depuis11:53:47 UTC, client aligné :
  /home/moi/.local/lib/bridget-communication/172-1e5ddd076b3d/bin/bridget.
  SHA256bc5daf985c5472e825291bb568596238463c544a2135fb367ac3168af1e2c97c.
  Build62,35s, RC0. Aucun enfant vivant et aucune tâche native non nettoyée
  au préflight de l’activation. Ce même PID13183 est resté actif pendant
  la relance du maître ; systemd NRestarts=0 après cette relance.
- Application T3637, serveur1068 et pont898 : mêmes PID et naissances08:40.
  Aucun redémarrage T3. Le paquet172 reste signé sur disque, sourcef3ddfca24278ab1ebb689087e5c42c6bc00c6551,
  ASAR4d57cee627d68079504e613cf93d532fa6399c7026a14c2b8f24fc26aa52a9b3.
  Le code T3 chargé reste171.5. Son activation attend la fin des autres conversations.
- Les modifications étrangères du Mac sont conservées. Empreinte AGENTS.md
  inchangée f0eb89de95fbd0bccb2759b51b2612d3b97e73ace41769d6e6a2f3b826a037a1.

## Continuation refusée et incident de reprise

Le port de continuation comparait le fait parent entier, dont run_id et revision.
Un nouveau tour T3 aux mêmes droits était donc refusé permission_not_inherited.
Le correctif9957714f, inclus dans1e5ddd07, n’ignore que ces deux champs et seulement
pour provider_turn. Identité, session, fournisseur, dossier, modes, politique,
révocation et preuve worker restent contrôlés. Deux tests RED puis PASS ; neuf
mutations d’identité/droits et le changement de mission native restent refusés.

L’ancien processus VOITURES54569150-ebef-4f11-9053-f17775a5c909 est perdu.
Sa ronde1, son résultat de diagnostic, son worktree et son commit de note sont conservés :
/home/moi/10.Scripts/69.opus2D/.worktrees/54569150,
c41780f4d8b27d01bda57774da27b639185bfdee. Cette note n’est pas une correction source.
La vérification de conservation des anciens processus après activation995 a échoué.
Le worker quittait sur EOF pendant sa sonde, avec worker_capability_unavailable.
Ce défaut a été reproduit puis corrigé dans527308bb, inclus dans1e5ddd07.
La reconnexion garde maintenant le même runtime ; un refus permanent reste fatal.
Le maintien du PID13183 au redémarrage du maître est prouvé sans modèle actif.
Il ne prouve pas la conservation d’un modèle en cours ni une continuation réelle.

BUS8dcb54b1-bfae-4693-afe9-596b44b375e7 a échoué api_error à11:04:18 UTC,
soit23s avant l’arrêt du maître995. Son journal Claude atteste error=rate_limit.
Aucune relance BUS par ce lot. Le propriétaire garde ses travaux.

Le propriétaire a confirmé qu’aucune consigne humaine applicable n’interdit une
nouvelle admission VOITURES. Sa reprise distincte r3 a été admise, puis a échoué :
Taskb9d8b49c-bf25-4c8b-be20-69d2fb1c128a, api_error, sans travail confirmé.
Le journal Claude donne la même limite de session, annoncée jusqu’à13:20 UTC
(15h20 Paris), et non une panne SSH ou un refus de droits. Son worktree est conservé :
/home/moi/10.Scripts/69.opus2D/.worktrees/b9d8b49c.

## Quota : correction, tests et limites

T3 conserve le code structuré assistant rate_limit quand le résultat dit api_error.
Ce mécanisme est repris dans Bridget : usage_limit, sans classer depuis le texte seul.
Une réponse ultérieure remplace la notice de limite et peut terminer normalement.
Le routage distant consulte désormais le refus lié à la même connexion, instance,
définition et époque. Un refus récent reste restrictif après nettoyage de l’enfant,
pendant le seul TTL de la politique. Aucun crédit positif d’une instance morte
n’est repris. Sans autre cible compatible, le refus est provider_quota_exhausted.
Aucune date de retour, fenêtre ou consommation n’est inventée depuis le texte humain.
Un fait périmé redevient inconnu ; aucun hold160 sans heure de retour attestée.

65tests concernés PASS : pilote Claude34, mémoire quota14, routage13,
sonde distante1, liaison stricte1, continuation2. Les14tests de rondes hérités
passent aussi après correction du profil temporaire0770→0700. Les deux anciens
échecs env_unfit avaient été reproduits sur la baseline ; contrôle produit inchangé.
Aucune FULL ni validation universelle déduite. Autres serveurs que Cartae non testés.

## Essais métier après activation1e5 et complément en préparation

r4 Task14c382ea-c831-4374-b6b8-c9f77fe456a7 et r5 Taskc9693bca-4304-4e72-abe2-442677bc8c14
ont toutes deux fini usage_limit, puis ont été lues et acquittées. Pas de working
confirmé ni de code produit. Les décisions ont choisi claude-cartae / claude-haiku-5-5.
Le refus est mieux classé, mais le routage n'est pas débloqué par1e5 : le délai
r4→sonde r5 de44–47secondes dépasse le TTL30. Aucun succès de routage déduit.

Le complément source courant corrige cette perte. Un refus daté reste restrictif
jusqu'au retour structuré. Un refus sans date conserve un délai de nouvelle tentative
de5minutes ; il redevient ensuite inconnu, sans capacité positive inventée.
Une vraie réponse assistant acceptée peut le lever avant ce délai. Les comptes
restent séparés par connexion, définition de base, politique et identité worker.
Les autres modèles du même compte sont écartés. Crédits et faits positifs gardent
les contrôles du digest de mission, de présence vivante et du TTL. Une réponse
vide ou une autre erreur ne lève pas le refus. Les fenêtres structurées gardent
leur vraie date, sans ajout d'un refus unknown permanent par le code assistant.

Tests finaux concernés : mémoire quota16PASS, sonde distante1PASS, pilote Claude35PASS
(en série). Diagnostic du rattachement2PASS, liaison stricte1PASS, héritage/revocation
auxiliaire1PASS, scope de connexion1PASS et continuation2PASS. Routage168 :35PASS.
Les séries peuvent se recouvrir ; elles ne sont pas additionnées comme une FULL.
La série Claude parallèle a eu34PASS/1FAIL Text file busy ; ce cas a été relu,
rejoué seul PASS, puis la série entière exécutée en série35PASS. Cause système
ponctuelle non établie ; aucun changement produit destiné à masquer ce cas.

Une mission indépendante est désormais working sur Cartae : Taskc998d2d5-50b2-4ec8-ac51-e0d32767e02f,
clé opus2d-vehicules-cartae-sol61-20261018-r1, propriétaire trajets bus, Codex/gpt-6.1-sol.
Dossier /home/moi/10.Scripts/69.opus2D/.worktrees/c998d2d5,
base d97a3db4a6015767c391a83cc86fff5836533c21. Elle ne qualifie pas le nouveau
routage automatique Claude ; aucun doublon ni reprise r6/r7 lancé par ce lot.
La préparation des binaires peut continuer. L'activation attend un préflight sans
mission native non nettoyée ; aucun arrêt du maître ou du worker pendant ce travail.
La continuation réelle sur un enfant gardé et l'activation T3 restent à qualifier.

## Refus de rattachement après Claude

Le fil17f50a6d-ec2f-494a-b9a3-0def5b14475b a rencontré usage_limit/api_error_429
sur Claude (tours169–172), avec retour structuré13:20UTC/15:20Paris. Le message
binding_registration_refused est apparu dans la reprise Codex (tour173,11:56:05UTC).
Aucune limite de taille de contexte n'est établie par ces traces. Le rattachement
local est distinct du quota. Son lien actuel est présent dans l'inventaire.
L'unique demande d'une lecture depuis ce fil n'a pas obtenu de réponse dans son
échéance ; ce n'est pas une preuve de succès ou de refus actuel.

Le complément distingue cinq motifs fermés, sans changer les gardes ni imprimer
de preuve privée. Les vieux clients MCP peuvent garder le libellé générique ;
le maître journalise la cause utile. Ce diagnostic n'est pas présenté comme une
cause historique établie ni comme une correction fonctionnelle déjà validée.
Aucun redémarrage T3 ; son paquet172 reste prêt sur disque.

## Paquets du complément prêts, activation différée

Source codebb83316c68b828fe4ae0c747d81830ac34ee9eea, publiée sur main.
Build172-bb83316c, sans nouvelle release ni changement de version commerciale.
93noms de tests distincts PASS dans9séries ciblées, dédupliqués dans le reçu privé ;
aucune FULL. Le correctif de quota a d'abord reproduit le défaut en RED.

- Artefact Mac : /Users/moi/.cache/bridget-delivery172/quota-delay172/bridget.
  SHA256d5281180383ddaf9d886de276a52c93084a06c3ae9c7482c54cb0a16f40ac9ea.
  Compilation71,81s RC0, signature ad hoc vérifiée strictement.
- Artefact Cartae : /home/moi/.cache/bridget-delivery172/quota-delay172/bridget.
  SHA2564a6164d0bdce8b58c033dd079c2f1294b6c1ba1e580625897e666f51d4117ead.
  Compilation64,82s RC0. Copie privée vérifiée, distincte du cache de compilation.

Le préflight à1792326544 conserve l'activation différée : la Taskc998d2d5 est
working, cleanup_done=false. Aucun binaire actif remplacé, aucun service relancé.
Maître50706, worker13183, T3637/1068 et pont898 conservés. Le statut officiel
annonce toujours172-1e5ddd07, en ligne. Le propriétaire a reçu une demande unique
de signalement après clôture complète, sans polling ni GO pour son travail.
La remise de cette note est in_flight ; aucune réponse ou lecture n'est supposée.

Reçus Mac :
/Users/moi/.cache/bridget-delivery172/quota-delay172/build-receipt.json
/Users/moi/.cache/bridget-delivery172/quota-delay172/activation-preflight.json
/Users/moi/.cache/bridget-delivery172/quota-delay172/CURRENT.json
Reçus Cartae :
/home/moi/.cache/bridget-delivery172/quota-delay172/build-receipt.json
/home/moi/.cache/bridget-delivery172/quota-delay172/tests.json

Ces paquets ne constituent pas une activation ni une preuve du nouveau parcours
réel. La prochaine intervention vérifie d'abord la clôture native et les admissions
en cours. Elle conserve la consigne d'attendre la fin des conversations avant T3.
La source documentaire ultérieure reste distincte de la source compilée bb83316c.

## Reçus privés du complément

/Users/moi/.cache/bridget-delivery172/quota172/build-receipt.json.
/Users/moi/.cache/bridget-delivery172/quota172/activation.json.
/home/moi/.cache/bridget-delivery172/quota172/build-receipt.json.
/home/moi/.cache/bridget-delivery172/quota172/activation.json.
/home/moi/.cache/bridget-delivery172/quota172/after-master.json.
Les anciens checkpoints ci-dessous restent historiques.

## Checkpoint124 — historique

- Bridget source124dc47a9fe2ad2717d06c53053b71d9f0f7dc26, intégrée et poussée.
- Client Mac installé : build172-124dc47a, signé, SHA256
  d960568d84d5a073cc08c44b62bfda5ee4b2b8f88160ba8fc806df7457a06f9b.
  /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget.
- Maître Mac81018 actif depuis11:53:40 CEST : build172-124dc47a. L’ancien
  maître51581 a été identifié puis arrêté par SIGTERM, sans mission native active.
- Worker Cartae4049619 actif depuis11:53:52 CEST, build172-124dc47a.
  /home/moi/.local/lib/bridget-communication/172-124dc47a9fe2/bin/bridget.
  SHA256cd62364093cd85820af50ac1af21f08c9879eaa1c15ecaa25aef1f6abc375569.
  Client /home/moi/.local/bin/bridget aligné. Aucune tâche native active au préflight.
- T3 source f3ddfca24278ab1ebb689087e5c42c6bc00c6551, intégrée et poussée
  vers le fork. Paquet0.0.45-local.172 signé installé dans
  /Applications/T3 Code (Local).app.
  ASAR4d57cee627d68079504e613cf93d532fa6399c7026a14c2b8f24fc26aa52a9b3.
  Application637, serveur1068 et pont898 préservés. Leur code T3 chargé reste
  la version171.5 ; ce n’est pas une preuve d’activation172.
- Le profil cx-pro pointe sur le même fichier que le profil Codex :
  /Users/moi/.codex/config.toml.
  Un seul champ du serveur officiel Bridget changé : tool_timeout_sec30→3700.
  Tous les autres champs comparés identiques, sauvegarde privée conservée.

## Recettes complémentaires prouvées

- Parent natif Cartae → enfant Cartae, sans service T3 ni project_id :
  request recette172-parent-natif-cartae-20261018-r3 ;
  Task parent c5d2017b-2e2e-47c7-8168-49daeb476ee0 ;
  Task enfant0caa190a-0fb4-4c0e-a493-254137804e24.
  Vrais modèles Haiku sélectionnés automatiquement. Lecture réelle du Git et
  de Cargo.toml au SHAefeeb08f, version0.4.1, marqueurs parent/enfant exacts.
  Chaque résultat lu/ACK ; rejouage strict, mêmes Tasks et mêmes résultats.
  Deux nettoyages terminés. Ce parcours remplace la limite native du premier reçu.
- Codage réel sur Cartae dans un worktree isolé :
  request recette172-codage-bridget-cartae-20261018-r1 ;
  Task e125950f-9ceb-4dc3-92e6-2841e0566f05 ;
  /home/moi/10.Scripts/64.bridget/.worktrees/e125950f.
  Un test ajouté à tests/test_spec172_project_preparation.py, 27lignes.
  python3 -m unittest -v tests/test_spec172_project_preparation.py :10PASS.
  HEADefeeb08f avant/après, aucun commit ni service modifié. Marqueur
  BRIDGET172_CODAGE_CARTAE_OK, résultat complet4676octets lu via l’outil
  officiel, ACK puis rejeu même Task/résultat. Worktree sale conservé, cleanup_done=true.
  Le test possède un contrôle positif de hook. Il ne prouve pas à lui seul
  que core.hooksPath est la protection active ; cette limite est conservée.

## Échecs réels qui ont déclenché le complément

- Deux lectures du résultat de codage ont d’abord expiré avec budget JSONL dépassé.
  La récupération ultérieure a réussi, sans présenter cette réussite comme un fix.
  Les piles du daemon ont ensuite montré l’attente de réattestation HTTP T3.
  Cinq cas RED avant correction ; budget complet réservé après correction.
- Revue r6 : premier appel et reprise ont expiré à30s dans l’hôte Codex, sans Task.
  Aucun reçu durable trouvé pour cette clé au contrôle. Une nouvelle reprise
  strictement identique a été demandée après installation, sans nouvelle clé.
- Opus2D r2 : Task b00ae037-ed93-4106-9e4b-68fbee5af3d2 admise, puis
  failed/remote_start_failed:permission_attestation_unavailable.
  Lecture et ACK de cet échec demandés au parent propriétaire ; recette r3
  distincte demandée après correction. Aucun succès Opus2D annoncé à ce checkpoint.

## Validation ciblée du dernier delta

- Neuf cas172 PASS, dont réponse volumineuse tardive, refus tardif et même clé au rejeu.
- Client Unix :19PASS ; façade de délégation :19PASS (ensembles partiellement communs).
- HTTP/attestation :8PASS, droits refusés et redirections toujours fermés.
- Contrats d’attente MCP :16PASS/1 ancien oracle qui assimilait une socket
  absente à unsupported ; oracle corrigé et rejoué seul1PASS.
- Montage Codex T3 :42PASS ; format et lint des deux fichiers conformes,
  zéro erreur et zéro avertissement. Deux cas nouveaux RED puis GREEN.
- Les tests de conservation des droits T3 du complément précédent restent
  prouvés séparément : trois nouveaux cas, dix goals et onze permissions,
  contrôle de21cas sur les overlays171 conservés. Aucune nouvelle FULL.
- Build Linux101,11s et build Mac142,30s, tous deuxRC0. Deux liens documentaires
  node_modules historiques exclus de l’extraction privée Mac ; sources compilées intactes.
- Aucun autre serveur que Cartae testé. Les recettes finales des parents
  Revue/Opus2D sont réussies ci-dessous. Activation T3 et tours automatiques restent ouverts.

## Reçus du complément

/Users/moi/.cache/bridget-delivery172/coding-model-final-task.json.
/Users/moi/.cache/bridget-delivery172/native-model-final-tasks.json.
/Users/moi/.cache/bridget-delivery172/mac172-worker-probe.json.
/Users/moi/.cache/bridget-delivery172/t3-install/bounded-attestation-install.json.
/Users/moi/.cache/bridget-delivery172/config-timeouts/receipt.json.
/home/moi/.cache/bridget-delivery172/linux172-worker-probe.json.
Journaux de tests dans /home/moi/.cache/bridget-delivery172.
Aucune base modifiée par les diagnostics. AGENTS.md étranger du Mac préservé.
Correction technique, sans changement d’interface : pas de diaporama requis.

## Complément de sondes activé

Les défauts de réponse tardive, déconnexion V2 et verrou worker ont leur preuve
RED/GREEN ; détails et logs dans implementation.md. Worker21PASS, protocole9PASS,
client19PASS, banc de sonde retardée1PASS. Le lot initial de routage35 a donné
28PASS/7FAIL ; oracle corrigé1PASS et Laya7PASS en reprises séparées.
Le correctif T3 de lecture bornée ajoute16PASS, sans retirer les contrôles de droits.
Les sources124dc47a sont compilées et actives sur le maître et le worker.
Le nouveau paquet T3 reste installé sur disque sans activation.
Revue r6 a désormais rendu expected_base_commit_mismatch ; Opus2D r3 a rendu
worker_unavailable. Ces refus ne sont pas transformés en succès. Après activation du worker, les nouvelles recettes Revue r7 et Opus2D r4 ont
réussi sur leurs SHA courants. Elles restent distinctes des anciennes demandes refusées.

## Recettes après activation124dc47a

Le banc sans T3 a été rejoué sur le dernier code :1test/2cas PASS en25,57s,
avec vrai daemon/worker/wrapper et fournisseur simulé. Pas de service T3.
Les missions réelles de codagee125950f et parent natifc5d2017b ont été relues
par rejeu strict après redémarrage du maître, alors que HEAD était passé à124dc47a.
Mêmes Tasks, résultats exacts et ACK conservés ; aucun nouveau lancement.
Base Bridget quick_check=ok. T3 et pont ont leurs mêmes PID et dates de naissance.

Une nouvelle mission réelle a ensuite été lancée sur Cartae :
request recette172-bridget-cartae-apres-sondes-20261018-r1 ;
Task a62d8fb1-9fef-4c92-9d80-0d1e1b475790, modèle Haiku choisi automatiquement.
Sans project_id, shared/discovery ; lectures .git/HEAD, refs/heads/main et Cargo.toml
sous /home/moi/10.Scripts/64.bridget ; SHA124dc47a, version0.4.1,
BRIDGET172_SONDES_CARTAE_OK. Résultat833octets lu via task_status, ACK, puis
rejeu strict : mêmes Task et résultat, acknowledged. Aucun fichier modifié.
La trace du worker mesure4,426s pour le premier lot de dix candidats, puis
1,431s et1,419s pour les deux recontrôles. L’ancienne borne5s avait peu de marge
même sur ce projet. Ces durées sont observées, pas extrapolées à Opus2D.
Preuve privée complète :
/Users/moi/.cache/bridget-delivery172/final-runtime-worker-probe172.json.
Cette recette part de T3 encore ancien. Elle prouve Bridget actuel, pas le
correctif T3 de continuation automatique ni sa nouvelle lecture bornée.

## Revue et Opus2D : recettes finales réussies

Les deux appels partent des vrais parents demandés par l’utilisateur, sans
project_id, autres axes automatiques ; shared/discovery, échéance180s.

Revue : request recette172-revue-cartae-controle-final-20261018-r7,
Task30a2525b-7d62-4be8-a3a6-b5b750382009.
SHA attendu et lu d9d29fe69c9969af913cb99ce70fd249fc723b67,
nom revue/version0.1.0, BRIDGET172_REVUE_FINAL_OK.
Lectures réelles :
/home/moi/.local/share/bridget-projects/auto-5c29175617db400dab394fa74bc97948/repository/package.json
/home/moi/.local/share/bridget-projects/auto-5c29175617db400dab394fa74bc97948/repository/.git/HEAD.
HEAD détaché, pas de référence supplémentaire. Le projet initialement absent
avait été créé automatiquement ; son dépôt géré a reçu le nouveau commit.
Le parent Rapport a lu le résultat, ACK, puis comparé le rejeu : même Task et
texte identique. Aucun travail de code demandé à l’enfant ; la préparation Git
reste l’action du framework.

Opus2D : request recette172-opus2d-trajets-cartae-20261018-r4,
Taskea8d9680-dd4e-4a71-a22d-1589ee028f9f,
SHA attendu et lu ed82547ff65fd23307f63ee6683c9e9e7b22d420,
nom opus2d/version0.1.0, BRIDGET172_OPUS2D_CARTAE_OK.
Lectures réelles :
/home/moi/10.Scripts/69.opus2D/package.json
/home/moi/10.Scripts/69.opus2D/.git/HEAD
/home/moi/10.Scripts/69.opus2D/.git/refs/heads/main.
Le parent trajets bus a lu le résultat1007octets, ACK, puis comparé son unique
rejeu : même Task, même résultat textuel. Aucune modification demandée à l’enfant.

Les sorties réelles worker sont exit_code0/failure=null ; générations557/558.
Sondes initiales observées : Revue4,317s ; Opus2D4,303s pour dix candidats.
SQL en lecture seule confirme pour les six clés réussies : une Task chacune,
result_available, error=null, cleanup_done=true, ACK présent. Les rapports
parents sont conservés dans :
/Users/moi/.cache/bridget-delivery172/project-final-recipes172.json.
Les deux rapports transportés sont bornés et leur fin est tronquée ; les
résultats natifs complets et leurs états durables ont été relus séparément.

Le correctif T3f3ddfca2 reste signé/installé sur disque et non activé. Les
recettes ci-dessus prouvent le parcours standard depuis T3 courant. Elles
ne prouvent pas encore la correction des tours automatiques dans le nouveau
runtime T3. La consigne utilisateur d’attendre les autres conversations reste
respectée ; aucun redémarrage de T3 n’est programmé.

## Liaison intermittente après les recettes finales

Le fil trajets bus a signalé le18octobre à10:08:13UTC un refus de
bridget_capabilities : t3_session_unavailable/binding_transport_unavailable,
avant toute mission métier. Ce refus reste un échec réel, distinct de la recette r4.
Le nouvel appel depuis le même fil a réussi à10:21:26UTC en17,759s.
La mission métier opus2d-bus-regressions-cartae-20261018-r1 a ensuite été
admise sous Task8dcb54b1-bfae-4693-afe9-596b44b375e7 puis observée working,
error=null, avec started_at1792319001. L’espace isolé est :
/home/moi/10.Scripts/69.opus2D/.worktrees/8dcb54b1,
base d01791b5b61d081dffde6ff1d234c67f5710be62.
Le parent reste responsable des missions bus/voitures ; aucune mission en doublon.
Ce démarrage ne prouve ni leur résultat métier ni la résolution définitive de
l’incident intermittent. Aucun nouveau correctif ni redémarrage appliqué pour
ce nouvel essai. Une capture du démon hors incident montre ses connexions au
repos, sans verrou bloqué ; elle ne suffit pas à identifier la cause historique.
La demande de retour mcp-16258-6ad49d80-7 a expiré pendant le travail du parent ;
la reprise du catalogue et l’admission ont été vérifiées dans son activité T3,
puis l’état de la Task a été lu séparément. Aucun silence transformé en succès.
L’utilisateur a explicitement choisi d’attendre la fin des autres conversations
avant de redémarrer T3. Le défaut de latence et les tours automatiques restent
à qualifier après activation du paquet déjà installé. Pas de relance programmée.

## Mission VOITURES : refus réel puis reprise corrélée

À10:24:04UTC, opus2d-voitures-route-cartae-20261018-r1 a rendu
permission_attestation_unavailable dans le tour T3847, sans Task ni reçu
automatique d’admission. BUS8dcb54b1 est restée working, sans erreur.
L’hypothèse du tour848 en file a été écartée : ThreadManagementService.isActiveRun
exclut queued. La source T3 de sessionIdentity et du fait de droits a été lue ;
le refus générique du daemon ne suffit pas à isoler sa cause exacte.
Un nouveau tour du même parent a relu le catalogue en4,261s : development et
worktree autorisés. Une seule reprise a ensuite utilisé la même clé et tous
les arguments d’origine, sans changement : admission52,195s, puis état working
relu en1,391s. Task54569150-ebef-4f11-9053-f17775a5c909, error=null, sur :
/home/moi/10.Scripts/69.opus2D/.worktrees/54569150.
Base d01791b5b61d081dffde6ff1d234c67f5710be62. La comparaison du reçu automatique
avec l’appel d’origine confirme zéro argument changé et une seule Task.
La demande suivie mcp-16258-6ad49fed-9 a reçu le rapport complet, état answered.
Aucun nouveau correctif ni service modifié, aucune interruption de BUS.
Les deux missions métier poursuivent sous leur responsable. Leur lancement
ne prouve pas encore leurs résultats métier, ni la correction définitive de
l’attestation intermittente. T3 reste non redémarré selon la consigne utilisateur.
Preuves privées :
/Users/moi/.cache/bridget-delivery172/cars172-attestation.json
/Users/moi/.cache/bridget-delivery172/cars172-resolution.json.

## Premier reçu conservé pour l’historique

Les états et chemins ci-dessous décrivent le premier lot, avant les compléments
ci-dessus ; ils ne décrivent pas les processus courants.

Date : 18 octobre 2026. Correction active, recettes Cartae réussies.
Périmètre : résolution implicite du projet, préparation Git/SSH, démarrage et retour.

## Résultat utile

Un appel Cartae depuis un dépôt Git local n'exige plus un project_id connu
ni une correspondance préparée manuellement. Bridget retrouve la racine du
parent, transporte le commit exact et crée la correspondance après préparation.
Le transport utilise la fédération SSH existante. GitHub n'est pas nécessaire :
un bundle Git peut contenir un commit non publié. Un transfert ultérieur
importe seulement les objets manquants quand un delta est possible.
La préparation appartient à Bridget ; elle n'appelle pas T3.

T3 fournit la référence de parcours (héritage du projet, préparation, lancement).
Il ne fournit pas le transfert Mac vers Cartae. Références et différences :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/172-resolution-projet-distant/plan.md.

## Versions du premier reçu (historiques)

- Maître Mac : a53b05c23426160dd34c5987613e7d0d878e273f, build172-a53b05c2,
  PID53473, actif depuis 09:59:02 CEST.
  Binaire : /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget.
  SHA256 : 8451acbd43c563e5ee05fcc851cd06bb213ed669bd5360afe6616b2bba73fc9f.
  Signature ad hoc vérifiée.
- Worker Cartae : 780d5df4ebfd5bb72f38fce509310b5ebb2523c9,
  build172-780d5df4, PID3893226, actif depuis 10:06:07 CEST.
  Binaire : /home/moi/.local/lib/bridget-communication/172-780d5df4ebfd/bin/bridget.
  SHA256 : f4e5bf79b6a903178a4d01f3b8f2a5eeeed26aa98979cce24e865c6a853ee8e2.
- Le dernier delta concerne l'enregistrement du wrapper distant ; il est
  exécuté sur Cartae. Le maître Mac a53 reste suffisant pour ce parcours.
- T3 PID637/1068 et pont898 conservés. Les anciens clients MCP déjà en mémoire
  conservent leur code jusqu'à leur prochaine ouverture ; aucun arrêt imposé.
- Aucun schéma SQL ou droit de fournisseur modifié. AGENTS.md local étranger préservé.

## Recettes réelles

### Projet Bridget connu

Request : recette172-bridget-cartae-connu-20261018-r3.
Task : 68f69ce3-62bb-4b17-8bbc-853399a6346f.
Sans project_id, machine Cartae, posture discovery, espace partagé.
Lecture réelle de /home/moi/10.Scripts/64.bridget/.git/HEAD, de sa référence
et de /home/moi/10.Scripts/64.bridget/Cargo.toml.
SHA a53b05c23426160dd34c5987613e7d0d878e273f, version0.4.1.
Résultat : BRIDGET172_CARTAE_CONNU_OK.
Résultat lu, ACK, rejeu strict : même tâche, même résultat.
État durable result_available, error=null, cleanup_done=true, un lancement.

### Projet Revue initialement absent

Source : /Volumes/8TB2/50-repos-archives/11.Repositories/revue.
Aucune déclaration manuelle avant la recette r1.
La préparation r1 a créé automatiquement :
/home/moi/.local/share/bridget-projects/auto-5c29175617db400dab394fa74bc97948/repository.
Le premier lancement et son résultat ont confirmé la création ; son brief
demandait par erreur un shell interdit en discovery, donc la lecture finale
n'était pas validée à ce stade.

Recette finale :
Request : recette172-revue-cartae-neuf-20261018-r5.
Task : 800dae46-f45a-4e7c-bc51-9d1e7dabba7d.
SHA ac8f9295b89d1475f32af85f9e3e9d12d3f89008 transféré automatiquement.
Ce commit a remplacé le SHA initial pendant les essais à la suite d'une fusion
faite par une autre session. L'appel r4 avec l'ancien SHA a été correctement refusé.
Fichiers lus :
/home/moi/.local/share/bridget-projects/auto-5c29175617db400dab394fa74bc97948/repository/.git/HEAD
/home/moi/.local/share/bridget-projects/auto-5c29175617db400dab394fa74bc97948/repository/package.json
Nom revue, version0.1.0, SHA exact, BRIDGET172_REVUE_NEUF_OK.
Résultat lu et acquitté par le parent Revue. Rejeu strict : même tâche,
même résultat, acknowledged. État durable result_available, error=null,
cleanup_done=true, un lancement.
Aucun fichier Revue modifié par les agents de recette.
La création initiale et la lecture finale sont deux essais distincts, pas
présentés comme un seul lancement ininterrompu.

## Défauts révélés et corrigés pendant les essais

1. Identifiant projet absent : résolution unique depuis le dossier attesté.
2. Projet absent : préparation Git/SSH et publication atomique des registres.
3. Réattestation après Ready plus longue que cinq secondes :
   bail de60s négocié ReleaseLeaseV2, ancien worker limité à5s ;
   contrôles de droits/révision/époque conservés.
4. Enregistrement du wrapper limité à1s : échec réel os error11 reproduit
   avec Registered retardé de1,5s. Attente initiale10s puis lecture de session1s.

Les échecs historiques sont conservés dans le journal d'implémentation.
Aucun refus de droits ou de commit n'a été transformé en réussite.

## Vérifications et limites

- Préparation autonome Python : 9 PASS, vrais dépôts Git et SSH simulé, sans T3.
- Résolution/routage : 2 PASS172, 35 PASS168, 1 PASSincident171.
- Délais : 3 PASS172, 8 PASST3MCP, 2 PASSrécupération admission.
- Worker : 21 PASS en série ; première exécution parallèle20PASS/1 verrou
  occupé. Rejeu exact PASS. Cause de concurrence non corrigée ici.
- Protocole : 9 PASS ; négociation ancienne/nouvelle capacité : 1 PASS.
- Enregistrement/reconnexion : 69 PASS/1 ignoré ; reproduction RED puis GREEN.
- Pas de suite globale, pas de succès global déduit.
- Test supplémentaire d'un parent natif hors T3 :
  recette172-parent-natif-autonomie-20261018-r1 refusée resource_saturated
  sur la machine locale, sans reçu ni tâche. Aucun repli ni changement de
  seuil. Ce parcours complet reste non validé en réel. Les recettes modèles
  ci-dessus partent de parents T3 ; la préparation autonome est testée séparément.
- Nouveaux projets Git seulement. Les fichiers non committés ne sont pas
  transférés. Les projets non Git déjà déclarés gardent leur parcours existant.
- La fédération SSH, Python3 et Git doivent déjà être installés.
- Correction technique : aucun changement d'interface, pas de diaporama requis.

## Reçus et conservation

Reçus privés Mac : /Users/moi/.cache/bridget-delivery172.
État final des tâches :
/Users/moi/.cache/bridget-delivery172/final-task-receipts.json.
Reçu worker :
/home/moi/.cache/bridget-delivery172/linux172-register.json.
Ancien binaire conservé :
/Users/moi/.cache/bridget-delivery172/bridget-before172.
Unités et binaires précédents conservés sur Cartae ; aucun effacement de base.
Worktree de correction conservé :
/home/moi/10.Scripts/64.bridget/.worktrees/172-resolution-projet-distant.


## Complément de preuve d'autonomie
Le banc natif complet est désormais PASS : deux scénarios avec vrai maître,
worker et wrapper, fournisseur simulé, sans service T3. La demande automatique
omet project_id ; le dossier et le commit sont vérifiés, puis le résultat et
l'unicité du rejeu. Le second scénario retarde Ready de6s et conserve le bail60s.
Le banc historique garde son protocole antérieur et ses quatre cas passent.
Commandes exécutées sur Cartae :
cargo test -p bridget-daemon --features test-support --test native_delegation_e2e spec172_parent_natif_auto_sans_t3_resout_projet_et_attend_ready -- --nocapture
cargo test -p bridget-daemon --features test-support --test native_delegation_e2e remote162_e2e_delegate_maitre_reel_resultat_unique_cwd_et_modele -- --nocapture
Logs privés :
/tmp/bridget172-native-e2e.log
/tmp/bridget172-native-legacy-e2e.log
La recette avec modèle parent natif réel reste distincte et non validée.
Le tour automatique a aussi rencontré permission_attestation_unavailable
dans l'adaptateur T3, avant toute admission ; aucun contournement effectué.


## Correctif du store distant préparé le 18 octobre 2026

Incident réel : remote_result_slot:StoreUnavailable sur la demande voitures r7.
Cause confirmée par lecture seule du store Cartae : 612964 octets réels contre
15788644 octets réservés. Les anciens reçus soldés retenaient chacun 64 Kio.
La Task350d5c53-3b0d-4b1e-90b2-150002b6f51a avait pourtant été admise ;
son identifiant a été transmis au propriétaire pour consultation de l’existant.

Trois tests RED prouvent le défaut avant correction. Après correction :
série172 23 PASS ; module worker 24 PASS ; résultats distants 46 PASS ;
routage168 35 PASS. Ces séries se recouvrent :
125 noms de tests distincts PASS, sans suite complète.
Reçu privé : /home/moi/.cache/bridget-delivery172/store-admission172-tests.json.
Les tests sont en série, avec un seul thread, sur Cartae. Une première commande
sans dossier Cargo et une erreur de syntaxe de fixture n’ont lancé aucun test ;
la première compilation du séparateur typé a nécessité de distinguer les erreurs
techniques des erreurs du store. Les trois échecs RED produits restent séparés
de ces erreurs de préparation du banc.

Le store conserve 120 résultats et reçus, leur relecture après réouverture et le
rejeu du premier. Sept résultats actifs restent possibles et le huitième est
refusé. Les gardes des orphelins, des liens symboliques, des identités, des
permissions et des rejeux passent dans les régressions. La réponse après
admission garde désormais TaskID et erreur ; une modification d’enveloppe reste refusée.

Aucun résultat réel sur ce nouveau binaire n’est encore revendiqué.
Paquets à préparer avant activation. BUS453c75be-503b-4934-9311-e382733498aa
et les processus T3/Bridget sont préservés. Aucun redémarrage n’est programmé.


Les deux paquets sont désormais construits depuis la source
72a8e8f8823c5f23b463a45693bb9b27079e41f3, build172-72a8e8f8, sans activation.
Mac : compilation76,28s, signature adhoc et vérification stricte conformes.
Binaire /Users/moi/.cache/bridget-delivery172/store172/bridget,
SHA256 2d9a543ade02f12e669d6a94e45ccc2cae8c843067ff00294712ef5f82d04485.
Cartae : compilation99,96s,
binaire /home/moi/.cache/bridget-delivery172/store172/bridget,
SHA256 2d2df02dacb28e72cf25273f5e7d558ab2ed25e0c5add5029588b28b459b6a92.
Le paquet intègre aussi le complément quota bb83316c préparé précédemment.

Reçu courant : /Users/moi/.cache/bridget-delivery172/store172/CURRENT.json.
Pointeur de préparation : /Users/moi/.cache/bridget-delivery172/prepared-current.json.
Le paquet quota-delay172 est signalé comme remplacé par cette préparation.

Le dernier contrôle en lecture seule trouve encore BUS453c75be-503b-4934-9311-e382733498aa
working, cleanup_done=false. La garde interdit donc l’activation.
Maître50706, worker13183 avec NRestarts0, pont898 et T3637/1068 inchangés.
Le binaire actif Mac garde le SHA d585bb22d60265b59133adcb451fa12ff5a9c20e687fd80744212d8537a38dd4.
Le binaire actif Cartae garde le SHA bc5daf985c5472e825291bb568596238463c544a2135fb367ac3168af1e2c97c.
La source active reste1e5ddd076b3d610c864e5d37dbe66d28dc780edc.
L’objectif reste paused et aucun réveil ni redémarrage automatique n’est programmé.

T037/T038/T039 sont closes pour le code, ses tests et ses paquets.
T036 reste ouverte pour l’activation puis le parcours réel. Aucun succès métier
sur cette nouvelle source n’est déduit de ses tests ou de sa compilation.
