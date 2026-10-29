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

À ce stade, aucune nouvelle matrice d’admissions opus2D n’a encore été reçue. La campagne ne peut pas être présentée comme intégralement qualifiée.

## Frontière de remise entre pairs

Signalement trajets bus mcp-61730-6ae37bb1-13 : accord métier en vol vers un pair occupé et steer refusé not_parent.
Sources vérifiées : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/guidance.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/reference-communication.fr.md.
Un pair peut envoyer en file ; le guidage actif exige un humain Attach ou un parent réellement autorisé. Une simple provenance UI ne crée pas ce droit. Le pair doit attendre un repos attesté/fin de tour au point sûr, ou demander l’intervention de l’autorité admise. in_flight ne prouve ni ACK ni application, et busy/délai seuls ne prouvent pas un défaut de transport.
Réponse envoyée à trajets bus : mcp-35474-6ae38608-a, replyfalse/in_flight. Aucun accord NE02 transféré, aucun geste métier, contrôle de Politique sol ou contournement effectué.
