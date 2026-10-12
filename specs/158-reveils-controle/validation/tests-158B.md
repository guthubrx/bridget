# Tests158B — exécution ROOT

## REDde surfaces T102/T114

Le12octobre2026 à11:04:30UTC, filtre wake158b_mcp_,1Cargo nice10/jobs2,namespaceprivé. 51.33s compilation48.20s/assertions3.03s ;0PASS/2FAIL attendus,RC101. bridget_wake absent du catalogue et missed inconnu du vrai deserializer, pas panneinfra ni assertionfutureinventée. Sourcesstables218hashes. Deux tests maximum exécutés simultanément ; l’optionthreads2 n’était pas explicite sur ce premier lot. Elle sera explicite sur les prochains lots.

Ce REDprouve seulement les surfaces. Les10groupes de domaine, les30régressions de référence au FULL et tous les GREEN restent à faire. T102/T114 restent ouvertes. Reçu et journal dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/validation/T102-T114-surfaces-RED.json ;logSHAe3bc2a5c7b748b1ee956dfd903b6db7bfe1504d230f720684dba8c26c332f992. Aucun succèsBglobal annoncé.


## Types de messagesT103

REDde compilation8.58s/RC101,18symboles/variantes absents : aucun test métier exécuté ni prétendu. Après typesfermés, GREEN2PASS/0FAIL/0ignored,19.4s/RC0,threads2explicitement/sourcesstables. Fermeture objet/champs d’autorité/null, formesself-agent-thread/interval-at/missed, anciencodecTypes préservé etWakeResult interditattach. LogGREENsha f7c1e27652c267b023d10ddd243a057b5a2229d1232fbdbd48372a2e6c9cf004. Les bornes60/2048/16/128SQL etFrozenWake restent à vérifier ; T102/T103ne sont pas encore cochées.


## Lots RED de compilation uniquement — raccords B

| Reçu | Temps / RC | Diagnostics compilateur | Assertions exécutées | Sources stables |
|---|---|---:|---:|---:|
| T102-T104-T107-T111-grouped-RED-compile.json | 38,04 s / RC101 | 107 | 0 | 220 |
| T114-store-RED-compile.json | 30,58 s / RC101 | 10 | 0 | 221 |
| T114-store-RED-runtime.json | 24,50 s / RC101 | 1 (E0596) | 0 | 221 |

Ces trois lots n’ont exécuté aucun test métier. Les diagnostics ne sont pas des
FAIL d’assertions. La dernière ligne « could not compile » récapitule les
107, 10 ou 1 erreurs ; elle n’est pas comptée comme un diagnostic supplémentaire.
Les empreintes avant/après sont identiques pendant chaque run.

Le lot groupé montre des raccords/API encore absents. Le lot Store montre
notamment des types à raccorder, une méthode privée et un import tempfile non
lié. Ces corrections doivent réemployer les dépendances et fixtures existantes,
sans ajouter de crate. Les stubs missed de raccord sont transitoires : ils ne
font pas partie de la livraison finale et leur retrait doit être vérifié
avant la revue T122. Aucun GREEN de ces lots ni assertion métier réussie n’est
crédité ici ; les preuves de surface et de codec précédentes restent distinctes.

Reçus ROOT :
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T102-T104-T107-T111-grouped-RED-compile.json
  Journal : /Users/moi/.cache/bridget-tests158B-20261012/T102-T104-T107-T111-grouped-RED-compile.log
  SHA256 : ab26cb4b3c05f0164e9fb4d6125716826c44252333589ccbf10c751b5b011fee.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T114-store-RED-compile.json
  Journal : /Users/moi/.cache/bridget-tests158B-20261012/T114-store-RED.log
  SHA256 : c8abe08acbf3ba923dc8f285d7c0a2efc79bd1b1f947f2daac23f9bc1d998abe.

## Store G8–G10 — premier RED d’assertions réel

Le nom historique T114-store-RED-runtime est conservé ; malgré ce label,
il s’est arrêté à E0596 en compilation (24,50 s, RC101), sans assertion.
ROOT a corrigé le raccord de mutabilité en une ligne, sans dépendance nouvelle.
Puis T114-store-RED-assertions a exécuté les trois oracles Store :
**0 PASS, 3 FAIL attendus, 0 ignored, RC101**, 57,83 s au total et 0,01 s pour les tests.
221 sources strictement stables. Ce lot constitue une preuve RED métier ;
il ne transforme pas les lots de compilation précédents en assertions.

| Oracle réel | Échec observé |
|---|---|
| store::observation_missed::tests::observation158b_g8_provenance_once_writer_et_transaction | UUID attendu absent : liste vide au lieu de la provenance sauvegardée. |
| store::observation_missed::tests::observation158b_g9_bornes_bytes_ttl_proprietaires_et_lacunes | Nombre 0 au lieu de 64 faits attendus. |
| store::observation_missed::tests::observation158b_g10_retour_cutoff_resume_uuid_et_ack_cas | observation_owner_presence attendu vrai renvoie faux. |

L’implémentation réelle Store est ensuite appliquée par ROOT. Le GREEN groupé
T102–T118 est en cours ; aucun PASS ni cochage déduit ici.

Reçus supplémentaires :
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T114-store-RED-runtime.json
  Journal : /Users/moi/.cache/bridget-tests158B-20261012/T114-store-RED-runtime.log
  SHA256 : 983140241e48a384cae4700c423194cb119b88bb6d319d3ba8d3e2bdbe448d2a.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T114-store-RED-assertions.json
  Journal : /Users/moi/.cache/bridget-tests158B-20261012/T114-store-RED-assertions.log
  SHA256 : ba2eea89d077158d407682d9b7a9640a0e3efae2cff15b6215e58829f1c6e107.


## Lot groupé T102–T118 r1 — résultats partiels

Reçu ROOT :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T102-T118-grouped-GREEN-r1.json
Journal : /Users/moi/.cache/bridget-tests158B-20261012/T102-T118-grouped-GREEN-r1.log
SHA256 : a4c7eacadfc46db30678ea5d730861a6bcc46c8fbef046df38b21561018d5ea6.

20,84 s au total ; tests principaux 0,72 s ; RC101. Processus principal :
**10 PASS, 4 FAIL, 0 ignored sur 14 tests**. Les quatre résultats enfants de
calendrier, 1 PASS chacun, sont exclus de ce compte. Ils rejouent des variantes
internes du même oracle G2 ; ils ne sont pas quatre tests principaux nouveaux.
221 sources avant/après strictement identiques.

| Oracle principal PASS | Couverture constatée |
|---|---|
| cli::guidance156_tests::wake158b_cli_g7_formes_fermees_et_identifiant_retry | Formes CLI et identifiant de retry, pas le quota |
| mcp::tests::wake158b_mcp_events_missed_passe_par_identite_attestee | Surface missed et identité attestée |
| mcp::tests::wake158b_mcp_surface_wake_et_arguments_fermes | Surface wake et arguments fermés |
| store::observation_missed::tests::observation158b_g10_retour_cutoff_resume_uuid_et_ack_cas | G10 Store, retour/cutoff/ACK CAS |
| store::observation_missed::tests::observation158b_g8_provenance_once_writer_et_transaction | G8 Store, provenance et transaction |
| store::observation_missed::tests::observation158b_g9_bornes_bytes_ttl_proprietaires_et_lacunes | G9 Store, bornes/TTL/lacunes |
| store::wake_schedules::tests::wake158B_groupe1_sql_limites_physiques_et_annulees | G1 bornes physiques SQL et annulées |
| store::wake_schedules::tests::wake158B_groupe3_intervalle_gel_restart_et_frontieres | G3 intervalle, gel, restart et frontières |
| wake_schedules::tests::wake158B_groupe1_domaine_ferme_et_bornes | G1 domaine et valeurs fermées |
| wake_schedules::tests::wake158B_groupe2_calendrier_trou_fold_tz_et_date | G2 calendrier et variantes enfants |

Mesure locale du calcul calendrier dans les quatre enfants (build debug,
fixtures synthétiques ; pas latence MCP ni fournisseur ou garantie de service) :

| Variante enfant | Temps observé |
|---|---:|
| utc | 1,105791 ms |
| gap | 1,747084 ms |
| fold | 1,462834 ms |
| non_dst | 1,94925 ms |

Les quatre échecs principaux restent dans le reçu r1 :

| Oracle FAILED | Fait observé |
|---|---|
| daemon::presence_tests::wake158b_daemon_g4_g5_pause_et_grace_avant_depart | Prepared au lieu de MissedBeforeDeparture à la borne 600s. |
| daemon::presence_tests::wake158b_daemon_g6_fil_commit_uuid_et_revalidation | Compte 1 au lieu de 2 attendu. |
| idempotency::wake158b_tests::wake158B_groupe7_cli_quota_rejeu_collision_et_reprise | InvalidIssuerScope avant les assertions de quota. |
| daemon::presence_tests::wake158b_daemon_g8_g10_once_worker_et_retour_ack | Register ne rend pas Registered ; pipeline non confirmé. |

ROOT corrige trois constantes de scope avec underscores, sans élargir le
validateur 099. Le raccord des fixtures Register est en cours. Ces diagnostics
ne remplacent pas des replays verts : quota et pipeline restent FAILED dans r1.
Aucun cochage, GREEN global ou livraison B n’est déduit de ce lot. La FULL finale
T121 n’a pas été lancée par cet owner ; le succès historique des 30 oracles reste
distinct de sa comparaison fraîche prévue dans cette unique FULL.


## Replays ciblés r2, RED calendrier et r3 — 12 octobre 2026

Ces reçus complètent le checkpoint r1 sans changer ses résultats historiques.
La première tentative T121 a été stoppée à la compilation. Aucun résultat global ni cochage n'est
attribué aux lots ci-dessous. Les reçus attestent des sources stables pendant
chaque exécution ; r2 et r3 portent sur 221 sources.

Reçu r2 : /Users/moi/.cache/bridget-tests158B-20261012/T107-T117-affected-GREEN-r2.json
Journal : /Users/moi/.cache/bridget-tests158B-20261012/T107-T117-affected-GREEN-r2.log
SHA256 : cdcedf7c6bebb6aeca5340967bf0dc13cd5b21c1155300f998825e37abbe7e6d.
26,56 s ; RC101 ; processus principal **3 PASS, 1 FAIL, 0 ignored** ; assertions 2,11 s.

| Oracle principal r2 | Résultat |
|---|---|
| daemon::presence_tests::wake158b_daemon_g8_g10_once_worker_et_retour_ack | PASS |
| idempotency::wake158b_tests::wake158B_groupe7_cli_quota_rejeu_collision_et_reprise | PASS |
| daemon::presence_tests::wake158b_daemon_g4_g5_pause_et_grace_avant_depart | PASS |
| daemon::presence_tests::wake158b_daemon_g6_fil_commit_uuid_et_revalidation | FAIL : SQLite ConstraintViolation, code étendu 787, FOREIGN KEY constraint failed dans la fixture. |

Le correctif de fixture 09 annoncé par ROOT ne transforme pas le FAIL r2 en
PASS. Sa preuve de replay est le succès G6 du lot r3 ci-dessous. Quota,
pipeline et pause ont une preuve PASS r2 distincte de leurs échecs r1.

Reçu du filtre incorrect : /Users/moi/.cache/bridget-tests158B-20261012/T104-calendar-date-RED.json
Journal : /Users/moi/.cache/bridget-tests158B-20261012/T104-calendar-date-RED.log
SHA256 : 0286886d433a6ddf070fed9c95f6cd104a8ff83ccdf27d1bd6c5c7e132d9fbbc.
23,21 s ; RC0 ; **zéro test, zéro assertion**. Le filtre wake158b_store_g3
ne sélectionne aucun oracle. Ce run ne prouve ni RED ni GREEN.

Reçu RED exact : /Users/moi/.cache/bridget-tests158B-20261012/T104-calendar-date-RED-exact.json
Journal : /Users/moi/.cache/bridget-tests158B-20261012/T104-calendar-date-RED-exact.log
SHA256 : 8b7d58628e98a22d45a8a4eead9b19b45821079d12cd601f78cbe0584a5ac7c1.
6,34 s ; RC101 ; **0 PASS, 1 FAIL, 0 ignored**. L'oracle
store::wake_schedules::tests::wake158B_groupe3_intervalle_gel_restart_et_frontieres
observe Some("2026-10-14") au lieu de Some("2026-10-16") attendu.

Reçu r3 : /Users/moi/.cache/bridget-tests158B-20261012/T104-T111-T113-G3-G6-CLI-GREEN-r3.json
Journal : /Users/moi/.cache/bridget-tests158B-20261012/T104-T111-T113-G3-G6-CLI-GREEN-r3.log
SHA256 : 31f27fd0b76c2067a3c17f2d13c760b909b39669be0f5c35e4394b2e62e7c7cf.
48,84 s ; RC0 ; **3 PASS, 0 FAIL, 0 ignored** ; assertions 0,21 s.

| Oracle principal r3 | Résultat |
|---|---|
| store::wake_schedules::tests::wake158B_groupe3_intervalle_gel_restart_et_frontieres | PASS : G3 calendrier/date et frontières |
| daemon::presence_tests::wake158b_daemon_g6_fil_commit_uuid_et_revalidation | PASS : G6 fil, commit UUID et revalidation |
| cli::guidance156_tests::wake158b_cli_g7_formes_fermees_et_identifiant_retry | PASS : scénario CLI prévu, pas une recette fournisseur |

Les noms répétés entre r1 et r3 restent les mêmes tests. Les comptes des lots
ne sont pas additionnés comme des tests uniques. Aucun modèle réel, nouvelle
suite complète ou succès global B n'est déduit de ces replays.

## Tentative T121 arrêtée à la compilation

Reçu : /Users/moi/.cache/bridget-tests158B-20261012/T121-FULL158B-final.json.
14,37 s ; RC101 ; zéro test exécuté. E0425 à daemon.rs:1824 : threads::handle était limité à cfg(test). ROOT réemploie handle_with_change existant. Journal SHA256 fdaf7f02b5dba6a246d7dfa66a689525a83c06bf91e2a86dbe05496b17fc017f. Cette tentative ne constitue aucune FULL de régression ni preuve GREEN. La suite prévue reste à exécuter.


## T123 — reprise après commit du fil, preuve ciblée

ROOT a ajouté T123 après le manque identifié par Converge passage 1.
Le périmètre B comporte désormais 23 tâches. Le même oracle G6 est étendu ;
ce reçu ne coche aucune tâche et ne constitue pas un verdict Converge final.

Reçu RED : /Users/moi/.cache/bridget-tests158B-20261012/T123-G6-postcommit-RED.json
Journal : /Users/moi/.cache/bridget-tests158B-20261012/T123-G6-postcommit-RED.log
SHA256 : c2ad0ae07ee3a3626e48ce109e2c08d7d69f7451957c5cd32a95e3bae523dbf8.
36,42 s ; RC101 ; **0 PASS, 1 FAIL, 0 ignored** ; assertions 0,26 s.
L'oracle daemon::presence_tests::wake158b_daemon_g6_fil_commit_uuid_et_revalidation
observe Ok(MissedBeforeDeparture) au lieu de Err("envelope_mismatch") pour
le même UUID102 avec un contenu canonique différent.

Reçu GREEN : /Users/moi/.cache/bridget-tests158B-20261012/T123-G6-postcommit-GREEN.json
Journal : /Users/moi/.cache/bridget-tests158B-20261012/T123-G6-postcommit-GREEN.log
SHA256 : 23ca3585e8f11df1c909fb8c687ba49f0980d44ac21a0110a1dbf33766ad424a.
43,11 s ; RC0 ; **1 PASS, 0 FAIL, 0 ignored** ; assertions 0,59 s.
Les deux reçus attestent des sources stables pendant leur exécution.
Ce PASS rejoue le même G6 ; il ne crée pas un test unique supplémentaire.

Relecture ciblée du 12 octobre :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon.rs,
lecture du raccord 2125–2190 et de l'oracle 24145–24485.
La lecture du reçu102 précède la grâce et les nouvelles gardes de départ.
Une collision canonique rend envelope_mismatch ; une erreur SQL rend
storage_unavailable. Le reçu posted retrouve CommittedThread sans publier
une seconde contribution. Le test garde l'UUID102, les octets canoniques,
l'identifiant d'occurrence et sa date issued_at.

Origine des données : fixture SQLite isolée, horloge avancée par le test,
phase Prepared rétablie pour simuler la frontière de crash après commit102.
L'oracle modifie puis restaure le canon, rend la table des reçus temporairement
illisible, retire le projet et dépasse la grâce de 600 s. Il vérifie le nombre
de contributions avant et après reprise. Ce montage ne prouve pas un crash
réel de processus. Le chemin de reprise CLI099 a seulement été relu par ROOT ;
il n'est pas couvert par cette extension G6. Aucun modèle réel n'est lancé.

Après la tentative T121 stoppée à la compilation, ROOT a lancé la suite
T121-FULL158B-compiled-final. Son résultat reste en attente à ce checkpoint.
Aucun succès global B ou seconde suite complète exécutée n'est annoncé ici.


## T121 — unique FULL exécutée et comparaison fraîche de référence

Ce checkpoint remplace l'attente du résultat dans les paragraphes précédents.
Reçu : /Users/moi/.cache/bridget-tests158B-20261012/T121-FULL158B-compiled-final.json
Synthèse : /Users/moi/.cache/bridget-tests158B-20261012/T121-FULL158B-summary.json
Copies durables identiques dans :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T121-FULL158B-compiled-final.json
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T121-FULL158B-summary.json
Journal : /Users/moi/.cache/bridget-tests158B-20261012/T121-FULL158B-compiled-final.log
SHA256 : fd5ba6c30e89954db7d40725589d0b368d4ded5569cffd1ef66dcc1be2b7a274.

**1356,44 s ; RC101 ; 95 cibles terminées ; 2456 PASS, 12 FAIL, 74 ignored.**
221 empreintes source avant/après strictement identiques. La synthèse garde le
dernier résultat de chaque cible Cargo. Sept résumés de processus enfants sont
exclus ; ils ne sont pas ajoutés comme tests principaux. La première tentative
stoppée à la compilation n'a exécuté aucun test. Cette FULL est l'unique suite
complète réellement exécutée pour T121 ; aucun GREEN global n'est annoncé.

Les 30 noms/cibles exacts sélectionnés à T101 retrouvent tous leur statut PASS
dans cette FULL : **30/30 comparés, aucun absent ni changement de statut**.
Cette preuve fraîche provient de la FULL, sans nouvelle suite de baseline.

Les douze échecs restent dans le reçu. ROOT en distingue huit déjà présents
sur A et quatre nouveaux. Les huit historiques sont :

- cli_uuid_herite_adresse_active_sans_pouvoir_usurper_ni_suivre_une_reponse_ephemere
- client_reference_cli_mcp_rejouent_le_meme_record_et_refusent_les_divergences_sans_mutation
- binaire_et_outil_mcp_partagent_les_quatre_issues_d_une_reponse_liee
- deux_processus_mcp_meme_identite_et_meme_cle_restent_isoles_par_instance
- noms_humains_et_crash_ne_changent_ni_scope_ni_canon_ni_instance_du_wrapper
- spec094_mcp_modifie_uniquement_sa_propre_presence
- cli_demande_mcp_repond_annulation_et_timeout_ne_rouvrent_ni_ne_dupliquent
- spec098_pont_expose_remet_repond_renouvelle_et_retire

Les deux nouveaux échecs lib sont :

- daemon::spec099_classic_delivery_tests::spec100_owner_source_collision_and_nonblocking_notifications
- daemon::spec099_classic_delivery_tests::spec101_upstream_gap_requires_primary_and_dnd_preserves_loss_counters

ROOT a identifié la régression du chemin DND et appliqué un correctif de trois
lignes après la FULL. Son replay ciblé reste en cours à ce checkpoint. Aucun
PASS n'est attribué à ce correctif avant son reçu ; la stabilité de la FULL
ne s'étend pas automatiquement au source corrigé ensuite.

Deux échecs nouveaux concernent la reprise/crash099 :

- recovery_acked_wrapper_finalise_accepted_apres_crash_daemon
- reprise_daemon_redelivre_les_octets_immuables_a_la_meme_instance

ROOT annonce des fonctions et tests byte-identiques à A, avec une borne de
reprise deadline+1 s déjà présente. Le diagnostic de non-déterminisme historique
est en cours de formalisation par l'owner protocole. Cette comparaison n'est
pas un replay réussi et ne démontre pas que ces deux échecs sont corrigés.
Aucune seconde FULL, aucun cochage, statut Implemented ou livraison B n'est
déduit de ce reçu. Les corrections et replays restent des preuves distinctes.


## Clôture des replays ciblés après la FULL — 12 octobre 2026

Ce checkpoint remplace l'attente des replays DND et crash099 dans le reçu
précédent. La FULL reste historiquement 2456 PASS/12 FAIL/74 ignored, RC101.
Les lots ci-dessous sont ciblés. Chaque reçu atteste 221 sources identiques
avant/après pendant son exécution. Aucun résultat n'est réattribué à une autre
empreinte source et aucune seconde FULL n'a été exécutée.

Les reçus sont dans :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation
Les journaux sont dans : /Users/moi/.cache/bridget-tests158B-20261012

| Reçu exact | Temps / code | Résultat principal | SHA256 du journal |
|---|---|---|---|
| T111-T113-G6-G7-attested-T116-diagnostic-GREEN.json | 44,08 s / RC101 | 4 PASS, 1 FAIL, 0 ignored | 61a1ca494c6dc9e2f8eec6237edd21e898ed734252ebd331d73961deb9b57b0e |
| T109-G5-expiration-writer-RED-G6-ascii.json | 37,52 s / RC101 | 1 PASS, 1 FAIL, 0 ignored | 3d773e7e90333c9e2ca2e475d3e5b94aab0d8c62032833a70785ccdfe200b41b |
| T109-departure-clock-GREEN-affected.json | 28,82 s / RC0 | 4 PASS, 0 FAIL, 0 ignored | f7b697e7f436f79678b86f181fd0e0ba1a60c2dfa31ed85f5d24cf05cbe429e3 |
| T121-099-recovery-timing-replay.json | 15,85 s / RC101 | 1 PASS, 1 FAIL, 0 ignored | f23d4365907c20d18f71fa56ed906cfa84d304f91cc5009412295a03c89a6ef3 |

Dans le premier lot, les deux régressions du chemin DND passent :

- daemon::spec099_classic_delivery_tests::spec100_owner_source_collision_and_nonblocking_notifications
- daemon::spec099_classic_delivery_tests::spec101_upstream_gap_requires_primary_and_dnd_preserves_loss_counters

G8/G10 pipeline et le scénario CLI G7 attesté passent aussi. G6 échoue sur
une clé de fixture avec accent, refusée par InvalidIdempotencyKey : status error
au lieu de prepared. ROOT corrige une seule ligne de clé en ASCII. Le libellé
UTF-8 et les assertions de conservation du texte restent inchangés. G6 passe
ensuite dans le lot RED G5 ; ce succès ne rend pas ce lot entièrement GREEN.

Le RED G5 utilise un writer réel tenu jusqu'à une frontière de seconde Unix.
Le log observe 1791808804.900563 → 1791808805.105013 et Ok(DispatchingAgent),
alors qu'aucune remise099 ne devait être créée après la grâce. L'assertion
observe Some("dispatching") au lieu de None. Le correctif T109 revérifie la
date au véritable départ. Le GREEN suivant observe Ok(Deferred) à la frontière
1791808859.900524 → 1791808860.102477 ; les assertions no099/noFrame passent.
Les quatre oracles réussis dans ce dernier lot sont :

- daemon::presence_tests::wake158b_daemon_g4_g5_pause_et_grace_avant_depart
- daemon::presence_tests::wake158b_daemon_g6_fil_commit_uuid_et_revalidation
- daemon::presence_tests::wake158b_daemon_g8_g10_once_worker_et_retour_ack
- cli::guidance156_tests::wake158b_cli_g7_formes_fermees_et_identifiant_retry

Le replay crash099 reste **partiellement en échec** :
recovery_acked_wrapper_finalise_accepted_apres_crash_daemon passe ;
reprise_daemon_redelivre_les_octets_immuables_a_la_meme_instance échoue.
Le second log montre uniquement deadline_at 1791811178 → 1791811179 entre
les deux trames comparées. Le diagnostic existant démontre la projection
historique par seconde au moment de la reprise ; fonctions, tests et fixture
comparés byte-identiques à A. Il n'attribue pas cette cause à SQL158B et ne
supprime pas l'échec réel. Rapport :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/099-recovery-deadline-diagnostic.md
Preuves de comparaison :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/099-recovery-deadline-diagnostic.json
Aucun correctif hors B ni suppression de l'assertion d'égalité n'est crédité.

Le périmètre de clôture reste les dix groupes prescrits et les trente références
historiques comparées fraîches (30/30 PASS dans la FULL). Les preuves de groupes
restent réparties entre les reçus précédents, la FULL et ces replays. Les huit
échecs historiques A restent inchangés. Les noms répétés ne sont pas ajoutés
comme de nouveaux tests uniques. US8 et HTTP restent hors scope. Aucun succès
global, cochage, statut Implemented ou livraison B n'est annoncé par cet append.


## Revue R2 — portée exacte de la preuve G5

Reçu : /Users/moi/.cache/bridget-tests158B-20261012/T109-departure-clock-GREEN-affected.json
28,82 s ; RC0 ; quatre PASS ciblés. Le journal observe Ok(Deferred) pour G5,
avec absence de remise099 et de trame. Il ne prouve pas la finalisation au
passage ultérieur qui relit cette occurrence. Callback refusé sans begin_send_delivery : aucun départ enregistré,
retour Deferred possible. Le parcours relu lors d’un passage ultérieur qui relit cette occurrence, pour now >=
grace_until et des lectures durables réussies, rend MissedBeforeDeparture ;
le parcours CLI appelle finalize Rejected. Aucun test de cette finalisation
n'est revendiqué et aucun nouveau scénario n'est demandé.

SHA daemon avant et après dans le reçu, également égal au fichier courant au checkpoint R2 initial :
42a95a02f22c7b223eb6d433bab741f7e4722d771dd05f204f9331685a0316ec.
Le candidat cache GREEN-daemon.rs porte 2a1490af0284174cbec5f0663caf4820d8c57b71caf5751b9b353e73d3a9ac5a ;
il est une préparation avant formatage final, pas l'empreinte exécutée attestée
par ce reçu. La preuve reste ciblée ; aucun succès global ou livraison déduit.


## Correction des deux retours et preuve finale ciblée

Le checkpoint R2 précédent sur daemon42a95a02 et son GREEN28,82 s reste
historique. La réfutation du retard sous horloge continue ne couvrait alors
que la boucle principale. ROOT a trouvé deux autres appels observer_return_tick,
Registered et Availability, qui évaluaient Unix avant Mono. Ils sont corrigés
par return_clock local capturé avant unix_now_secs, sans helper ni scénario neuf.
Le raisonnement de borne conservatrice s'applique désormais aux trois chemins
relus. Time/SQL non atomique et saut d'horloge civile restent des limites.

Reçu : /Users/moi/.cache/bridget-tests158B-20261012/T109-T117-return-clock-order-GREEN.json
Journal : /Users/moi/.cache/bridget-tests158B-20261012/T109-T117-return-clock-order-GREEN.log
SHA256 : a7fda2d6fa2a47e7556e4ef18fdef988b7ddb36552dbd727e48afe6ce982b5fd.
22,51 s ; RC0 ; **4 PASS, 0 FAIL, 0 ignored** ; assertions 1,84 s.
221 sources avant/après strictement identiques. SHA daemon exécuté et courant :
2124b1d0b42c3232698269918adf9f9df012a7aa8dae07cac98919f0c82064a5.
Les mêmes oracles G4/G5, G6, G8/G10 et CLI G7 passent. Ce lot ne prouve pas
la finalisation d'une occurrence par un passage ultérieur qui la relit.
La page CLI de 128 entrées ne garantit pas la relecture au prochain tick.
Aucun second FULL, succès global, nouveau scénario ou livraison annoncé.
