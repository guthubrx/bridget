# Archives techniques de l’adaptateur T3

Document d’origine : [specs/172-resolution-projet-distant/validation/livraison172.md](../../../../../../../specs/172-resolution-projet-distant/validation/livraison172.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

État courant du 18 octobre 2026, après continuation, reconnexion et quota : **In Progress**.
Le complément corrige aussi les défauts rencontrés lors des vraies délégations.
T3 est installé sur disque ; son activation reste à faire selon la consigne utilisateur.

<a id="extrait-2"></a>

## Extrait 2

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

<a id="extrait-3"></a>

## Extrait 3

Le port de continuation comparait le fait parent entier, dont run_id et revision.
Un nouveau tour T3 aux mêmes droits était donc refusé permission_not_inherited.
Le correctif9957714f, inclus dans1e5ddd07, n’ignore que ces deux champs et seulement
pour provider_turn. Identité, session, fournisseur, dossier, modes, politique,
révocation et preuve worker restent contrôlés. Deux tests RED puis PASS ; neuf
mutations d’identité/droits et le changement de mission native restent refusés.

<a id="extrait-4"></a>

## Extrait 4

T3 conserve le code structuré assistant rate_limit quand le résultat dit api_error.
Ce mécanisme est repris dans Bridget : usage_limit, sans classer depuis le texte seul.
Une réponse ultérieure remplace la notice de limite et peut terminer normalement.
Le routage distant consulte désormais le refus lié à la même connexion, instance,
définition et époque. Un refus récent reste restrictif après nettoyage de l’enfant,
pendant le seul TTL de la politique. Aucun crédit positif d’une instance morte
n’est repris. Sans autre cible compatible, le refus est provider_quota_exhausted.
Aucune date de retour, fenêtre ou consommation n’est inventée depuis le texte humain.
Un fait périmé redevient inconnu ; aucun hold160 sans heure de retour attestée.

<a id="extrait-5"></a>

## Extrait 5

Une mission indépendante est désormais working sur Cartae : Taskc998d2d5-50b2-4ec8-ac51-e0d32767e02f,
clé opus2d-vehicules-cartae-sol61-20261018-r1, propriétaire trajets bus, Codex/gpt-6.1-sol.
Dossier /home/moi/10.Scripts/69.opus2D/.worktrees/c998d2d5,
base d97a3db4a6015767c391a83cc86fff5836533c21. Elle ne qualifie pas le nouveau
routage automatique Claude ; aucun doublon ni reprise r6/r7 lancé par ce lot.
La préparation des binaires peut continuer. L'activation attend un préflight sans
mission native non nettoyée ; aucun arrêt du maître ou du worker pendant ce travail.
La continuation réelle sur un enfant gardé et l'activation T3 restent à qualifier.

<a id="extrait-6"></a>

## Extrait 6

Le complément distingue cinq motifs fermés, sans changer les gardes ni imprimer
de preuve privée. Les vieux clients MCP peuvent garder le libellé générique ;
le maître journalise la cause utile. Ce diagnostic n'est pas présenté comme une
cause historique établie ni comme une correction fonctionnelle déjà validée.
Aucun redémarrage T3 ; son paquet172 reste prêt sur disque.

<a id="extrait-7"></a>

## Extrait 7

Ces paquets ne constituent pas une activation ni une preuve du nouveau parcours
réel. La prochaine intervention vérifie d'abord la clôture native et les admissions
en cours. Elle conserve la consigne d'attendre la fin des conversations avant T3.
La source documentaire ultérieure reste distincte de la source compilée bb83316c.

<a id="extrait-8"></a>

## Extrait 8

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

<a id="extrait-9"></a>

## Extrait 9

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

<a id="extrait-10"></a>

## Extrait 10

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

<a id="extrait-11"></a>

## Extrait 11

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

<a id="extrait-12"></a>

## Extrait 12

/Users/moi/.cache/bridget-delivery172/coding-model-final-task.json.
/Users/moi/.cache/bridget-delivery172/native-model-final-tasks.json.
/Users/moi/.cache/bridget-delivery172/mac172-worker-probe.json.
/Users/moi/.cache/bridget-delivery172/t3-install/bounded-attestation-install.json.
/Users/moi/.cache/bridget-delivery172/config-timeouts/receipt.json.
/home/moi/.cache/bridget-delivery172/linux172-worker-probe.json.
Journaux de tests dans /home/moi/.cache/bridget-delivery172.
Aucune base modifiée par les diagnostics. AGENTS.md étranger du Mac préservé.
Correction technique, sans changement d’interface : pas de diaporama requis.

<a id="extrait-13"></a>

## Extrait 13

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

<a id="extrait-14"></a>

## Extrait 14

Le banc par le parcours natif Bridget a été rejoué sur le dernier code :1test/2cas PASS en25,57s,
avec vrai daemon/worker/wrapper et fournisseur simulé. Pas de service T3.
Les missions réelles de codagee125950f et parent natifc5d2017b ont été relues
par rejeu strict après redémarrage du maître, alors que HEAD était passé à124dc47a.
Mêmes Tasks, résultats exacts et ACK conservés ; aucun nouveau lancement.
Base Bridget quick_check=ok. T3 et pont ont leurs mêmes PID et dates de naissance.

<a id="extrait-15"></a>

## Extrait 15

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

<a id="extrait-16"></a>

## Extrait 16

Le correctif T3f3ddfca2 reste signé/installé sur disque et non activé. Les
recettes ci-dessus prouvent le parcours standard depuis T3 courant. Elles
ne prouvent pas encore la correction des tours automatiques dans le nouveau
runtime T3. La consigne utilisateur d’attendre les autres conversations reste
respectée ; aucun redémarrage de T3 n’est programmé.

<a id="extrait-17"></a>

## Extrait 17

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

<a id="extrait-18"></a>

## Extrait 18

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

<a id="extrait-19"></a>

## Extrait 19

Un appel Cartae depuis un dépôt Git local n'exige plus un project_id connu
ni une correspondance préparée manuellement. Bridget retrouve la racine du
parent, transporte le commit exact et crée la correspondance après préparation.
Le transport utilise la fédération SSH existante. GitHub n'est pas nécessaire :
un bundle Git peut contenir un commit non publié. Un transfert ultérieur
importe seulement les objets manquants quand un delta est possible.
La préparation appartient à Bridget ; elle n'appelle pas T3.

<a id="extrait-20"></a>

## Extrait 20

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

<a id="extrait-21"></a>

## Extrait 21

- Préparation autonome Python : 9 PASS, vrais dépôts Git et SSH simulé, par le parcours natif Bridget.
- Résolution/routage : 2 PASS172, 35 PASS168, 1 PASSincident171.
- Délais : 3 PASS172, 8 PASST3MCP, 2 PASSrécupération admission.
- Worker : 21 PASS en série ; première exécution parallèle20PASS/1 verrou
  occupé. Rejeu exact PASS. Cause de concurrence non corrigée ici.
- Protocole : 9 PASS ; négociation ancienne/nouvelle capacité : 1 PASS.
- Enregistrement/reconnexion : 69 PASS/1 ignoré ; reproduction RED puis GREEN.
- Pas de suite globale, pas de succès global déduit.
- Test supplémentaire d'un parent natif dans un terminal :
  recette172-parent-natif-autonomie-20261018-r1 refusée resource_saturated
  sur la machine locale, sans reçu ni tâche. Aucun repli ni changement de
  seuil. Ce parcours complet reste non validé en réel. Les recettes modèles
  ci-dessus partent de parents T3 ; la préparation autonome est testée séparément.
- Nouveaux projets Git seulement. Les fichiers non committés ne sont pas
  transférés. Les projets non Git déjà déclarés gardent leur parcours existant.
- La fédération SSH, Python3 et Git doivent déjà être installés.
- Correction technique : aucun changement d'interface, pas de diaporama requis.

<a id="extrait-22"></a>

## Extrait 22

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

<a id="extrait-23"></a>

## Extrait 23

Aucun résultat réel sur ce nouveau binaire n’est encore revendiqué.
Paquets à préparer avant activation. BUS453c75be-503b-4934-9311-e382733498aa
et les processus T3/Bridget sont préservés. Aucun redémarrage n’est programmé.

<a id="extrait-24"></a>

## Extrait 24

BUS453c75be-503b-4934-9311-e382733498aa reste working, ronde2,
cleanup_done=false. Observation directe Cartae : commande released/native_ready,
PGID320286 présent, naissance185163502 identique au marqueur durable.
Cette observation prouve la survie de son processus, sans conclure à la progression
de son travail métier. Maître50706, pont898 et worker13183 avec NRestarts0 conservés.
Le correctif Bridget72a8e8f8 reste préparé seulement. Aucun service n’a été relancé
par ce contrôle. T016 reste ouverte pour la continuation automatique réelle.
Reçu privé : /Users/moi/.cache/bridget-delivery172/store172/t3-user-restart.json.

<a id="extrait-25"></a>

## Extrait 25

Le18octobre2026, après le signalement de clôture BUS :
cleanup_done=true, guard_released=true, groupe320286vide, aucun autre mandat
natif ni admission auto ouverts. Résultat BUS conservé. Activation réalisée
depuis les paquets vérifiés, sans compilation supplémentaire et sans relance T3.

<a id="extrait-26"></a>

## Extrait 26

Après le premier résultat, le store garde74commandes/104slots/18préparations,
619207octets physiques et1184455octets comptés avec les réserves. L’ancien
plafond16Mio reste inchangé ; aucun reçu réel n’a été effacé pour faire réussir.
Les125tests distincts déjà exécutés restent la base de validation du code.
Aucune nouvelle FULL, aucun typecheck T3 global, aucune release supplémentaire.

<a id="extrait-27"></a>

## Extrait 27

Une tâche locale indépendante46fd5bd7 est working pendant la préparation.
Aucune activation ni interruption de celle-ci à ce checkpoint.
Le maître actif reste72a8e8f8, worker624845 et T3 préservés.

<a id="extrait-28"></a>

## Extrait 28

T041/T042 closes pour ce défaut ;17tests ciblés et lecture réelle validés.
Aucune FULL, aucun changement de schéma ou T3, aucun réveil programmé.
Le statut de l’objectif reste paused. Les autres limites historiques restent distinctes.
