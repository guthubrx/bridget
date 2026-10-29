# Validation213 — déclarations complètes et portée des preuves

## Lecture native des modèles

Lecture model/list du programme Codex de chaque machine, le29octobre2026, observed_at1793296047/1793296048. initialize et model/list uniquement ; aucun thread/start ou turn/start. Pagination limitée à quatre pages de100, réponse bornée à1Mio, attente bornée. Entrée standard fermée puis arrêt du processus confirmé ; aucune sonde laissée active, aucun SIGKILL.
Les rapports sont privés sous /Users/moi/.cache/bridget213/codex-catalogues.json et /home/moi/.cache/bridget213/codex-models.json sur chaque serveur. Ils ne contiennent que modèles, efforts, provenance et empreinte du programme ; aucune donnée de compte ou clé.

Cartae et SYS proposent huit modèles :
gpt-6.1-sol, gpt-6-astra, gpt-6.1-terra, gpt-6-sol, gpt-6-luna, gpt-5.6-sol, gpt-5.6-terra, gpt-5.6-luna.

KS propose huit modèles :
gpt-6.1-sol, gpt-6-astra, gpt-6-sol, gpt-6-luna, gpt-5.6-sol, gpt-5.6-terra, gpt-5.6-luna, gpt-5.5.

Cette lecture ne prouve pas l’accès réel. En particulier, model/list répond sur KS alors qu’une mission précédente a reçu workspace routing discovery unauthorized (401). Il ne faut donc pas transformer cette lecture en authentification réussie ni annoncer Terra6.1 sur KS alors qu’il n’est pas dans sa liste observée.
Les anciens alias déclarés ne sont pas effacés ni déclarés qualifiés sur la seule absence de cette nouvelle observation.

## Configuration persistante, sans nouveau binaire

Profils ajoutés :
- codex-cartae-terra213, modèle gpt-6.1-terra ;
- codex-sysle2-terra213, modèle gpt-6.1-terra ;
- claude-sysle2-haiku213, modèle claude-haiku-5-5 ;
- claude-ksstore-haiku213, modèle claude-haiku-5-5.

Les trois modèles Claude sont désormais déclarés sur chaque machine : claude-opus-5-5, claude-sonnet-5-5, claude-haiku-5-5. Les huit modèles Codex réellement annoncés par chaque programme sont couverts par les déclarations de la machine correspondante. Codex KS reste disabled dans les autorités, en raison du refus401 attesté ; aucune nouvelle tentative sur ce compte.

Fichiers privés concernés :
/home/moi/.cache/bridget-core/agents.json sur le maître Cartae ;
/home/moi/.config/bridget/agents.json sur chacun des trois workers ;
/Users/moi/.cache/bridget-core/origin-agents177.json sur l’origine Mac.

Sauvegardes, verrou project-preparation.lock, comparaison des octets et publication atomique avec fsync. Les anciennes définitions et connexions sont inchangées. Les mappings, empreintes de credentials et révisions de provisionnement sont conservés. Aucun compte, token ni dossier d’authentification copié. Les profils sont ajoutés pour préserver les empreintes figées des travaux déjà admis.
La préparation de l’origine a refusé la première candidate avant toute écriture : sa définition locale Codex n’avait pas l’observation du binaire Linux. Les nouveaux profils distants explicites ont ensuite été pris depuis l’autorité maître et ajoutés à l’origine ; aucune garde n’a été relâchée et aucune ancienne définition n’a été remplacée.
Les contrôles clients natifs sur les trois serveurs réussissent : compatible, inventaires non vides. Les assertions de couverture passent sur maître et workers.

Ces réglages sont persistants pour cette flotte. Ils ne constituent pas des comptes ni droits universels pour des installations futures. Une future préparation doit relire le catalogue du programme et de la connexion concernés ; ne pas copier une liste d’un autre compte comme preuve d’accès.

## Activation et témoin technique

Le build212-fleet-19e0b5c3 reste actif. Aucun nouveau binaire, testCargo ou redémarrage requis pour ces données. La nouvelle admission relit le registre par le parcours natif existant.
Lecture réelle bridget_capabilities : fleet.status=declared,45options, gel=false. Les quatre options ajoutées apparaissent avec les identifiants machine exacts cartae,sysle2,ksstore.

Qualification technique SYS/Haiku : tâcheb22eef97-47cc-480c-8fca-3f5176d1e672, requestbridget213-qualification-haiku-sys-20261029-01, sourceBridgetd020d37b, worktree/development. Résultat BRIDGET213_SYS_HAIKU_OK lu et acquitté ; Cancel propriétaire demandé, résultat conservé, retentionreview_required.
Cette preuve porte sur le lancement depuis le parent Bridget. Elle ne prouve pas le contexte du parent opus2D.

## Qualification depuis opus2D

La campagne unique qual213-opus2d-20261029 a été transmise au propriétaire monuments af5ff0db-b846-45a2-8158-c4ab53a89b84, message mcp-35474-6ae38927-b, sans réponse attendue par le transport mais avec action demandée. La même campagne est portée par une notification native du fil32e1b8f8-938b-488c-a05e-67200732b973, clientRequestIdbridget213-qualification-opus-parent-20261029-01, livraisonstarted/run578. Le propriétaire doit la traiter une seule fois.
Les nouvelles admissions doivent provenir de son parent attesté. Aucun usage d’une identité empruntée ou d’un cwd hors du périmètre de notre parent. Réutiliser les preuves opus2D déjà acquises ; garder les modèles seulement déclarés distincts des modèles essayés. Chaque essai conserve sa clé, son enveloppe, sa tâche et son résultat. Codex KS attend une connexion utilisateur validée ; ne pas multiplier les essais refusés.

Matrice reçue du propriétaire, fil32e1b8f8, tours581 et582, le29octobre2026 à18:24UTC. Base commune d96b0b7ec88bce5b1ffdeb1a9ddea67ecb021de4, worktree/development, child_requests deny. Les marqueurs suivants proviennent du parent opus2D :

| Machine | Modèle | Tâche | Résultat conservé |
|---|---|---|---|
| cartae | claude-haiku-5-5 | d61d26d7-c1cd-4547-8488-255282341186 | QUAL213-OPUS2D-HAIKU-CARTAE-OK |
| sysle2 | claude-haiku-5-5 | f81327ee-f23a-4e5e-a018-48e839230da7 | QUAL213-OPUS2D-HAIKU-SYSLE2-OK |
| ksstore | claude-haiku-5-5 | 5e8cd223-6cea-4b68-a9a1-e89da1e3ddf8 | QUAL213-OPUS2D-HAIKU-KSSTORE-OK |
| cartae | claude-sonnet-5-5 | 1bfe71d2-b94f-4136-9044-11a87b4585b5 | QUAL213-OPUS2D-SONNET-CARTAE-OK |

Le propriétaire a confirmé les réceptions. Ses missions réelles Opus sur Cartae sont déjà acquises ; elles ne sont pas relancées. Les quatre témoins Opus/Sonnet SYS/KS de212 restent des preuves depuis le parent Bridget, sans prétendre requalifier le parent opus2D.

Les deux enveloppes Terra ont été retrouvées par UN rejeu strict propriétaire : qual213-opus2d-20261029-terra-cartae et qual213-opus2d-20261029-terra-sysle2. Refus routing_candidate_unavailable, permissions_changed, state not_recorded, task_id null. Aucun remplacement ni nouvelle clé. Les autres modèles Codex depuis ce parent restent non qualifiés.

### Cause constatée des refus Codex depuis le parent Claude

Les journaux workers corrélés portent permission_mapping_unavailable au site projection :
- Cartae : admission a-1bf77046baf7ba304e52b45f771a9043, corrélation f5619be9-0468-42e3-b2c7-b2e081f3c8c3 ;
- SYS : admission a-fd209b726097c9ebe4896bfede36ac04, corrélation15a03253-0c0a-456c-8cea-7ead036194a8.

La preuve parent conservée sur l’essai Haiku atteste driver claude_stream_json, outils preset claude_code, règle allowed_tools exacte mcp__t3-code__*, callback t3_runtime. Dans native_permissions.rs, project_remote_permissions_in_operation appelle require_isolated_claude_mount171 pour cette règle. Ce contrôle exige une définition claude_stream_json ; une définition codex_app_server est refusée avant projection. Le chemin de conversion Claude vers Codex refuse aussi les allowed_tools non vides. Il s’agit donc d’une frontière de traduction des droits du parent, pas d’un programme absent ni d’une révision qui aurait changé. La projection publique permissions_changed reste moins précise que la preuve worker.

Aucun retrait de règle, changement de parent, élargissement de droits ou contournement n’a été effectué. La demande de reprise interdit de recommencer code/builds/qualifications. Une correction de cette traduction et de son guidage demeure à traiter dans un chantier exécutable distinct, avec qualification de la conservation des restrictions. Ne pas annoncer Codex utilisable depuis ce parent, ni demander des rejeux tant que cette cause reste présente. Les essais existants conservent leur clé et leur enveloppe. Les travaux Claude autorisés suivent leur propre état.

### Incident transitoire de liaison et client conservé

Après les trois premiers résultats, les appels du propriétaire ont rencontré budget JSONL dépassé puis binding_transport_unavailable. La lecture publique native du daemon répondait en0,439s surMac et0,015s surCartae ; le catalogue racine restait accessible. Ces observations ne reconstituent pas la cause des délais historiques.
UNE réattestation puis UNE lecture de la même tâche d61d26d7 ont réussi au tour580, à18:19:24UTC. La reprise de la campagne au tour581 a obtenu le résultat Sonnet ensuite. Le client MCP du propriétaire est resté209 pendant ce rétablissement. Aucun restart ni correctif de transport n’a été inventé à partir de ce retour. Le prélèvement passif du processus a seulement montré une attente stdin ; il est non conclusif.

Au contrôle final du fil racine, UNE bridget_session a réussi et UNE bridget_capabilities a rendu fleet declared,45options,delegations_halted=false. ps/lsof attestent encore MCP35474 chargé depuis /Users/moi/.local/lib/bridget-communication/211-directory-650e5ef5/bin/bridget. La description reste211, les données212 sont accessibles. Aucune boucle de restart ; une reconnexion réelle du client gérée par T3 reste nécessaire pour charger sa nouvelle description. Les services déjà activés ne sont pas redéployés.

### Blocage humain Codex KS

Le refus fournisseur401 attesté en212 reste distinct. Aucune nouvelle tentative de login, aucune mission et aucune réactivation Codex KS dans213. Geste utilisateur : ssh -t ksstore '/home/moi/.local/bin/codex login --device-auth'. Attendre un succès réel, puis une qualification autorisée avant réactivation. model/list ne remplace pas cette preuve.

## Frontière de remise entre pairs

Signalement trajets bus mcp-61730-6ae37bb1-13 : accord métier en vol vers un pair occupé et steer refusé not_parent.
Sources vérifiées : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/guidance.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/reference-communication.fr.md.
Un pair peut envoyer en file ; le guidage actif exige un humain Attach ou un parent réellement autorisé. Une simple provenance UI ne crée pas ce droit. Le pair doit attendre un repos attesté/fin de tour au point sûr, ou demander l’intervention de l’autorité admise. in_flight ne prouve ni ACK ni application, et busy/délai seuls ne prouvent pas un défaut de transport.
Réponse envoyée à trajets bus : mcp-35474-6ae38608-a, replyfalse/in_flight. Aucun accord NE02 transféré, aucun geste métier, contrôle de Politique sol ou contournement effectué.

## Fin de reprise documentaire

Contrôle final limité : aucun code, Cargo, build, fournisseur, qualification ou service relancé. Les guides passent32pages/522liens/13paires et git diff --check est propre.
Le résultat a été envoyé à monuments par le nouveau message Bridget mcp-35474-6ae39189-c, deliveryf2a35ee5-0fe1-4758-9d80-a70987e2c88d, issued_at1793298825, reply=false. Le reçu est in_flight : aucune lecture du demandeur n’est déduite.
Les quatre tâches opus2D ci-dessus restent result_available/error=null/cleanup_done=true ; guard_released=false au registre. Seul leur propriétaire les acquitte/clôture. Leurs dossiers sont conservés. Le témoin Bridget Haiku SYS est déjà clôturé ; son dossier reste conservé sous observation d’usage incomplète. Aucune suppression de travail ni de binaire sous usage.
L’onglet privé de diagnostic non apparié a été vidé ; aucun jeton copié ni contrôle d’un autre onglet.
La session213 n’est pas déclarée intégralement résolue : accès Codex depuis le parent Claude et connexion utilisateur CodexKS restent ouverts. Les données ajoutées sont actives ; le binaire212 déjà chargé n’a pas besoin d’être reconstruit pour ce bilan.
