# Tests 151 - après ACK et Converge 1

Date : 2026-10-11. Périmètre : worktree 151, socle 150 importé sans commit.
Ce relevé consolide les preuves produites. Le principal a coché 26 tâches
sur 26 après W004. Converge 1 ajoute ensuite T027. Son test distinct donne
1 PASS, 0 FAIL, 0 ignoré ; le principal le coche : état Implemented, 27/27.
W004 reste2046/8/72 avant T027 ; aucun nouveau total workspace2047 n'est déduit.
Aucun Cargo n'a été relancé pour cette consolidation.
La suite complète garde huit échecs connus150 ; aucun échec nouveau151.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/151-attente-fin-de-tache/validation/tests-151.md#extrait-1).

SC-008 est désormais mesuré sur les connexions SQLite réelles : S001 donne
2 PASS au calme, S002 donne 1 PASS sous charge de 64 attentes. Ces passes
ciblées ne remplacent pas le full final requis après ACK.

Les premières passes de bout en bout E001/E002 restent historiques. E003/E004
sont vertes après correction. E005/E006 ajoutent le scénario d'avis d'échec ;
E007 rejoue les trois E2E 151 : 3 PASS, 0 FAIL, aucun ignoré, deux tests filtrés.
Le test historique 148 n'est pas dans ce filtre.
E008/E009 renforcent ensuite l'ancrage de SC-001 sur la capture SQLite :
1 PASS ciblé, puis trois E2E verts en 12,69 s. La borne conservatrice est
de 980 ms, avec la capture encadrée sur 6 ms.
E010 rejoue les trois E2E151 après correction du délai de fixture : 3 PASS,
0 FAIL, borne SC-001 de 945 ms, capture encadrée sur 7 ms. E011 vérifie
séparément l'ancien148, dont le délai de 2 s reste conservé.

T018 est prouvée par les appels réels C001 contre l'ancien daemon.
Le principal a coché T007 et T021 après ces régressions, puis T022/T026 après
les corrections et la relecture fraîche. T023 et T024 sont ensuite cochées
par le principal, puis T025 après W004. Ces 26 tâches restent cochées ;
T027 est ajoutée après Converge 1 puis cochée sur la preuve distincte S003,
portant le total à 27 tâches cochées sur 27.

## Recette et lecture des preuves

Les passes Cargo suivent la recette privée de la session : `umask 077`,
`env -i`, HOME utilisateur, TMPDIR court sous /tmp, BRIDGET_HOME privé,
fonction de test `test-support` et cache Cargo 150. Aucun service de
production n'est utilisé.

La référence utilise :
`cargo test -p bridget-daemon --features test-support FILTRE -- --test-threads=4`.
Les passes ciblées du daemon utilisent le filtre `native151` avec `--lib`.
Le protocole utilise le filtre `native151` de `bridget-transport`.
Les régressions de stockage utilisent `store::tests --lib` et
`idempotency::tests --lib`.

Les compteurs ci-dessous proviennent des lignes `test result:` des journaux.
Pour W001/W002, les headers Cargo `Running unittests`, `Running tests/` et
`Doc-tests` délimitent 94 cibles. Seul le dernier bilan de chaque cible compte.
Les bilans de sous-processus imprimés dans leur sortie ne sont pas additionnés.
Les premiers totaux bruts restent signalés comme comptes préliminaires dans
l'historique. Les filtres se recouvrent : leurs réussites ne constituent pas un total de
tests distincts. Une compilation interrompue avant les tests ne compte pas
comme une passe rouge du comportement.

## Référence avant code

| Filtre | PASS | FAIL | Ignorés |
|---|---:|---:|---:|
| native148 | 27 | 0 | 1 |
| native149 | 78 | 0 | 0 |
| native150 | 51 | 0 | 0 |
| delegation149 | 10 | 0 | 0 |
| spec_087 | 11 | 0 | 0 |
| batch | 0 | 0 | 0 |
| Total avant code | 177 | 0 | 1 |

Les six journaux bruts et le JSON de référence donnent les mêmes compteurs.
Le filtre `batch` ne sélectionne aucun test. Il ne prouve aucune couverture
de regroupement. Les deux tests de T014 la vérifient ensuite dans V005.
Cette référence ne remplace pas la comparaison après code de T021.

Référence détaillée :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/baseline.md

Compteurs structurés :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/baseline-results.json

## T021 - régressions finales après correction de FR-034

Les six journaux G001 à G006 ont été relus. Les bilans de chaque binaire sont
additionnés par filtre, comme pour la référence. Les noms de tests et leurs
résultats `ok` ou `ignored` sont identiques entre référence et passe finale.
Aucun test vert de ces filtres ne manque ou ne devient rouge.

| Preuve | Filtre | PASS | FAIL | Ignorés | Comparaison avec la référence |
|---|---|---:|---:|---:|---|
| G001 | native148 | 27 | 0 | 1 | Identique |
| G002 | native149 | 78 | 0 | 0 | Identique |
| G003 | native150 | 51 | 0 | 0 | Identique |
| G004 | delegation149 | 10 | 0 | 0 | Identique |
| G005 | spec_087 | 11 | 0 | 0 | Identique |
| G006 | batch | 0 | 0 | 0 | Identique, aucun test sélectionné |
| Total | Six filtres | 177 | 0 | 1 | Identique |

T021 prouve SC-007 dans ce périmètre. Elle ne remplace pas la suite complète
de T022 et ne prouve pas l'ACK de lecture ajouté ensuite en Phase 9.

## Passes relues sur disque

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/151-attente-fin-de-tache/validation/tests-151.md#extrait-2).

E001 contient une réussite 148, puis un échec 151 et un arrêt SIGABRT pendant
le cas EOF. Il ne contient pas de bilan normal de suite. Aucun compteur
complet n'est inventé pour cette passe.

Les deux compilations intermédiaires de stockage ont échoué avant les tests
sur des symboles ou un type de façade encore en édition. Elles restent un
historique de coordination. Elles ne prouvent aucun échec du stockage.

La mesure de 1988 ms dans E007 allait de la libération de l'enfant à ready.
Cet intervalle était plus large que la capture requise par SC-001.
E008/E009 lisent la base de recette depuis une connexion extérieure en lecture
seule. Le début du dernier SELECT encore NULL donne une borne antérieure à la
capture. Le premier SELECT non NULL ferme son encadrement, ici 6 ms.
La borne conservatrice de capture -> ready est inférieure à 2 secondes.
Cette observation de recette ne modifie aucun code de production.

## SC-008 - mesure au calme et sous charge

S001 observe les SELECT pertinents avec `sqlite3_trace_v2`, sur les connexions
effectivement utilisées par TaskWait et RequestWait. Chaque attente fait trois
SELECT : au début, après environ 5 secondes, puis après environ 10 secondes.
Les epochs restent respectivement à 2 et 0. Les réservations sont libérées ;
le payload de tâche reste intact et `working`. La demande reste `open`, avec
son échéance métier de 3600 secondes. Aucun réveil métier n'est ajouté pour
obtenir cette mesure. Le garde de trace est limité à `cfg(test)`.

S002 garde un écrivain actif jusqu'après les deux contrôles. La barrière attend
son premier save. Les relevés passent de 1 à 2 autour de Status, puis de 2 à 5
autour de Cancel ; le total est 6. L'arrêt et le join viennent après les mesures.
L'écrivain s'arrête aussi si son canal de commande est déconnecté. Avec 64
attentes, Status prend 2430 microsecondes et Cancel 19056 microsecondes.
Ces deux mesures restent sous la borne de 500 ms du test.

Ces chiffres décrivent une recette privée avec données synthétiques. Ils ne
décrivent pas la charge des services de production.

## T022 - premier passage complet, non final

Les 94 cibles de W001 totalisent 2029 PASS, 10 FAIL et 72 ignorés.
Le premier compte préliminaire brut additionnait 97 lignes de bilan :
2032 PASS, 10 FAIL et 72 ignorés. Il comptait trois réussites de sous-processus
une seconde fois. Le compte corrigé retient le dernier bilan de chaque cible.
Le principal confirme le code retour 101. Les dix échecs restent visibles ;
aucun n'est transformé en réussite ou retiré du relevé.

Deux échecs apparaissent avec les ajouts 151 :

| Test | Fait observé | Correction et preuve ciblée |
|---|---|---|
| `spec094_claude_recoit_la_liste_fermee_des_outils_bridget_autorises` | Liste fermée attendue à 20 outils, réelle à 22 avec les deux attentes. | V013 : 1 PASS, liste fermée exacte corrigée. |
| `exemples_skill_envoient_repondent_et_cloturent_sans_service_compagnon` | Extraction de tout le skill : 9 objets JSON au lieu des 6 du banc de communication. | V014 : 1 PASS et 1 ignoré, six exemples138 exécutés et RequestWait151 exécuté. |

Les huit autres échecs sont hérités de 150 selon la comparaison transmise par
le principal :

- `cli_uuid_herite_adresse_active_sans_pouvoir_usurper_ni_suivre_une_reponse_ephemere`
- `client_reference_cli_mcp_rejouent_le_meme_record_et_refusent_les_divergences_sans_mutation`
- `binaire_et_outil_mcp_partagent_les_quatre_issues_d_une_reponse_liee`
- `spec094_mcp_modifie_uniquement_sa_propre_presence`
- `deux_processus_mcp_meme_identite_et_meme_cle_restent_isoles_par_instance`
- `noms_humains_et_crash_ne_changent_ni_scope_ni_canon_ni_instance_du_wrapper`
- `cli_demande_mcp_repond_annulation_et_timeout_ne_rouvrent_ni_ne_dupliquent`
- `spec098_pont_expose_remet_repond_renouvelle_et_retire`

La correction du banc 089 réutilise `examples(section)` et coupe au titre
stable du contrat 138. Elle conserve les six objets, leur ordre, leurs noms,
leurs arguments et toutes les assertions de remise, rejeu et clôture. Le skill
entier doit toujours contenir exactement neuf objets. Les trois objets 151
ont une assertion complète séparée ; aucun objet inconnu n'est filtré.

RequestWait est ajouté au scénario après la réponse corrélée connue, sous
l'identité du parent initial. Seul son placeholder `request_id` est remplacé.
Les assertions portent sur `answered`, l'absence d'expiration, l'id et l'auteur
de la réponse, son résumé et un ledger inchangé. V014 exécute réellement ce
RequestWait publié : 1 PASS, 0 FAIL et 1 ignoré, code retour 0 confirmé par
le principal. TaskWait et Ack publiés étaient alors en attente du banc E2E ;
E012 prouve maintenant leur exécution réelle. Leur parse
et leur assertion JSON ne constituent pas une preuve d'exécution.

Source du banc corrigé :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/tests/core_089_skill_test.rs

## T026 et T023 - corrections EOF prouvées, snapshot autorisé

Le principal rapporte la relecture de tous les deltas de production 151 par
Explorer. Le défaut alors identifié était FR-038. R007 le reproduit avec Wait3600 et
Status déjà démarrés et authentifiés. L'EOF ferme les deux connexions backend,
mais la façade finit en 10,000712583 secondes, au-delà de la limite de 6 secondes.
Les anciens tests EOF verts couvraient d'autres scénarios ; ils ne prouvaient
pas ce mélange d'appels. T026 a donc été réouvert sur ce complément.

V010 rend le même scénario vert : fin affichée à 0 ms, donc inférieure à 1 ms,
avec les deux connexions backend fermées. Le registre couvre les sockets de
tous les appels MCP. Un contexte local au thread relie chaque appel au même
registre. Le callback générique du connecteur enregistre le socket avant
connect ; le socket enregistré reste fermable durant l'authentification.
L'EOF ferme tous les appels. Une notification d'annulation nonwait laisse
l'opération finir et masque seulement sa réponse ; V010 le vérifie pendant
la connexion, l'authentification et l'opération.

R008 puis V012 prouvent le callback avant connect avec un backlog Unix réel
plein. Rouge : aucune observation. Vert : une observation et une annulation
synthétique `Interrupted` avant dial. Sur macOS, le noyau rend immédiatement
`ECONNREFUSED`. Le test n'observe donc pas `EINPROGRESS` ni sa boucle de poll.
Cette branche est couverte par lecture de code seulement, pas par une mesure
runtime. Aucun nouveau budget, retry ou dial n'est ajouté : l'échéance absolue
reste la même ; le callback permet de vérifier l'annulation durant le poll.

V011 conserve les 19 tests du connecteur. V013 et V014 corrigent séparément les
deux fixtures nouvelles du premier full. P001 confirme que `execute_send`
reste identique au socle150 : 6283 octets, même SHA-256, sans le LF final.
La relecture RO après correctif était terminée, sans défaut confirmé. Le
principal avait coché T012/T026 sur ces preuves et la revue fraîche. La suite
complète W002 révèle ensuite quatre échecs nouveaux :

| Test | Échec observé dans W002 | État |
|---|---|---|
| `cli_et_mcp_reels_lisent_les_memes_octets_sans_droit_supplementaire` | `Option::None` à la ligne 261 du banc contenu089. | R009 confirme la perte EOF ; V016 rend les sept tests verts, banc inchangé. |
| `vrai_daemon_et_clients_n_utilisent_que_namespace_prive` | `Option::None` à la ligne 415 du banc isolation089. | V017 : huit PASS et un ignoré après grâce, banc inchangé. |
| `partial_or_invalid_t3_proof_never_falls_back_to_pid` | Le cas `endpoint_only` imprime l'échec de son sous-processus `isolated_identity_entrypoint`, sur `Option::None` à la ligne 99. | V018 : un PASS et un ignoré, banc inchangé. Le FAIL imbriqué n'est pas un treizième échec de cible. |
| `native151_e2e_deleguer_puis_attendre_et_message_unique_apres_expiration` | D001 observe `failed`, `result: null` et `error: "échéance Claude dépassée"`. | Délai de fixture de 2 s expiré avant capture. E010 : trois E2E151 verts à 15 s ; E011 conserve l'ancien148 à 2 s. |

Sources des quatre bancs :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/tests/core_089_content_test.rs
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/tests/core_089_isolation_test.rs
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/tests/t3_session_identity_env.rs
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/tests/native_delegation_e2e.rs

R009 confirme empiriquement la fermeture prématurée des appels ordinaires.
Status est authentifié et reçu avant EOF ; le backend répond après 100 ms.
La façade finit à 0 ms et ne transmet que initialize id1. La réponse id2 est
absente. Les trois clients historiques ferment stdin après leur envoi.

Le principal autorise une grâce de 4,5 s sur EOF pour les appels ordinaires.
Les attentes doivent être annulées sans attendre cette grâce. Une condition
variable (Condvar) dans le registre MCP attend la fin de ses membres, avec
le même verrou pour le prédicat et les notifications de sortie. Ce choix
réunit la compatibilité FR-034 et la fermeture en au plus 6 s de FR-038.
Le compteur `WaitEpoch` du daemon ne convient pas : il lierait ce registre
de processus MCP au magasin et aux réveils métier du daemon. Aucun budget
de connexion ou d'opération n'est réduit tant que l'entrée reste ouverte.
V015 prouve maintenant la grâce compatible : Status rend id2 en 105 ms,
avec backend confirmé ; Wait3600 et Status muet finissent en 4504 ms, avec
les deux sockets fermés. Les 76 tests MCP passent, avec un ignoré. V016,
V017 et V018 rendent verts les trois bancs clients inchangés. Les trois
fixtures unitaires MCP à entrée finie ont aussi leur forme originale restaurée.
V019 conserve les 19 tests du connecteur. Ces filtres se recouvrent ; leur
addition ne formerait pas un total de tests distincts.

D001 est une lecture réelle de la base privée E2E et de ses journaux.
La tâche est nettoyée mais porte `failed`, un résultat NULL et l'erreur de délai.
La fixture avait `notify_timeout_secs: 2` ; son enfant attendait encore le
fichier de libération. La correction réutilise le constructeur existant :
ancien148 garde 2 s, les trois151 ont 15 s. E010 donne 3 PASS, 0 FAIL et
aucun ignoré, avec deux tests filtrés. SC-001 mesure 945 ms, capture encadrée
sur 7 ms, zéro Status avant résultat ; la borne reste au plus 2 s. SC-006
confirme pending -> claimed -> delivered, un avis unique et le nettoyage
durable. EOF ferme quatre attentes de 3600 s en 11 ms. E011 rejoue séparément
l'ancien148 : 1 PASS, 0 FAIL, quatre filtrés. Le délai de notification et la
mesure de réveil restent distincts.

W002 est terminé : 2030 PASS, 12 FAIL et 72 ignorés, code retour 101,
neuf cibles en échec. Ses 98 lignes de bilan brutes donnaient le compte
préliminaire de 2033 PASS, 13 FAIL et 72 ignorés. Trois PASS de sous-processus
et le FAIL imbriqué d'identité étaient comptés deux fois. Les huit FAIL
hérités restent inchangés ; les quatre nouveaux ont ensuite leurs GREEN isolés.
Les GREEN ciblés FR-038 sont historiques et ne prouvent pas la compatibilité
des clients qui ferment stdin. Les nouveaux GREEN compatibles comblent cet
écart. La relecture fraîche ne trouve aucun défaut concret ; la compilation
normale est sans warning selon le principal. Il clôt T022 avec « échecs
expliqués », clôt T026 et autorise le snapshot T023. W002 reste en échec.
Le prochain full final est obligatoire après ACK ; aucune répétition complète
intermédiaire n'est requise par ce verdict. T023 restait alors ouverte jusqu'au
relevé du snapshot. Le principal l'a ensuite vérifié et coché ; cet owner
ne modifie aucune case.

## FR-034 - correction d'une attente de test erronée

Le contrôle initial a supposé que le statut 150 rendait le texte de `result`
dans tous les états. La relecture du vrai `task_view`, au commit de base 150
`1f2c661d39a9ce106723a93ac883f76ab9fe52c6`, montre le contraire : le texte
n'est rendu que dans `result_available`. Dans les autres états, la clé
historique est présente et sa valeur est `null`.

R005 a vu en échec cette attente erronée. V008 a ensuite donné 3 PASS après
un correctif qui rendait trop largement le texte. Cette passe reste historique ;
elle ne valide pas FR-034. Le succès d'une assertion incorrecte ne prouve
pas la compatibilité.

L'assertion correcte a ensuite été vue rouge dans R006 : en état `queued`,
le statut rendait le résultat privé au lieu de `null`. Le code et le test
sont corrigés dans V009 : trois tests de vue verts. La règle 150 est conservée.
L'outil d'attente nouveau continue à omettre le champ `result` entier.

## T018 - vrai ancien daemon

C001 est un reçu JSON avec `status: pass`. Il porte le commit ancien
`1f2c661d39a9ce106723a93ac883f76ab9fe52c6`, les empreintes SHA-256 des deux
binaires et les captures complètes d'un relais Unix transparent.

| Appel réel | Durée | Résultat |
|---|---:|---|
| bridget_task_wait, délai demandé 3600 s | 44,710 ms | wait_unsupported |
| bridget_request_wait, délai demandé 3600 s | 54,874 ms | wait_unsupported |

Les deux captures contiennent chacune exactement un `DaemonIdentityRequest`,
33 octets avec LF. Leurs réponses sont des rapports anciens sans `features`.
Les octets reçus et relayés sont identiques dans les deux sens. Aucune trame
151 n'est transmise ; les deux refus prennent moins de 5 secondes.

Le reçu confirme l'absence des deux processus après arrêt et le retrait du
namespace, du worktree ancien et du client temporaire. Le cache Cargo partagé
reste conservé. Le runtime de recette utilise un HOME privé et un environnement
fermé ; il n'utilise aucun service de production. C002 fournit la reproduction.

## Résultats transmis par l'agent façade

Ces passes ont été capturées par l'outil d'exécution, sans journal intégral.
L'agent façade a ensuite archivé leurs extraits authentiques avec les identifiants
de session, sans rejeu. Cette consolidation a relu cet archivage, F005.

| Identifiant | Filtre ou montage | PASS | FAIL | Ignorés | Session d'outil |
|---|---|---:|---:|---:|---|
| F001 | native151_facade, passe de l'agent façade | 14 | 0 | 0 | 86627 |
| F002 | mcp::tests | 73 | 0 | 1 | 94795 |
| F003 | client::security_tests | 19 | 0 | 0 | 4682 |
| F004 | Montage : Spec091, Spec094 Codex, native148 Claude, configurations utilisateur | 5 | 0 | 1 | 85438 |

F004 comprend 2 PASS et 1 ignoré pour Spec091, puis trois passes de 1 PASS.
V005 inclut aussi le nouveau test `native151_facade_progression_64_attentes`.
Son filtre par le mot `native151_facade` contient ainsi 15 tests réussis.
F001 et V005 décrivent deux passes à des moments différents.

## ACK de lecture - RED puis GREEN

A001 compile avant production et échoue au runtime. A002 prouve huit cas cœur :
premier timestamp commun, durable et rejeu ; bornes et lot de seize ; propriété
de tout le lot avant readiness ; sept états refusés dont cancelled ; rollback
au deuxième UPDATE et au COMMIT différé sans réveil ; sauvegarde stale ou
forgée ; pureté payload, messages, livraison, Lineage et purge ; ancien JSON
sans champ décodé et `null` sérialisé sans dates inventées.

Trois cas façade couvrent le schéma fermé, les bornes et annotations, l'ordre
des trames et listes, les refus et le quota nonwait. Avec quatre attentes en
vol, ACK reste disponible. L'ancien daemon refuse en 53 ms et ne reçoit que
`DaemonIdentityRequest`. Aucun message151 ne lui est envoyé.

Les régressions post-ACK `--lib` sont distinctes de T021 : native151 58 PASS ;
native148 23 ; native149 72 ; native150 51 ; delegation149 10 ; spec_087 11 ;
batch 0. Chaque filtre a 0 FAIL et 0 ignoré. Le total hérité est 167 PASS.
L'ancien T021 de 177 PASS et 1 ignoré comprenait aussi dix PASS d'intégration
(148 quatre, 149 six) et l'ignoré148, hors de ce lot `--lib`.

| Preuve | Filtre post-ACK `--lib` | PASS | FAIL | Ignorés |
|---|---|---:|---:|---:|
| H001 | native151 | 58 | 0 | 0 |
| H002 | native148 | 23 | 0 | 0 |
| H003 | native149 | 72 | 0 | 0 |
| H004 | native150 | 51 | 0 | 0 |
| H005 | delegation149 | 10 | 0 | 0 |
| H006 | spec_087 | 11 | 0 | 0 |
| H007 | batch | 0 | 0 | 0 |
| Total hérité, hors H001 | Six filtres | 167 | 0 | 0 |

Le RED E2E `red-e2e-ack-champ-public151.log` donne 0 PASS / 1 FAIL : Status
rend bien la boîte `acknowledged`, mais aucune date `acknowledged_at` publique.
L'assertion demandait un champ hors contrat. TaskWait publié600 et la chaîne
pending/claimed/delivered avec nettoyage ont réussi avant cette assertion.
La correction de test observe la date privée via SQLite en lecture seule.
E012 donne 4 PASS / 1 ignoré, dont trois cas151 et l'ancien148. TaskWait
publié600 puis Ack publié sont réellement exécutés. La date est confirmée.
SC-001 prend 971 ms avec capture encadrée sur 5 ms et zéro Status. L'EOF
ferme quatre attentes3600 en 14 ms. V020 donne séparément 4 PASS pour les
permissions Claude. Aucune régression métier ACK n'est déduite de R010.
Le workspace W003 est terminé : 2045 PASS, 9 FAIL et 72 ignorés sur 94 cibles
de premier niveau, RC101. Huit FAIL sont connus du socle150. Le neuvième est
`mcp::tests::matrice_fr009_couvre_les_quinze_cas`, qui conserve l'attente26
alors que le catalogue fermé a27 outils après ACK. Cet oubli T025 est distinct
des échecs hérités. Le principal confie à la façade la correction de ce seul
littéral après le full. La façade change l'attente de 26 à 27, sans autre
changement de code. V021 donne 79 PASS / 0 FAIL / 1 ignoré sur MCP complet.
L'EOF mixte Wait3600/Status muet ferme les deux sockets en 4505 ms.
W004 est terminé : 2046 PASS / 8 FAIL / 72 ignorés, RC101 sur 94 cibles.
Les huit noms sont exactement ceux de la référence150. Aucun nouvel échec151
ne subsiste. La bibliothèque daemon entière donne 1347 PASS et 14 ignorés.
W003 ne devient pas vert. Le principal a coché T025, portant le total à 26/26.

Référence des huit noms, relue indépendamment :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation/specs/150-socle-delegation/validation/tests-150.md

## Tâche vers preuve

« Rouge puis vert » décrit la preuve du test demandé ; une case n'est cochée
que lorsque son résultat est prouvé. Les tests regroupent plusieurs assertions.

| Tâche | État à cette consolidation | Preuve et résultat |
|---|---|---|
| T002 | Cochée | R001 puis V001 : quatre tests de protocole verts. |
| T003 | Cochée | V005 : trois tests `native151_attente_compteur_*`, dont insert/save, annulation humaine et purge après commit. |
| T004 | Cochée | Six résumés vus rouges dans R002, puis sept verts dans R003 et V005. V006/V007 : demandes suivies et idempotence vertes. |
| T005 | Cochée | Sonde annoncée/ancienne rouge dans R002, verte dans V005. Budget 3 s vert. V001 décode l'ancien rapport sans features. |
| T006 | Cochée | Trois tests `native151_vue_*` rouges dans R003 puis verts dans V005 : table, frontières UTF-8 et anciennes clés. |
| T007 | Cochée par le principal | V005 prouve la première extension. R006 puis V009 : règle réelle 150 conservée, trois tests de vue verts. G001/G002 : régressions 148/149 vertes. V008 ne valide pas FR-034. |
| T008 | Cochée | Table de boîte et statut sans écriture rouges dans R003, verts dans V005. |
| T009 | Cochée | V005 : deux tests de boîte, avec phases absentes, orphelines et accusées. |
| T010 | Cochée | Cinq tests d'attente initiaux rouges dans R003, verts dans V005. La passe verte ajoute la course de publication répétée 100 fois. La preuve du message unique de bout en bout reste portée par T013. |
| T011 | Cochée | V005 : retour immédiat, réveil sans filet, expiration/pause, refus et capacité, fermeture du client, course de publication. |
| T012 | Cochée | R002 puis V005 : schéma, bornes, sonde, montage, quatre attentes avec contrôles disponibles. S002 renforce la progression à 64 attentes : écrivain actif durant Status/Cancel. F002/F003/F004 : régressions transmises. |
| T013 | Preuves vertes complétées ; cochage au principal | W002 a un FAIL de fixture expliqué par D001. E010 : trois E2E151 verts, SC-001 à 945 ms, zéro Status, SC-006 complet, EOF à 11 ms. E011 : ancien148 vert à 2 s conservées. |
| T026 | Cochée par le principal après correction | R009 puis V015 : grâce EOF compatible, id2 conservé en 105 ms, cas bloqué fermé en 4504 ms. V016/V017/V018 : trois bancs clients inchangés verts. Relecture fraîche sans défaut concret. |
| T014 | Cochée | R003 puis V005 : trois résultats dans une réponse ordonnée et regroupement des remises natives en famille Message. |
| T015 | Cochée | R002 puis R004 montrent les échecs initiaux du stockage et des quatre scénarios daemon. V005 les rend verts, avec schéma et transmission façade. |
| T016 | Cochée | V005 : états/résumé et lecture pure, refus/rappel après coupure, réveils d'annulation et de guichet sans filet. V006 : instantané filtré et premier résumé stable. |
| T017 | Cochée | R002 puis V005 : schéma et bornes de request_wait ; R004/V002 puis V005 : transmission des contrats et annulation de connexion réelles. |
| T019 | Cochée | Trois documents relus contre le contrat : outils, délais, résumés, pureté, refus, annulation, doublon FR-030. A001/A002 prouvent les tests ACK ; E012 exécute les exemples publiés. W004 final terminé. |
| T020 | Cochée | ADR présente, statut Accepté, décisions finales R1 à R14 et raisons du compteur commun et du résumé figé. |
| T018 | Prouvée, cochée par le principal | C001/C002 : vrai binaire ancien, deux refus en 44,710/54,874 ms, seules sondes anciennes transmises, nettoyage observé. |
| T021 | Cochée par le principal | G001 à G006 : 177 PASS, 0 FAIL, 1 ignoré ; compteurs, noms et résultats identiques à la référence. |
| T022 | Cochée par le principal, échecs expliqués | W001 : 2029 PASS, 10 FAIL, 72 ignorés ; W002 : 2030 PASS, 12 FAIL, 72 ignorés. Chaque suite compte 94 cibles, RC101. Huit FAIL hérités, quatre nouveaux corrigés isolément. W002 reste en échec ; full final après ACK obligatoire. |
| T023 | Cochée par le principal | Snapshot vérifié : deux SHA de patch concordants, 19 SHA du socle et patch importé identiques, six réponses XX finalisées. |
| T024 | Cochée par le principal | A001 : compilation réussie puis RED runtime 0 PASS / 11 FAIL avant production. |
| T025 | Cochée par le principal | A002 : 11 PASS / 0 FAIL. Régressions lib vertes, 58 ciblés151 et 167 hérités ; E012 : E2E 4 PASS / 1 ignoré, V020 : permissions Claude 4 PASS. V021 : MCP 79 PASS / 1 ignoré. W004 : 2046 PASS / 8 FAIL / 72 ignorés, tous les huit connus150 et aucun FAIL151 nouveau. |
| T027 | Cochée par le principal | CV001 relève le manque ; S003 prouve SC-009 : vrai DeliverAcked normal, open avant ACK, 406/357 µs, résumés exacts et stables entre scopes, capacité libérée. |

La première consolidation a porté le total à 18 des 26 cases, dont T001.
Elle a ajouté 17 coches. T018 a ensuite été prouvée et cochée par le principal.
Le principal a ensuite coché T007 et T021, portant le total à 21 cases.
Après les preuves SC-008/FR-038 et la relecture indépendante fraîche, le
principal confirme T012/T026 cochées et 22 cases sur 26. T022 à T025 restent
ouvertes. Ce relevé ne reconstitue pas les changements de cases intermédiaires.
W002 réouvre ensuite T026 ; le principal confirme 21 cases sur 26, avec
T022/T023/T024/T025/T026 ouvertes. T012 reste cochée.
Le présent complément documentaire ne modifie aucune case.
Après les GREEN compatibles et E010/E011, le principal clôt T022/T026 et
confirme 23 cases sur 26. T023/T024/T025 restent ouvertes. Il donne ensuite
le GO du snapshot T023 ; cet owner ne modifie toujours aucune case.
Il a ensuite vérifié et coché T023, puis coché T024 sur le RED ACK. T025
restait ouverte jusqu'à W004. Le principal l'a ensuite cochée : 26/26.
Cette consolidation ne change aucune case et conserve les snapshots T023.

## T023 - snapshot autorisé avant ACK

Le socle150 est vérifié contre le reçu importé : même HEAD de base, 19
empreintes concordantes et patch importé byte-identique au diff live limité
à ces chemins. Le cumul contient 33 fichiers suivis ; le delta151 en contient
30 différents du socle, dont 15 avec production et 11 de tests ou formats.
Les trois sources150 restantes sont byte-identiques. Le manifeste donne les
empreintes, tailles, catégories et limites exactes de ce snapshot.

Le delta concerne les fichiers suivis. Les documents nouveaux ont leur entrée
dans l'inventaire complet des non-suivis hors exclusions Git. Les journaux
ignorés de validation150/151 ont un inventaire séparé. Les deux patches et
le manifeste lui-même sont exclus de cet inventaire pour éviter la récursion.
Le snapshot n'est pas un résultat de test. W002 reste à 2030 PASS, 12 FAIL
et 72 ignorés ; ses quatre FAIL nouveaux ont des corrections vertes isolées.
Le full final après ACK reste obligatoire.

Artefacts du snapshot :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/diff-151.patch
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/delta151-vs150.patch
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/snapshot-151-manifest.json

## Registre des journaux

| Identifiant | Chemin absolu |
|---|---|
| R001 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-T002.log |
| V001 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-T002.log |
| R002 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-workers151.log |
| R003 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-core151-green-workers.log |
| R004 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-core151-red-request.log |
| V002 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-151-first.log |
| V005 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-151-complet.log |
| V006 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/regression-store151.log |
| V007 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/regression-idempotency151.log |
| E001 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/e2e-151.log |
| E002 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/e2e-151-expiration-r1.log |
| E003 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/e2e-green151-extraits.log |
| E004 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/e2e-eof-151-green-facade.log |
| F005 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/facade-regressions151-extraits.log |
| R005 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-result-historique151.log |
| V008 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-result-historique151.log |
| R006 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-compat-result150-relecture151.log |
| V009 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-compat-result150-relecture151.log |
| E005 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-e2e-sc006151.log |
| E006 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-e2e-sc006151.log |
| E007 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-e2e151-sc001-sc006-eof.log |
| C001 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/compat-old151.json |
| C002 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/compat-ancien-daemon.md |
| G001 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T021-final-native148.log |
| G002 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T021-final-native149.log |
| G003 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T021-final-native150.log |
| G004 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T021-final-delegation149.log |
| G005 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T021-final-spec_087.log |
| G006 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T021-final-batch.log |
| E008 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-e2e-sc001151-sqlite.log |
| E009 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-e2e151-final-sqlite.log |
| S001 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/SC008-151-calme-final.log |
| S002 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/SC008-151-charge.log |
| W001 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T022-workspace151.log |
| R007 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-eof-statut151.log |
| R008 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-fd-connect151.log |
| V010 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-mcp-eof151-final.log |
| V011 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-client-eof151.log |
| V012 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-transport-eof151.log |
| V013 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-claude-catalogue151.log |
| V014 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-skill-publications151.log |
| P001 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/execute-send-eof151-sha.log |
| W002 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T022-workspace151-post-eof.log |
| R009 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-eof-nonwait151.log |
| D001 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/diagnostic-e2e-deadline151.log |
| V015 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-mcp-final.log |
| V016 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-core_089_content_test.log |
| V017 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-core_089_isolation_test.log |
| V018 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-t3_session_identity_env.log |
| V019 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-client.log |
| E010 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-e2e.log |
| E011 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-e2e148.log |
| A001 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-ack151.log |
| A002 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-ack151.log |
| H001 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T025-final-native151.log |
| H002 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T025-final-native148.log |
| H003 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T025-final-native149.log |
| H004 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T025-final-native150.log |
| H005 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T025-final-delegation149.log |
| H006 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T025-final-spec_087.log |
| H007 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T025-final-batch.log |
| R010 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-e2e-ack-champ-public151.log |
| V020/E012 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-e2e-ack151.log |
| W003 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T022-workspace151-final-ack.log |
| V021 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-mcp-ack-catalogue151.log |
| W004 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T022-workspace151-final.log |
| CV001 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/converge151-passage1.md |
| S003 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/T027-ack-requestwait151.log |

## Limites et contrôle documentaire

Ce relevé ne valide ni le binaire de production, ni un redémarrage, ni une
installation. Les anciennes passes rouges restent conservées.

La relecture du stockage confirme la transaction et les notifications après
commit. Elle relève un bump inutile au rejeu de Cancel, sans violation de
FR-039 : réserve non bloquante. La concurrence des migrations n'a pas été
testée. Les réveils des chemins de réponse/demande après rollback ou opération
sans changement ne disposent pas d'une mesure runtime spécifique. Le test de
rollback du résumé prouve l'atomicité des faits, pas ces mesures du compteur.
La relecture complète des deltas de production151 a trouvé le seul FR-038 actif.
Sa correction dispose maintenant des preuves ciblées vertes. Le test macOS
du backlog ne couvre pas runtime `EINPROGRESS`/poll. La relecture RO après
correctif n'avait confirmé aucun défaut. Le full post-EOF est terminé avec
quatre FAIL nouveaux. R009 confirme la perte EOF des trois clients ordinaires ;
D001 observe séparément l'échéance E2E trop courte. V015 à V019 et E010/E011
prouvent leurs corrections isolées. Le principal clôt T022/T026 avec les
échecs expliqués, sans dire W002 verte. Le snapshot T023 est terminé et
vérifié par le principal. W003 après ACK reste historique en échec ; W004 est terminé.
Les trois artefacts T023 pré-ACK ont leurs SHA inchangés après W003. Les
snapshots finaux sont écrits sous trois nouveaux noms après GO final.
Aucun artefact T023 n'est remplacé.

Les quatre documents de T019/T020 décrivent une tâche purgée comme un refus
`task_unavailable`, sans ajouter un état de boîte. La documentation de
`acknowledged` était anticipée ; A002 apporte désormais onze tests ACK verts.
Ces preuves ciblées ne rendent ni W001 ni W002 verts.

Contrôle documentaire initial : `git diff --check` sur les fichiers suivis
a rendu le code 0. Les cinq documents de ce complément sont nouveaux pour Git ;
leur contrôle avec `--no-index --check` ne signale aucune erreur d'espacement.
Son code 1 indique leur différence avec le fichier vide. La spécification
et le modèle ne présentent plus la purge comme un état de boîte.
La comparaison complète de la première consolidation prouve que seules les
17 cases autorisées avaient changé. Le complément documentaire conserve les tâches.
Les chemins de preuve du rapport existent. Les six réponses Article XX agrègent
le stockage, le cœur et la façade, avec l'ACK implémenté et testé en Phase 9.
Les régressions, E2E et full sont rapportés séparément. W003 reste en échec,
sans assimiler l'assertion T025 aux huit FAIL connus150. V021 prouve sa correction
ciblée ; W004 final conserve huit échecs connus150 et aucun nouveau151.
Le principal a coché les 26 tâches ; aucune suite globalement verte n'est annoncée.
Converge 1 ajoute ensuite T027. Le principal la coche après S003 : Implemented, 27/27.

## Snapshots finaux, distincts de T023

Les trois SHA pré-ACK restent inchangés. La référence150 conserve ses 19 SHA
et son patch importé exact. Les HEAD150 et main ne changent pas pendant la
capture. Les scopes et inventaires, tailles et SHA sont dans le manifeste final.

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/diff-151-final.patch
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/delta151-vs150-final.patch
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/snapshot-151-final-manifest.json

## Converge 1 - SC-009 après ACK normal prouvée

CV001 reste l'archive d'une tâche ajoutée et aucun bug de production prouvé.
S003 mesure RequestWait `answered` en406 µs puis357 µs après le vrai traitement
DeliverAcked, bien sous2 s. L'ancrage est conservateur : avant le handler ACK.
L'attente a lu open avant la barrière ; aucune réponse ne précède ACK. Les
deux scopes partagent message_id et destinataire, avec des demandes distinctes.
Le second corps remplace la clé du ledger ; chaque résumé demeure exact.
Epoch+1 est émis par le seul handler, capacité0 après retour. Test, journal,
stabilité et barrière sont relus. Le principal coche T027 :27/27.

Format du bloc T027 : diff vide, seule correction newline du while à tokens
identiques, ciblé non rejoué. Format global151 RC1 ;150 RC1 avec39 fichiers et
561 hunks ;151 avait568 hunks avant newline. La comparaison des seuls noms
ne prouve pas tout l'écart hérité. La comparaison après newline donne567
hunks. La façade corrige exactement les18 du delta151 dans10 fichiers ;
normalisations rustfmt avant/après byte-identiques et diffcheckRC0.
Le format global final reste151 RC1/549 hunks :547 corps identiques150 et
2 contextes anciens qualifiés par le principal. Aucun format globalPASS.
lineage_page est byte-identique150 sur2451 octets, SHA256
`00dfbb7741b503b23f10363718d4e9e89894827abd7838253412128bf7d240ad`.
insert_projected a857 octets contre825, identiques après retrait de la seule
ligne du bump déjà formaté ; SHA256 du corps150
`5fe1adfae142a1cdc595d687fc33197af28b7f805e787c0f59de06ec5a48ab67`.
Cette preuve du principal complète l'annexe façade sans l'écraser.
T027 ne lance aucun modèle réel,
n'observe ni la production ni une attente3600 réelle. W004 demeure2046/8/72
avant T027 ; le ciblé distinct n'est pas ajouté à ce total.

Le principal confirme les 47 fichiers gelés inchangés hors tasks. Son préfixe
historique est intact. Les six snapshots existants gardent leurs SHA.
Le GO des captures r2 est reçu après correction et qualification du format.
Les six captures précédentes sont préservées à l'octet près.

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/diff-151-final-r2.patch
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/delta151-vs150-final-r2.patch
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/snapshot-151-final-r2-manifest.json
Ces vérifications ne remplacent aucun test Rust.
