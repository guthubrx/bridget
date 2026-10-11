# Journal d'implémentation151

Feature :151-attente-fin-de-tache. Branche :session-151-attente-fin-de-tache.
Début :2026-10-11 08:18:53 CEST. Statut :Implemented. Aucun commit.

État courant après T027 : Implemented, 27 tâches cochées sur 27.
La preuve SC-009 distincte donne 1 PASS, sans FAIL ou ignoré. Les relevés
26/26 ci-dessous décrivent l'étape avant cette tâche de convergence. W004
garde son total2046/8/72 avant T027 ; aucun total workspace2047 n'est déduit.

Préflight/synchronisation effectués, audit de réutilisation PASS, Analyze exécuté
puis relu après corrections, revue du plan APPROVE_WITH_CHANGES traitée.
Specify/Plan/Tasks réutilisés, aucun document régénéré en masse.

## T001 validée

Tests177PASS,0FAIL,1ignored ; filtre batch0test. Connexion sans trame15.011s
puis sonde d'identité sans Register réussie. Voir validation/baseline.md.
Les deux réserves de forme de la checklist sont levées explicitement, avec
justification ; les tests fonctionnels restent obligatoires.

Le socle150 est importé sans commit et conservé à part dans dependency150.patch
et dependency150.json. Le diff final distinguera ce socle des changements151.

## Lot stockage 151 - état partiel validé

T004 et les portions stockage de T015/T016 sont implémentées. La colonne
`tracked_requests.reply_summary` existe dans les deux schémas. Chaque ouvreur
migre les anciennes bases avec un contrôle `PRAGMA table_info`, sous transaction
immédiate. Le champ reste nullable ; les anciennes lignes gardent `NULL`.

La transition commune `mark_answered_in_transaction` écrit l'état et le résumé
dans le même UPDATE. Elle filtre la demande, son émetteur, sa cible et l'état
`open`. Le premier passage à `answered` fige le résumé, y compris `None` pour
le guichet. Une réponse tardive ne remplace pas ce premier fait.

`request_reply_summary` calcule un JSON `id/from/summary/summary_truncated`
depuis le vrai message de réponse. Le champ `summary` garde 1024 octets au plus
sur une frontière UTF-8. Cette borne porte sur le corps résumé, pas sur le JSON
entier. Le helper réutilise `floor_char_boundary`. Il sert aux chemins de
réponse idempotent et hérité. Le chemin hérité est raccordé par le principal.

`Store::request_wait_snapshot` lit état, échéance et résumé dans un seul SELECT,
avec identifiant et émetteur. Elle ne lit pas le ledger et n'écrit aucun fait.
Les notifications après commit appartiennent au daemon, raccordé par le principal.

Complexité : la migration contrôle C colonnes, O(C). La coupe du corps examine
1024 octets au plus ; la sérialisation ajoute les longueurs des identifiants
`id` et `from`. L'instantané utilise la clé primaire de demande : O(log n)
pour n demandes conservées, puis lecture d'une seule ligne.

Périmètre source et tests de cet owner :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/store.rs
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/store/ledger_requests.rs
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/store/service_events.rs
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/store/tests.rs
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/idempotency.rs
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/idempotency/send_delivery.rs
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/idempotency/tests.rs

## Revue préparatoire du cœur et de la façade

Le principal a transmis le retour Explore du cœur. Cette revue a lu trois
fichiers seulement. Elle ne déclare aucun test exécuté. Le seul écart réel
retenu porte sur FR-034 : le code intermédiaire rendait le texte de `result`
hors `result_available`. La relecture du code 150 a confirmé le filtrage
historique. L'assertion correcte a été vue rouge, puis les trois tests de vue
sont verts après correction.

L'ancienne passe verte fondée sur l'attente de test erronée reste historique.
Elle ne valide pas FR-034. Dans le périmètre lu, Explore n'a trouvé aucun autre
défaut de pureté, propriété, réveil perdu, fermeture, permission ou protocole.
Ce constat ne constitue pas une revue exhaustive du dépôt.

L'owner façade confirme le registre unique, les quotas et la fermeture sur
annulation ou EOF. Les faits de tests sont consignés séparément des constats
de lecture. Le principal a retiré deux lignes liées à `cwd` inutilisé.
Deux wrappers hérités, appelés uniquement par des tests, sont marqués `cfg(test)`.
Aucune migration ni aucun refactor hors 151 n'est ajouté.

La comparaison fraîche de `execute_send`, faite en lecture seule par l'owner
façade, est identique octet par octet entre le socle 150 et 151.
L'extraction bytes utilise `rb'(?ms)^fn execute_send\(.*?^}'` : elle va de la
signature à l'accolade finale en colonne 1, sans le LF final.
Chaque extrait mesure 6283 octets et porte le même SHA-256 :
`7461ca7a3eb53a6e3ca78fcb9e2c0663b1f27f69b053e95ad35571d6f44edf68`.
En incluant le LF final, les deux extraits mesurent 6284 octets et portent :
`fdc0168ba3720a7d0de467d4a026a1c8ad7694beb5e732abca6991b39bda4d26`.
La comparaison des bytes est réussie. Ce contrôle compare la fonction source ;
il ne constitue pas une exécution de bout en bout.

Source 150, ligne 673 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation/crates/bridget-daemon/src/mcp.rs

Source 151, ligne 870 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/mcp.rs

À cette étape, T023 restait ouverte. Le principal a ensuite vérifié et coché
son snapshot avant ACK ; cet owner ne modifie aucune case.

## SC-008 - preuves finales et arbitrage des outils de test

Les deux tests au calme donnent 2 PASS, 0 FAIL et 0 ignoré. La trace SQLite
observe trois SELECT pertinents sur chaque connexion réelle. TaskWait lit
aux millisecondes [0, 5000, 10001] ; RequestWait lit à [0, 5002, 10002].
Les epochs restent à 2 et 0. La capacité est libérée, le payload de tâche est
intact et la demande reste ouverte avec son échéance métier de 3600 secondes.

Le test de charge donne 1 PASS, 0 FAIL et 0 ignoré avec 64 attentes. Status
prend 2,430 ms et Cancel 19,056 ms. L'écrivain passe une barrière après son
premier save et continue pendant les contrôles : progression 1 -> 2 pour
Status, puis 2 -> 5 pour Cancel, total 6. L'arrêt et le join viennent après
les mesures. La déconnexion du canal arrête aussi cet écrivain de fixture.

L'observateur `SelectTrace` est un garde RAII limité à `cfg(test)`, dans le
module `native151_sql_trace_tests` du daemon. Il a deux usages réels : les connexions de
TaskWait et RequestWait. L'API C `sqlite3_trace_v2` exige un contexte à adresse
stable. Le Box le conserve ; Drop désinstalle le callback avant sa libération.
Les tests gardent la connexion vivante au-delà du garde. Le callback empêche
une panique Rust de traverser la frontière C. Ce garde évite deux montages FFI
dupliqués ; il ne constitue pas une abstraction de production.

L'accesseur unique `DelegationStore::test_connection()` est lui aussi limité
à `cfg(test)`. Il est absent du socle 150 et appartient aux ajouts 151 partagés.
Il est réutilisé pour les tests d'ACK. Aucun second accesseur, hook de
production ou dépendance nouvelle n'est nécessaire.

Sources de ces seuls outils de test :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/daemon.rs
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/daemon/native_delegation.rs
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/delegation.rs

Preuves finales SC-008 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/SC008-151-calme-final.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/SC008-151-charge.log

## Relecture du stockage - faits et réserves

Le principal transmet une relecture Explore du stockage. Elle confirme le
premier passage à `answered` et son résumé figé dans le même UPDATE, ligne 699 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/store/ledger_requests.rs

Elle confirme la transaction commune de l'enveloppe, de l'ACK et de la clôture
de demande, ligne 763 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/idempotency/send_delivery.rs

Le guichet garde `None`, ligne 379 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/store/service_events.rs

La migration gardée utilise `Immediate`, ligne 655 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/store/ledger_requests.rs

Les notifications du daemon viennent après commit si les faits ont changé.
Le rejeu de Cancel provoque encore un bump inutile : réserve non bloquante,
sans violation de FR-039. La concurrence des migrations et les mesures runtime
du compteur après rollback ou opération sans changement de réponse/demande
ne sont pas prouvées. Les tests d'atomicité du résumé ne remplacent pas ces
mesures. Ces constats sont des lectures, pas de nouvelles exécutions de tests.
Le principal confirme ensuite la relecture de tous les deltas de production151.
Le seul défaut actif alors trouvé était FR-038. Son correctif dispose maintenant
des preuves ciblées vertes ; la relecture RO après correctif n'avait confirmé
aucun défaut. Le full post-EOF révèle toutefois quatre échecs nouveaux.
Leur diagnostic et les nouveaux GREEN ciblés sont distingués ci-dessous.

## T022 premier passage et correction du banc des exemples

Le premier passage complet donne 2029 PASS, 10 FAIL, 72 ignorés et le code
retour 101 sur 94 cibles Cargo. Le premier compte préliminaire brut additionnait
97 bilans : 2032 PASS, 10 FAIL et 72 ignorés. Trois réussites de sous-processus
étaient comptées une seconde fois. Le compte corrigé groupe les headers Cargo
`Running unittests`, `Running tests/` et `Doc-tests`, puis garde le dernier
bilan de chaque cible. Huit échecs sont hérités
de 150 selon la comparaison du principal. Deux échecs suivent les ajouts151 :
la liste fermée Claude attend encore 20 outils au lieu de 22, et le banc089
compte neuf JSON publiés au lieu des six exemples de communication138.
La façade possède le premier correctif ; cet owner possède uniquement le second.

Le helper `examples` existant prend le texte à analyser. Le titre stable du
contrat138 définit la portée des six exemples de communication, y compris les
chapitres suivants d'envoi, de réponse et de ledger. Aucun filtrage par nom
n'exclut un exemple. Le nombre global reste exactement neuf ; les groupes
doivent avoir six et trois objets. Les assertions complètes protègent leurs
noms, leurs arguments et leur ordre. Toutes les assertions métier du banc089
sont conservées.

Le RequestWait151 publié est ajouté après la réponse corrélée connue, sous
l'identité du parent. Seul le `request_id` reçoit la vraie clé de question.
Le code vérifie l'état answered, la réponse exacte et un ledger inchangé.
Le principal a exécuté ce banc : 1 PASS, 0 FAIL et 1 ignoré, code retour 0,
durée 2,93 secondes. RequestWait publié est donc réellement exécuté.
À ce stade, TaskWait et Ack publiés restaient en attente d'E2E. Leur validation
JSON ne prouvait pas leur exécution. Le GREEN post-ACK apporte maintenant
cette preuve réelle.

Source corrigée, seul test possédé dans ce complément :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/tests/core_089_skill_test.rs

Premier journal complet :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T022-workspace151.log

GREEN des exemples publiés :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-skill-publications151.log

La liste fermée Claude corrigée donne séparément 1 PASS, sans échec ni ignoré.
Ces deux réussites ciblées ne remplacent pas le premier bilan complet.
Le full post-EOF W002 est ensuite terminé : 2030 PASS, 12 FAIL et 72 ignorés,
code retour 101, sur 94 cibles dont neuf en échec. Le premier compte brut de
98 bilans donnait 2033 PASS, 13 FAIL et 72 ignorés. Trois PASS de sous-processus
et le FAIL imbriqué `isolated_identity_entrypoint` étaient comptés deux fois.
Ce dernier appartient à la cible `t3_session_identity_env` ; il n'ajoute pas
un treizième échec. L'ACK n'est pas modifié avant la clôture de T023.

## FR-038 - EOF compatible prouvé, snapshot autorisé

La relecture finale a repéré l'EOF d'un mélange Wait + Status bloqué. Sa
reproduction empirique donne 0 PASS, 1 FAIL : Wait3600 et Status démarrés et
authentifiés, connexions backend fermées, mais fin de la façade en
10,000712583 secondes. La limite de FR-038 reste 6 secondes.

Le correctif de la façade couvre inscription et annulation durant la connexion
JSONL, puis fermeture des appels enregistrés sur EOF. Le GREEN MCP mesure le
même mélange Wait + Status à 0 ms, donc moins de 1 ms. Ses 75 tests passent,
avec 1 ignoré. Les 19 tests du connecteur restent verts. Le JSONL donne 3 PASS
et la liste fermée Claude donne 1 PASS.

Tous les appels MCP ont un contexte local au thread, retiré par leur garde.
Un unique contrôle relie le connecteur au registre existant. L'EOF ferme leurs
sockets, y compris ceux des appels ordinaires. Une notification d'annulation
nonwait préserve au contraire l'opération et masque seulement la réponse.
Les tests vérifient ce comportement pendant connexion, authentification et
opération. Il n'y a pas de nouvelle règle dans `execute_send` : sa comparaison
archivée garde 6283 octets et le même SHA-256 que 150, sans le LF final.

Le transport expose un callback générique à l'intérieur de la connexion
non bloquante. Le wrapper `connect_nonblocking` reste compatible et passe
`None`. Le callback observe le socket avant connect, durant le poll et après
connexion ; le socket déjà enregistré couvre ensuite l'authentification.
La même échéance absolue reste utilisée. Aucun nouveau budget, retry ou dial
n'est ajouté. Le transport ne dépend pas du module MCP et aucun module ou
dépendance nouveau n'est créé.

Le test du callback avant connect a été vu rouge : 0 PASS, 1 FAIL, aucune
observation. Le GREEN JSONL observe le socket une fois et interrompt avant dial
en 14 microsecondes. Le backlog est réel, mais macOS rend `ECONNREFUSED`.
Le test ne passe donc pas par `EINPROGRESS`/poll. Cette branche est relue dans
le code seulement ; aucune observation runtime du poll n'est revendiquée.

T026 a été réouvert pour ce scénario. Ses premières preuves ciblées sont
vertes. La relecture indépendante fraîche ne confirmait alors aucun défaut. Le
principal transmet les points de contrôle suivants : registre et garde MCP
aux lignes 84, 90, 97, 158, 294 et 361 ; connecteur à la ligne 953 ; JSONL aux
lignes 32, 64, 73 et 93. Cette revue relit le code ; elle ne déclare pas de
nouvelle exécution de tests.
Le principal avait coché T012/T026 et confirmé 22 cases sur 26. Le full
post-EOF révèle ensuite quatre échecs nouveaux : contenu089,
`cli_et_mcp_reels_lisent_les_memes_octets_sans_droit_supplementaire`, et
isolation089, `vrai_daemon_et_clients_n_utilisent_que_namespace_prive`.
Ils échouent sur `Option::None` aux lignes 261 et 415. La preuve d'identité
`partial_or_invalid_t3_proof_never_falls_back_to_pid` échoue aussi ; son
sous-processus `isolated_identity_entrypoint` n'est pas un échec supplémentaire.
Le quatrième FAIL est l'E2E du message unique après expiration.
Les huit échecs hérités de 150 restent inchangés. T026 est alors réouvert ;
le principal confirme de nouveau 21 cases sur 26.

La perte EOF des trois clients ordinaires est désormais reproduite : Status
est authentifié et reçu, le backend répond après 100 ms, mais initialize id1
est la seule sortie et id2 manque. Le test donne 0 PASS, 1 FAIL. Ces clients
ferment stdin après l'envoi. Fermer immédiatement leurs sockets casse FR-034.

Le principal autorise une grâce de 4,5 s pour leurs appels ordinaires sur EOF.
Les attentes sont annulées immédiatement. La Condvar ajoutée au registre MCP
attend que ses membres terminent, sous le même verrou que leur retrait et
leur notification. Aucun état parallèle, thread ou polling n'est ajouté.
Le compteur `WaitEpoch` du daemon serait inadéquat : il couplerait la fin des
appels du processus MCP aux réveils métier et au stockage du daemon.
Les budgets existants restent inchangés tant que l'entrée est ouverte.

Les journaux transmis ensuite prouvent le GREEN compatible : MCP 76 PASS et
1 ignoré ; contenu089 7 PASS ; isolation089 8 PASS et 1 ignoré ; preuve
d'identité T3 1 PASS et 1 ignoré ; connecteur 19 PASS. Les trois bancs clients
historiques restent inchangés. Status conserve sa réponse id2 après EOF en
105 ms, avec réception backend confirmée. EOF avec Wait3600 et Status muet
finit en 4504 ms et ferme les deux sockets. Les trois fixtures MCP unitaires
à entrée finie ont aussi leur forme originale restaurée et passent.
Ces passes ciblées ne réécrivent pas les échecs historiques de W002.

Le quatrième FAIL a une cause distincte observée dans la base SQLite privée.
La tâche porte `failed`, `result: null`, `error: "échéance Claude dépassée"`
et `cleanup_done: true`. La configuration de recette a
`notify_timeout_secs: 2`, expiré alors que l'enfant attend le fichier de
libération. Le constructeur de fixture existant est adapté : ancien148
conserve 2 s, les trois151 ont 15 s. Le nouveau journal donne trois E2E151
verts, aucun échec, deux filtrés, en 6,41 s. SC-001 mesure 945 ms, capture
encadrée sur 7 ms, zéro Status avant résultat. Sa borne reste au plus 2 s
de capture à réveil. SC-006 confirme pending -> claimed -> delivered, un avis
unique et nettoyage durable. EOF ferme quatre attentes de 3600 s en 11 ms.
Le test148 séparé donne 1 PASS, 0 FAIL, quatre filtrés, à son délai de 2 s.

La relecture fraîche EOF ne trouve aucun défaut concret. La compilation
normale est sans warning selon le principal. Il clôt T022 avec « échecs
expliqués » et T026 : 23 cases sur 26, T023/T024/T025 encore ouvertes.
Il donne le GO du snapshot T023. W002 reste historique en échec, avec huit
FAIL hérités et quatre corrigés isolément. Le full final après ACK est
obligatoire ; aucune nouvelle répétition complète intermédiaire n'est requise.
Cet owner ne modifie aucune case.

Journal W002 terminé, non final :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T022-workspace151-post-eof.log

RED EOF des clients ordinaires et diagnostic privé E2E :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-eof-nonwait151.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/diagnostic-e2e-deadline151.log

GREEN compatibles relus :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-mcp-final.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-core_089_content_test.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-core_089_isolation_test.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-t3_session_identity_env.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-client.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-e2e.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-e2e148.log

Preuve rouge :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-eof-statut151.log

RED du callback avant connect :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-fd-connect151.log

GREEN ciblés et preuve source de `execute_send` :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-mcp-eof151-final.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-client-eof151.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-transport-eof151.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-claude-catalogue151.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/execute-send-eof151-sha.log

## Six réponses Article XX - stockage, cœur et façade

1. **Pourquoi cette solution est-elle nécessaire ?** Une réponse doit rester
   lisible après son accusé. La clé du ledger `(id, target)` peut être remplacée
   par un autre émetteur dont l'identifiant de réponse est identique. Conserver
   le résumé dans la demande, au premier passage à `answered`, empêche de rendre
   le texte d'une autre réponse. L'instantané filtré limite la lecture au demandeur.
   Le parent doit aussi attendre dans le même tour sans relire sans cesse le statut.
   Le compteur de réveil rend cette attente possible. Les bornes et la fermeture
   des sockets empêchent les appels longs d'occuper indéfiniment les créneaux.
   Les contrôles restent disponibles pendant l'attente.
   FR-038 impose aussi la fin de la façade sur EOF même si un appel ordinaire
   bloque en même temps qu'une attente. Le socket doit être enregistré avant
   connect, puis rester fermable pendant authentification et opération.
   FR-034 impose de conserver les réponses rapides des clients qui ferment
   stdin après l'envoi. La grâce de 4,5 s réunit ces deux contraintes.
   L'ACK de lecture distingue une tâche lue d'un résultat seulement livré.
   Son premier timestamp doit survivre aux sauvegardes métier anciennes.

2. **Pourquoi est-elle plus simple ou plus maintenable ?** Elle étend la
   transition et les deux schémas existants. Elle ajoute une colonne nullable,
   un calcul partagé du résumé et une lecture ciblée. Elle évite une table de
   boîte, une nouvelle relation de message et un balayage des enveloppes.
   L'état et le résumé ont une seule transaction et une seule règle de clôture.
   Le cœur étend les modules existants avec un petit compteur partagé ; le canal
   Lineage existant est à consommateur unique et ne couvre pas tous les faits.
   La façade garde un seul registre pour les créneaux, l'annulation et le socket.
   La sonde utilise une trame ancienne, avant toute trame nouvelle. Elle évite
   une attente muette chez un ancien daemon, sans nouvelle négociation de capacité.
   Les deux mesures SC-008 partagent un garde de trace SQLite réservé aux tests.
   La contrainte de durée de vie du contexte C justifie son Box et son Drop.
   Le correctif EOF étend le connecteur commun avec un callback générique.
   Son wrapper compatible garde les anciens appelants. Un seul contexte local
   au thread et le registre existant couvrent tous les appels MCP ; aucun
   montage de connexion, budget, retry ou dial parallèle n'est ajouté.
   Sa Condvar attend directement la fin des membres du registre. Elle évite
   un thread ou un polling supplémentaire. Réutiliser `WaitEpoch` du daemon
   créerait un couplage entre le processus MCP et le stockage métier.
   L'ACK étend `DelegationStore` avec une transaction JSON1 pour tout le lot.
   Il évite `save_projected`, qui commit par tâche et peut modifier les dates
   et Lineage. Aucun module, table ou dépendance ne lui est ajouté.

3. **Quelles hypothèses ont été prises ?** Les demandes ont une clé primaire
   globale `id`. Les collisions permises portent sur les identifiants des
   réponses, dans deux scopes différents ; les demandes initiales restent
   distinctes. Le guichet et les anciennes lignes peuvent ne pas porter de
   message utilisable : `NULL` est un fait valide, rendu comme
   `reply_unavailable`. Le résumé est figé lors de la réponse, pas recalculé
   depuis un ledger ultérieur.
   Une attente valide 1 à 16 identifiants avec la garde de propriété de Status,
   sans reprise implicite du propriétaire. Expirer l'appel ne clôt pas la mission.
   Le délai de réponse et celui d'attente restent distincts. Quatre attentes par
   façade et 64 dans le daemon bornent les ressources. L'EOF et l'annulation
   ferment le socket enregistré, y compris pendant la sonde ou l'authentification.
   Le montage Codex de Bridget porte seul le délai d'outil à 3700 secondes.
   Sur EOF, les attentes sont annulées immédiatement ; les appels ordinaires
   ont au plus 4,5 s pour terminer avant fermeture forcée. Une notification
   d'annulation nonwait doit laisser finir l'opération et masquer uniquement sa réponse.
   L'échéance absolue de connexion reste inchangée avec ou sans callback.
   Le prédicat de la Condvar et les retraits des membres utilisent le même
   verrou. La notification de sortie doit donc réveiller la grâce sans course.
   L'ACK valide 1 à 16 identifiants uniques. Sous le verrou du daemon, toute
   la propriété est contrôlée avant les états. Seuls `result_available` et
   `failed` sont admis ; `cancelled` reste refusé. La date vient du serveur.

4. **Qu'est-ce qui a été vérifié ?** Six tests ont été vus rouges avant la
   production : migrations des deux ouvreurs, guichet, collision de scopes,
   coupe UTF-8 et atomicité de l'ACK. Sept tests sont ensuite verts ; le septième
   vérifie l'instantané filtré, la première réponse et l'absence d'écriture.
   Le test d'atomicité force un échec SQL : ni l'ACK ni l'état `answered` ne
   gagnent sans le résumé. Les filtres `store::tests` et
   `idempotency::tests` donnent respectivement 50 PASS et 46 PASS, sans échec.
   Le premier filtre inclut aussi les tests de blobs et de reçus.
   Relecture et contrôle d'espacement sont faits sur les sept fichiers.
   Le protocole a quatre tests verts. La première passe ciblée 151 a 42 PASS.
   La correction exacte de FR-034 a un vrai RED 0 PASS/1 FAIL, puis un GREEN
   de trois tests. T021 rejoue les six filtres complets : 177 PASS, 0 FAIL et
   1 ignoré, avec les mêmes noms et résultats que la référence. Les trois E2E
   151 donnent 3 PASS et deux tests filtrés, sans compter le test historique 148.
   La façade a aussi des régressions archivées : 14 PASS ciblés, MCP 73 PASS et
   1 ignoré, client 19 PASS, montage 5 PASS et 1 ignoré. Ces filtres se recouvrent.
   La fermeture EOF renforcée de quatre attentes de 3600 secondes est mesurée
   à 11 ms, puis à 10 ms dans le dernier rejeu. La preuve finale SC-001 ancre
   la mesure sur la capture SQLite, encadrée sur 6 ms : borne conservatrice
   de 980 ms, zéro Status avant ready. Le vrai ancien daemon refuse les deux
   attentes en 44,710/54,874 ms
   et ne reçoit que les sondes anciennes. Les constats Explore sont des lectures ;
   les compteurs viennent des journaux des exécutions coordonnées.
   SC-008 ajoute deux tests au calme et un sous charge. La trace mesure trois
   SELECT à environ 0, 5 et 10 secondes. Avec 64 attentes et un écrivain actif,
   Status prend 2,430 ms et Cancel 19,056 ms. Les réservations sont libérées.
   Le premier workspace complet donne 2029 PASS, 10 FAIL et 72 ignorés sur
   94 cibles ; le compte brut préliminaire de 2032 PASS incluait trois doublons.
   Ce bilan est non final. Le RED EOF mixte confirme FR-038 avec 0 PASS et 1 FAIL,
   délai observé 10,000712583 secondes contre la limite de 6 secondes.
   Après correction, le même EOF mixte prend moins de 1 ms. MCP : 75 PASS et
   1 ignoré ; client : 19 PASS ; JSONL : 3 PASS ; liste Claude : 1 PASS.
   Le callback avant connect a son RED 0 PASS/1 FAIL, puis le GREEN JSONL.
   Les notifications nonwait sont testées pendant connexion, authentification
   et opération. Le banc089 donne 1 PASS et 1 ignoré avec RequestWait publié
   réellement exécuté ; les six exemples138 et leurs assertions restent présents.
   Le full post-EOF donne 2030 PASS, 12 FAIL et 72 ignorés sur 94 cibles,
   code retour101. Le compte brut préliminaire de 2033 PASS/13 FAIL incluait trois PASS
   et le FAIL d'identité imbriqué une seconde fois. Huit FAIL sont hérités,
   quatre nouveaux. Le RED EOF rapide montre id2 absent malgré la réponse
   backend après 100 ms. Le GREEN compatible conserve id2 en 105 ms ; le
   mélange Wait3600 et Status muet finit en 4504 ms, sous les 6 s de FR-038.
   MCP : 76 PASS et 1 ignoré ; contenu : 7 PASS ; isolation : 8 PASS et
   1 ignoré ; identité T3 : 1 PASS et 1 ignoré ; connecteur : 19 PASS. Les trois bancs clients
   n'ont pas été modifiés. La lecture SQLite E2E observe séparément l'échéance
   de 2 s dépassée. La fixture151 à 15 s a ensuite trois E2E verts : SC-001
   à 945 ms, capture encadrée sur 7 ms, zéro Status ; SC-006 complet ; EOF
   à 11 ms. L'ancien148 reste vert séparément à son délai de 2 s.
   La comparaison de `execute_send` conserve exactement 6283 octets et le même
   SHA-256 dans les deux sources, selon les bornes d'extraction ci-dessus.
   L'ACK a un RED runtime de 0 PASS et 11 FAIL, compilation réussie, puis
   un GREEN de 11 PASS et 0 FAIL sans warning. Huit tests cœur et trois tests
   façade prouvent bornes, propriété du lot, états refusés, premier timestamp
   durable, rejeu, sauvegarde stale ou forgée et pureté des faits.
   L'échec du deuxième UPDATE et l'échec du COMMIT annulent tout le lot sans
   réveil. Un lot changé réveille une fois ; un rejeu ne réveille pas.
   L'ancien daemon refuse en 53 ms après la seule `DaemonIdentityRequest`.
   Les régressions `--lib` post-ACK donnent 58 PASS ciblés151 et 167 PASS
   hérités, sans FAIL ou ignoré. Les dix PASS et l'ignoré d'intégration de
   l'ancien T021 ne sont pas inclus dans ce lot `--lib`.
   Le GREEN E2E post-ACK donne 4 PASS et 1 ignoré : trois cas151 et l'ancien148.
   Les permissions Claude donnent séparément 4 PASS. TaskWait publié avec
   600 s et Ack publié sont réellement exécutés. SQLite confirme la date
   privée. SC-001 prend 971 ms, capture encadrée sur 5 ms, zéro Status.
   Quatre attentes de 3600 s se ferment sur EOF en 14 ms.
   W003 est terminé : 2045 PASS, 9 FAIL et 72 ignorés sur 94 cibles Cargo
   de premier niveau, code retour101. Huit FAIL sont connus du socle150.
   Le neuvième est un oubli T025 dans l'assertion du catalogue MCP : réel27,
   attendu26. Il reste distinct des huit échecs hérités.
   La correction de ce seul littéral, de 26 à 27, donne ensuite 79 PASS,
   0 FAIL et 1 ignoré sur le filtre MCP complet. L'EOF mixte Wait3600 et
   Status muet ferme les deux sockets en 4505 ms. Aucun autre code ne change
   dans ce correctif. W004 est terminé : 2046 PASS, 8 FAIL et 72 ignorés
   sur 94 cibles de premier niveau, RC101. L'ensemble des huit noms est
   exactement celui de la référence150. Aucun échec nouveau151 ne subsiste.
   La bibliothèque daemon entière donne 1347 PASS, 0 FAIL et 14 ignorés.
   Le principal coche alors T025 : 26 tâches sur 26 avant Converge 1.

5. **Qu'est-ce qui n'a pas été vérifié ?** Le full final W004 est exécuté.
   Converge 1 avait retenu l'absence de mesure SC-009 après ACK normal.
   T027 est maintenant verte : 406 µs et 357 µs, sans changement de production.
   Le format global151 reste RC1. Le socle150 est aussi RC1, 39 fichiers et
   561 hunks ; 151 avait568 hunks avant le newline de T027. Comparer les seuls
   noms de fichiers ne prouve pas que tout l'écart est hérité. La comparaison
   réelle après newline comptait567 hunks151. Les18 corrections exactes
   dans10 fichiers ont une normalisation rustfmt byte-identique avant/après.
   Le résultat final est RC1/549 :547 corps identiques au socle et2 contextes
   anciens qualifiés par le principal. Aucun format globalPASS n'est annoncé.
   Ses huit échecs connus restent visibles ; aucune suite globalement verte
   n'est annoncée. W001 et W002 contiennent dix et douze échecs. Ils ne deviennent
   pas verts par les corrections isolées. SC-001 est vert, avec son ancrage SQLite. Tous les deltas de
   production151 sont ensuite relus ; FR-038 était le seul finding actif.
   La revue initiale du cœur était bornée à trois fichiers. Le runtime de
   production n'est pas validé par ces recettes privées. L'ACK de lecture de
   T025 est implémenté et ses onze tests sont verts. Les E2E post-ACK sont
   verts ; W003 est terminé en échec. L'assertion T025 est corrigée et le
   filtre MCP complet est vert. W004 est terminé avec les seuls huit échecs
   connus150, comparés par noms à la référence, et sans échec nouveau151.
   Aucun compteur restant n'est déduit
   de la référence. Le RED E2E lisant `acknowledged_at` dans Status relève
   une assertion erronée : cette date privée n'est pas un champ public.
   La concurrence des migrations et le compteur runtime après rollback ou
   opération sans changement de réponse/demande restent non mesurés. Le bump
   inutile du rejeu de Cancel est une réserve non bloquante, sans violation
   de FR-039. W002 est terminé en échec. La correction de la grâce a ses GREEN
   ciblés et la relecture fraîche ne trouve aucun défaut concret. La fixture
   E2E151 à 15 s est verte, avec l'ancien148 à 2 s conservées. La borne SC-001
   demeure au plus 2 s. Le principal clôt T022 avec « échecs expliqués » et
   T026. Le principal a vérifié et coché le snapshot T023, puis T024 après RED.
   Le test du backlog
   macOS rend `ECONNREFUSED` ; il ne
   prouve pas runtime `EINPROGRESS`/poll. Les exemples TaskWait et Ack publiés
   ont désormais leur preuve d'exécution E2E, distincte de la validation JSON.

6. **Qu'a-t-on conservé, et pourquoi ?** Les deux schémas et leurs migrations
   gardent leurs points d'entrée autonomes ; les réunir couplerait les ouvreurs
   du Store et de l'idempotence. Le wrapper `Store::mark_answered` reste utile
   au chemin hérité existant. Le calcul partagé du résumé évite deux règles
   de coupe différentes entre les vrais écrivains. Les sept tests protègent
   des faits distincts : migration, isolation, immutabilité, frontière UTF-8,
   atomicité et lecture pure. Aucun module, type ou dépendance nouveau n'a
   été ajouté pour le lot stockage. Le compteur commun est conservé car il
   remplace un polling répété sans consommer le canal Lineage. Le registre MCP
   unique porte la fermeture et les quotas ; deux registres auraient séparé
   des faits qui doivent changer sous le même verrou. La sonde ancienne est
   conservée pour la compatibilité prouvée. Les deux lignes `cwd` inutilisées
   sont supprimées et les deux wrappers nécessaires aux seuls tests sont
   limités à `cfg(test)`. La revue façade n'identifie aucune abstraction inutile.
   Le garde SQLite et l'accesseur de connexion sont limités aux tests. Le garde
   a deux usages réels ; la frontière C impose un contexte stable et une
   désinstallation avant libération. L'accesseur unique, absent de 150, évite
   d'ajouter un second accès pour les tests d'ACK désormais verts.
   Le wrapper JSONL compatible reste utile aux anciens appelants. Son extension
   par callback répond à FR-038 avant connect et durant poll ; le socket
   enregistré sert aussi pendant authentification. L'adaptateur du connecteur
   et le contexte local au thread évitent de modifier chaque outil. Aucune
   abstraction de connexion concurrente ou nouvelle dépendance n'est créée.
   La Condvar est membre du registre existant et utilise son état déjà présent.
   Elle conserve les réponses rapides après EOF, puis force la fermeture à
   l'expiration de la grâce. Les budgets d'entrée ouverte ne sont pas réduits.
   `save_projected` conserve exactement le timestamp ACK de la ligne durable,
   même si l'objet appelant est stale ou forgé. La méthode métier d'ACK
   conserve l'ordre d'entrée et ne change que le champ JSON privé.
   Aucune extension générique n'anticipe les sessions 152 à 158.

## T023 - périmètre du snapshot avant ACK

Le GO du principal suit la relecture EOF fraîche et les GREEN isolés des quatre
échecs nouveaux. Les 19 SHA-256 de `dependency150.json` correspondent au
worktree150 actuel. Son HEAD reste la base
`1f2c661d39a9ce106723a93ac883f76ab9fe52c6`. Le diff live limité à ces 19 chemins
est byte-identique à `dependency150.patch`, SHA-256
`2a538537622e9fe12e8fdc0fff2bd9c453ddf2708152734cfcfade4513088f92`.
Les autres documents de planification du worktree150 ne sont pas utilisés
comme une base immuable de comparaison.

Le patch cumulatif contient 33 fichiers suivis, avec150 et151. Le delta151
porte les 30 fichiers suivis différents du socle vérifié : 15 avec production,
11 de tests ou adaptations de format, trois documents et un sélecteur de
session. Le fichier T3 n'ajoute qu'un test dans son module existant. Trois
sources du cumul restent byte-identiques au socle150 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/desired_state.rs
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/fleet.rs
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/referent_control.rs

Le manifeste couvre tous les fichiers non suivis hors exclusions Git, plus
les journaux ignorés des dossiers de validation150/151. Il donne leur taille
et leur SHA-256. Les deux patches et le manifeste lui-même sont exclus de
l'inventaire récursif ; les patches ont leurs propres empreintes dans les
métadonnées. Les nouveaux documents sont portés par l'inventaire, pas par le
delta des seuls fichiers suivis. Aucun compteur de tests n'est calculé depuis
le nombre de fichiers du snapshot.

Artefacts :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/diff-151.patch
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/delta151-vs150.patch
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/snapshot-151-manifest.json

Le snapshot décrit ce point avant ACK. Il ne clôt aucune tâche par lui-même
et ne déclare pas la session terminée. Le full final après ACK reste requis.

Preuve des six tests rouges :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-workers151.log

Preuve des sept tests verts :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-core151-green-workers.log

Régressions :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/regression-store151.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/regression-idempotency151.log

Preuves de la correction réelle FR-034 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-compat-result150-relecture151.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-compat-result150-relecture151.log

Trois E2E 151 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-e2e151-sc001-sc006-eof.log

Reprise finale avec mesure SC-001 ancrée sur la capture SQLite :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-e2e151-final-sqlite.log

Régressions de façade, extraits authentiques :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/facade-regressions151-extraits.log

Reçu de compatibilité réelle :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/compat-old151.json

Compteurs, comparaison T021 et registre des six journaux finaux :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/tests-151.md

## ACK de lecture - implémentation et preuves de Phase 9

Le gate est appliqué : `acknowledge_tasks` étend `DelegationStore`. Le handler
valide toute la propriété avant les états, sous le verrou existant. Une seule
transaction écrit `json_set(payload, '$.acknowledged_at', ...)` pour 1 à 16
tâches, uniquement sans ACK existant. Le timestamp commun initial est immuable.
Les listes `acknowledged` et `already` gardent l'ordre d'entrée. Le compteur
avance une seule fois après commit si le lot change, jamais après rollback
ou rejeu. L'ACK n'appelle ni `save_projected`, ni stamp, ni projection.

`save_projected` conserve le timestamp durable. Les tests couvrent une copie
chargée avant l'ACK et les tentatives de créer, effacer ou remplacer sa date.
Le JSON ancien sans champ se décode en `None`, sérialisé en `null`, sans dates
inventées. La boîte devient `acknowledged` après sa garde terminale.
La lecture ne change ni payload hors ACK, dates métier, génération, séquence,
Lineage, messages, livraison, nettoyage ou purge. Le RED compile puis échoue
au runtime : 0 PASS / 11 FAIL. Le GREEN donne 11 PASS / 0 FAIL sans warning.

Preuves :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-ack151.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-ack151.log

T024 et T025 sont cochées par le principal. Les régressions
post-ACK `--lib` sont vertes ; les E2E donnent 4 PASS / 1 ignoré, dont trois
cas151 et l'ancien148. Les permissions Claude donnent 4 PASS séparément.
Le RED E2E de Status est une erreur de test corrigée vers SQLite en lecture
seule, sans ajout de champ public. Le GREEN exécute réellement TaskWait
publié600 puis Ack publié, et confirme le premier timestamp durable.
SC-001 prend 971 ms, capture encadrée sur 5 ms, zéro Status. L'EOF ferme les
quatre attentes3600 en 14 ms. W003 est terminé en échec : 2045 PASS, 9 FAIL,
72 ignorés et code retour101, sur 94 cibles Cargo de premier niveau.
Huit FAIL sont connus150. Le neuvième, propre à T025, est
`mcp::tests::matrice_fr009_couvre_les_quinze_cas` : catalogue réel27,
assertion attendue26. Le principal confie à la façade la correction de ce
seul littéral, de 26 à 27. Le filtre MCP complet donne ensuite 79 PASS,
0 FAIL et 1 ignoré en 4,65 s. L'EOF Wait3600 et Status muet ferme les deux
sockets en 4505 ms. Aucun autre code ne change dans ce correctif.
W004 est terminé : 2046 PASS / 8 FAIL / 72 ignorés, RC101 et 94 cibles.
Les huit noms sont exactement ceux de la référence150 ; aucun nouveau FAIL151.
La bibliothèque daemon entière est verte : 1347 PASS et 14 ignorés.
Le principal clôt alors T025 : 26 cases sur 26 avant Converge 1. W003 reste historique
en échec. Cette clôture ne valide ni installation ni runtime de production.

Preuve du correctif de catalogue :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-mcp-ack-catalogue151.log

Journal W004 terminé, avec les huit échecs connus conservés :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T022-workspace151-final.log

Preuves E2E :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-e2e-ack-champ-public151.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-e2e-ack151.log

Journal W003 terminé, conservé en échec :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T022-workspace151-final-ack.log

La décision est inscrite dans le gate de réutilisation :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/reuse-audit.md

## Snapshots finaux après ACK

Le GO final du principal autorise trois sorties distinctes. Elles complètent
T023 sans remplacer les trois preuves pré-ACK. Les 19 empreintes du socle150
et son patch importé sont revérifiés. Les HEAD150 et main restent au commit
1f2c661d39a9ce106723a93ac883f76ab9fe52c6 ; aucun code ni Git n'est modifié
par cette capture. Le diff cumulatif couvre les fichiers suivis ; le delta151
compare le socle reçu. Le manifeste inventorie séparément les fichiers non
suivis et les preuves ignorées, avec taille et SHA, hors sorties récursives.

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/diff-151-final.patch
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/delta151-vs150-final.patch
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/snapshot-151-final-manifest.json

## Converge 1 - preuve SC-009 complétée par T027

Verdict : une tâche T027 ajoutée, aucun bug de production prouvé. Le statut
revenait à In Progress, avec 26 cases cochées sur 27. Le test ouvre
RequestWait pendant `open`, puis traiter la réponse par le vrai DeliverAcked
normal. Il doit mesurer `answered` en 2 s au plus après cet ACK et vérifier
le résumé exact. Deux réponses de scopes différents partagent le même
message_id et le même destinataire ; leurs demandes ont des clés distinctes.
Un `mark_answered` direct ou un bump manuel ne constitue pas cette preuve.

Le principal confirme les 47 fichiers du gel inchangés hors tasks. Le préfixe
de 21424 octets est conservé ; seule la section Convergence est ajoutée.
Les six snapshots précédents gardent leurs empreintes. Le GO des nouveaux r2
est reçu après correction et qualification du format ; aucune capture n'est écrasée.

Estimation historique : 140–320 min initialement, puis 130–290 min.
Après T027 : 240–290 min, fin centrale annoncée à 12:44 CEST.

Reçu de la revue :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/converge151-passage1.md

## T027 - preuve SC-009 distincte et relecture Article XX

Le principal a relu et coché T027 : 27/27. Le test ciblé donne 1 PASS,
0 FAIL et 0 ignoré, en 0,07 s. Le retour après le vrai ACK normal prend
406 µs dans scopeA et 357 µs dans scopeB, sous la borne de 2 s.
Les demandes sont distinctes. Les réponses partagent message_id et destinataire.
Après la seconde réponse, le ledger contient son corps ; chaque demande garde
son propre résumé exact. Le compteur avance une fois par vrai handler et la
capacité est libérée. W0042046/8/72 reste historique avant ce test complémentaire.

| Question Article XX | Réponse T027 |
|---|---|
| Pourquoi est-ce nécessaire ? | SC-009 exige une mesure après le vrai ACK normal. Les tests séparés de résumé et de réveil ne la prouvaient pas. |
| Pourquoi cette solution est-elle simple ? | Elle réutilise la fixture et les handlers RequestWait/DeliverAcked existants. Aucun hook de production, mark_answered direct ou bump manuel. |
| Quelles hypothèses ? | L'attente lit open avant ACK. Deux scopes partagent message_id et destinataire, avec deux demandes distinctes. L'ancrage précède le handler ACK, donc la borne est conservatrice. |
| Qu'a-t-on vérifié ? | Test et journal relus : 1 PASS, 406/357 µs, résumés exacts puis stables malgré collision du ledger, epoch+1 et capacité0. La barrière active_waits est acquise après le premier snapshot open ; aucune réponse ne précède ACK. |
| Qu'est-ce qui reste non vérifié ? | Aucun modèle réel, runtime de production ou attente réelle de3600 s dans T027. Le format global reste RC1/549, avec corps et contextes anciens qualifiés. Aucun nouveau full après T027 ni total2047 déduit. |
| Qu'a-t-on conservé ? | Le seul test requis dans le module existant, ses fixtures et observateurs. Pas de helper, module, dépendance ou modification de production. |

Le format du bloc T027 est sans diff. La seule correction est le retour à la
ligne du while ; les tokens sont identiques, sans rejeu du ciblé. Le format
global151 a RC1 ; celui de150 a RC1, 39 fichiers/561 hunks. Le relevé151
avant ce newline comptait568 hunks. L'égalité des noms de fichiers ne qualifie
pas les hunks nouveaux. La comparaison réelle après newline comptait567 hunks :
547 corps identiques,18 touchant le delta151 et2 contextes à qualifier.
La façade corrige exactement les18 dans10 fichiers. Les normalisations rustfmt
avant/après sont byte-identiques ; diffcheckRC0. Le format global final reste
RC1/549 hunks. Les547 corps anciens sont conservés et les2 contextes sont
qualifiés par le principal. Aucune nouvelle exécution globale n'est déduite.

Test et mesures :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T027-ack-requestwait151.log

Source relue, sans écriture par l'owner documentaire :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/daemon/native_delegation.rs

Qualification finale des deux contextes, preuve du principal distincte de
l'annexe antérieure du journal façade : `lineage_page`,2451 octets, est
byte-identique au socle150, SHA256
`00dfbb7741b503b23f10363718d4e9e89894827abd7838253412128bf7d240ad`.
`insert_projected`,857 octets contre825, est byte-identique après le seul
retrait de `        self.wait_epoch.bump();\n`, SHA256 du corps150
`5fe1adfae142a1cdc595d687fc33197af28b7f805e787c0f59de06ec5a48ab67`.
Le bump est déjà formaté ; les styles restants viennent du contexte ancien.
Le journal façade reste intact, avec ses constats antérieurs conservés.

Captures r2 après GO du principal :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/diff-151-final-r2.patch
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/delta151-vs150-final-r2.patch
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/snapshot-151-final-r2-manifest.json
