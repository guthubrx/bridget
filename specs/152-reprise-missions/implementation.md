# Journal d’implémentation — session152

Date : 2026-10-11.
Branche : session-152-reprise-missions.
Worktree : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions.

## Référence T001

T001 est validée : six filtres existants, 239 PASS/0 FAIL/1 ignoré, chaque run RC0.
Détail : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/baseline.md.

## T002/T003 — premier RED observé, sans validation de tâche

Exécution par le principal sur les fixtures/tests nouveaux. Le principal confirme
la production native_delegation.rs/daemon.rs inchangée pendant ce run.
Aucun test ni service n’a été lancé par le propriétaire documentaire.

Commande exacte, dans le worktree 152 :

```sh
python3 scripts/build.py cargo test --locked --offline -p bridget-daemon --features test-support native152_rouge --lib -- --test-threads=4 --nocapture
```

Résultat : compilation terminée, puis **0 PASS/3 FAIL/0 ignoré, RC101**.
Durée totale : 33,45 s ; compilation 32,21 s ; tests 0,92 s.
1362 tests filtrés : aucun résultat déduit pour eux.

| Test rouge | Donnée réelle observée | Assertion en échec |
|---|---|---|
| Chemin A, vrai DeliveryRejected | tâche failed, erreur tour annulé par le pilote ; suivi=false ; vrai avis d’échec reçu par le parent | failed au lieu de working |
| Chemin B, interrupted puis tick | tâche failed, erreur interrupted | failed au lieu de working |
| Ordre inverse, TurnState(false) avant interrupted | avant interrupted : absence de reprise et exécution interrupting vérifiées ; après interrupted : tâche failed, aucune trame de reprise | failed au lieu de working |

Le chemin A logue le retrait du suivi et l’avis réel. Les assertions finales du
suivi conservé et du silence parental ne sont pas atteintes, car l’assertion de
l’état échoue d’abord. Dans l’ordre inverse, l’assertion de l’identifiant de la
mission reprise n’est pas atteinte. Ces observations sont des RED réels ; elles
ne remplacent pas les futures assertions GREEN. T002/T003 restent non cochées.

Journaux immuables de ce premier run :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-pause152.log — SHA-256 92b599d55a5824f497f1c69edae25644e5b7e0a04425a76f6a90c32003276a0b.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-pause152.json — SHA-256 2f9c525be6ef34056da15b30cd0b44d3f83c0c253c932952c207154ff449aa75.

## Écart de recette à conserver

Racine temporaire réellement utilisée : /var/folders/82/29vmttnx4pz9mc73kzzfvw1r0000gn/T/b.aczqbxyd.
Son mode 0700 est observé en lecture des métadonnées. L’environnement privé et
les assertions réelles sont confirmés par le principal. Cette racine se trouve
sous /var/folders, alors que la recette 152 demande une racine courte /tmp/b.XXXX.
Ce premier run reste conservé tel quel. Le principal utilisera /tmp pour les
prochains runs ; aucun succès de conformité intégrale à la recette /tmp n’est
attribué à ce run.

## Suite autorisée

T033 et T035 sont seulement préparées en lecture seule. Leur publication attend
les finitions prévues et les preuves du code livré. Les gates de conception
archivés précèdent ce RED ; leurs empreintes d’entrées décrivent ce moment.

## Nouvelle passe — fondations et RED étendu, 2026-10-11

Le principal a exécuté les cinq filtres ci-dessous. Le propriétaire documentaire
a lu le JSON et les journaux bruts ; il n’a lancé aucun test.

| Filtre | Résultat | RC | Durée totale | Tâche concernée |
|---|---|---|---|---|
| native152_chaine | 2 PASS/0 FAIL/0 ignoré | 0 | 3,11 s | T010 |
| native152_task_payload151 | 1 PASS/0 FAIL/0 ignoré | 0 | 2,99 s | T008 |
| native152_horloges_pause_overlap_durable | 1 PASS/0 FAIL/0 ignoré | 0 | 2,99 s | T050 |
| garde_unique_table_de_verite, table existante | 1 PASS/0 FAIL/0 ignoré | 0 | 3,12 s | T009 |
| native152_rouge | 0 PASS/7 FAIL/0 ignoré | 101 | 13,82 s | RED étendu, avec réserve T005 |

Les quatre filtres de fondation comptent **5 PASS/0 FAIL**, chaque run RC0.
Le principal autorise la clôture de T008, T009, T010 et T050 avec ces preuves.
Ces quatre cases sont cochées ; toutes les autres cases sont conservées.
La progression est **5/50 (10 %)**, avec T001 déjà validée. Aucun succès
n’est déduit pour les tests filtrés. Ces cinq PASS restent distincts des
239 PASS/0 FAIL/1 ignoré de la baseline T001 ; aucun total global n’est annoncé.

La passe rouge a compilé en 13,13 s, puis exécuté sept tests en 0,51 s.
Son résultat brut reste **0 PASS/7 FAIL, RC101**. Six échecs sont des RED
valides : les trois chemins de T003 déjà observés ; T039, qui conserve
mission_pending au lieu de working lors du rejeu ; les deux cas T049,
qui retrouvent respectivement une expiration du suivi pendant la pause et
un échec mission_reply_timeout après une pause sans tick. La répétition des
trois T003 n’ajoute pas trois défauts nouveaux.

**T005 n’est pas un RED définitif.** Son premier arrêt porte sur un témoin de
demande ordinaire : l’oracle attendait expired, alors que le contrat réel
rend timed_out. Le journal montre timed_out pour ce témoin et pour la lecture
native, sans dépassement du budget d’attente. La façade corrige seulement
l’oracle du témoin ; le principal rejouera ensuite sur le comportement ancien.
Cette passe ne clôt ni T005 ni les autres tâches de reproduction.
T002/T003 restent non cochées jusqu’au GREEN.

Racine temporaire réellement utilisée : /tmp/b.2ykydeej.
Son mode 0700 est observé dans les métadonnées. Ce chemin court suit la recette
prévue pour ce run. Le premier run sous /var/folders reste conservé plus haut.

Commandes consignées dans le JSON, depuis
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions :
`python3 scripts/build.py cargo test --locked --offline -p bridget-daemon --features test-support <filtre> --lib -- --test-threads=4 --nocapture`.
Les cinq valeurs de filtre sont celles du tableau ; aucune commande n’a été
réexécutée pour établir ce reçu.

Source des résultats :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-fondations152-results.json — SHA-256 3c2c40465b7a3ead2b7eb17616bbaceda8984357344b7984907235e7ce5b4921.

Journaux bruts lus, sans modification :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-fondations152-native152_rouge.log — SHA-256 cb52bfe63ac4d81c1b901008963d2c476513c2a1bd897934c143319c04c54a86.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-fondations152-native152_chaine.log — SHA-256 cc50ebff93f54238ed92606833ffee5db5e27701878bd229e67ed90afb683a9d.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-fondations152-native152_task_payload151.log — SHA-256 89e6b18fe87a82971c4419b101a75f44fc130c3d92dc0402a13bd48b8c3f3d96.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-fondations152-native152_horloges_pause_overlap_durable.log — SHA-256 8705c2d8135d0b5e8db0dacdb9012608ae5b1bfc10f80b868aeb9d30b426b2e9.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-fondations152-garde_unique_table_de_verite.log — SHA-256 9de085abd12aa20724b7b1a1178a82877fb4a51a2baabc4f751478e354d1058c.

## Limite documentaire de livraison, à trancher par le principal

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/152-reprise-missions/implementation.md#extrait-1).

Textes concernés :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/spec.md — H6.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/quickstart.md — contrôle manuel.

À cette étape, rien n’est livré ni activé. Les preuves de test, l’intégration
Git, les fichiers installés et le code chargé par les processus restent
des états distincts. T033/T035 restent en préparation, sans publication.

## R1 de la reproduction complète — échec de compilation, sans test

Cette tentative est distincte des premiers RED de pause et de la passe de
fondations. Le principal a exécuté le filtre native152_rouge avec la cible
lib test ; la commande a échoué en compilation, **RC101, 3,92 s**.
Le JSON ne contient aucun résultat de test et le journal n’annonce aucune
exécution. Cette tentative ne constitue donc pas un RED comportemental.

Les deux diagnostics concernent la fixture à la ligne historique 2497 de
native_delegation.rs : E0061, l’appel register_connected omet le troisième
argument now ; E0600, assert! est appliqué au retour SpawnCommandIssue,
qui n’est pas un booléen. La correction de la fixture précède la passe R2.
Aucun PASS ou FAIL de test n’est attribué à R1.

Racine privée du run : /tmp/b.e55jnxrs, mode 0700 observé en lecture.
Preuves R1 conservées sans remplacement :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-complet152-results.json — SHA-256 20ce627b794079921d594a4dd9073951f8a2340d3cfaae5e1d59b9fe017ca680.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-complet152-rouge-lib.log — SHA-256 6fb5d9017e4ce0b908c51886e2bc56be43ba46d8d3216e6e9e79c2eea5d488c8.

## R2 de la reproduction complète — RED exécutés et témoins ordinaires

Le principal a exécuté les quatre commandes consignées dans le JSON R2.
Le propriétaire documentaire a lu le JSON, les résultats et les diagnostics
des quatre journaux. Il n’a lancé aucun test.

| Cible et filtre | Résultat | RC | Durée totale |
|---|---|---|---|
| lib, native152_rouge | 0 PASS/14 FAIL/0 ignoré | 101 | 11,19 s |
| native_delegation_e2e, native152_rouge | 0 PASS/5 FAIL/0 ignoré | 101 | 14,00 s |
| lib, TEMOIN_GARDE_a_61_secondes_l_humain_n_expire_pas_et_l_agent_expire | 1 PASS/0 FAIL/0 ignoré | 0 | 3,15 s |
| lib, spec_087_pause_differe_les_relances_mais_pas_le_palier_trois | 1 PASS/0 FAIL/0 ignoré | 0 | 3,09 s |

Les deux compilations rouges sont terminées : 10,05 s pour la bibliothèque,
8,62 s pour l’intégration. Les tests exécutés durent respectivement 1,00 s
et 5,27 s. Les **14 RED natifs et 5 RED E2E** sont des échecs comportementaux
compilés, contrairement à R1. Les résultats des tests filtrés restent inconnus.

Répartition des 14 RED natifs : T003 (3), T004 lancement et contrôles (1),
T005 (1), T006 (1), T007 (1), T038 (1), T039 (1), T040 (1),
T047 (1), T048 instance neuve/flotte ancienne (1), T049 (2).
Les cinq E2E couvrent T004 mission working et quatre frontières de T048 :
lien absent, flotte neuve/tâche ancienne, marqueur lisible différent et
marqueur déjà réconcilié. Trois frontières refusent encore le démarrage du
daemon isolé ; le marqueur déjà réconcilié et la mission working deviennent
failed/unreachable. Ce sont des tests du daemon isolé avec pilote de fixture,
sans modèle réel ni activation de production.

**T005 dispose maintenant d’un RED valide.** Le témoin ordinaire retourne
timed_out, accepté par l’oracle corrigé. Le test atteint ensuite l’assertion
sur le gel métier natif et le budget d’attente d’une seconde. La donnée
réelle native est timed_out, timed_out_wait=false, waited_secs=0,
en 36,292 µs ; la tâche échoue avec mission_reply_timeout.
L’erreur d’oracle de la passe antérieure reste conservée dans son reçu ;
elle n’est pas attribuée à cette passe R2. Aucune assertion située après
un arrêt du test n’est présentée comme exécutée.

Les deux témoins ordinaires sont **2 PASS/0 FAIL, tous RC0**.
Avec ces preuves, le principal autorise la clôture de T011. Seule sa case
est nouvellement cochée. La progression est **6/50 (12 %)**.
Les tâches de reproduction RED restent décochées jusqu’au GREEN.
Ces deux PASS restent distincts des fondations et de la baseline ; aucun
total global de tests ni validation de toutes les exigences n’est déduit.

Le JSON R2 indique source_hashes_stable=true pour les six fichiers ci-dessous.
Ces empreintes décrivent la passe R2 et sa stabilité interne ; elles ne
prétendent pas figer les sources pendant l’implémentation suivante :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs — SHA-256 da99083ef7ba6b31282a8d8f84b7dd8f77c7c698ca55c211f470b3794136b855.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs — SHA-256 2498b999975fd92a01ff68287078546835f9cc691f376524e3b1d9b64b50f5f2.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/tests/native_delegation_e2e.rs — SHA-256 7368afc3da2aeeb48bb1c7bf084e2fcdf56031051308f89aa65bcaff2527cfe5.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/delegation.rs — SHA-256 281488d9b8b92747b20920282635d9f6c2da2a41c9d41ce024b8018fe98e99ce.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/execution_store.rs — SHA-256 cf60345886171461040b92c2183f52d1515a3e3a353c4f68dc40d4e2b5193f0a.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/referent_control.rs — SHA-256 b2125132c12312dcf6173c5200a85eb3d079dd6cdc3788550e30fa918f68d8f4.

Racine privée du run : /tmp/b.n9q0iy5p, mode 0700 observé en lecture.
Les commandes exactes et leurs paramètres sont conservés dans le JSON ;
aucun changement de budget ou de filtre n’est déduit du seul nom des tests.

Preuves R2 conservées sans modification :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-complet152-r2-results.json — SHA-256 b46380f079ee7de0467e9e10f1b68ffd501d75f48f741020be2615174afe3878.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-complet152-r2-rouge-lib.log — SHA-256 8c077cbedaca32d99e07c5a5883ffb34aded019b46c28de13de538738b6f2313.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-complet152-r2-rouge-e2e.log — SHA-256 cfeb37636dc7dbededd68d4f34ed72c3c47b865912489503079f7f0258b8c11e.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-complet152-r2-temoin-ordinaire.log — SHA-256 07c1acad35c315a5519a155abf690c9e9ab41f171de57af7c4a6788bee8123f1.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-complet152-r2-temoin-palier-ordinaire.log — SHA-256 f1395be550fa1117293c690e1acf1d6ae8d63b3bb7a8fa2ef93aab68b5247588.

## T032 — adaptation explicite du contrat de reprise 149

SPEC152 remplace l’échec systématique au redémarrage par une reprise bornée
des missions éligibles (FR-013/FR-014/FR-015), avec filiation préservée
(FR-027). Quelques tests149 imposaient le contrat antérieur. Leur oracle
doit être adapté à cette décision, avec une correspondance du nom de test
aux FR concernés et les attentes avant/après.

Ces écarts ne sont pas des échecs hérités à ignorer. Les contrôles de refus
pour droits révoqués, sources modifiées et identité non attestée restent
obligatoires. Les conditions d’échec de FR-016, les états conservés de FR-017
et l’échec de la seule tâche incohérente prévu par FR-029 restent vérifiés.
Adapter la portée de cet échec conformément à FR-029 ne permet jamais une
reprise non autorisée. Aucun test n’est supprimé ni rendu moins exigeant
pour obtenir un résultat vert. T032 reste décochée avant son exécution.

Le principal a donné GO au propriétaire façade pour US1 → US2 → US3 →
US4 → US5. T033/T035 attendent le GREEN et les finitions. Rien n’est livré
ni activé à ce stade. La limite documentaire H6/quickstart reste ouverte
pour la phase de livraison coordonnée par le principal.

## US1/US2 R1 — résultats partiels et tâches prouvées

Lot 71384 exécuté par le principal. Le propriétaire documentaire a lu le JSON,
les huit journaux et les oracles des tests concernés, sans lancer de test.
Les sept filtres US1/US2 comptent **13 PASS/3 FAIL**. Le filtre des refus
ajoute **2 RED T029 attendus**, séparés des trois échecs de reprise.
L’ensemble des huit runs compte 13 PASS/5 FAIL, aucun ignoré ; aucun résultat
n’est déduit pour les tests filtrés ni pour une suite globale.

| Filtre | PASS | FAIL | RC | Durée totale |
|---|---:|---:|---:|---:|
| native152_pause | 3 | 2 | 101 | 3,67 s |
| native152_rouge_pause | 4 | 1 | 101 | 3,74 s |
| native152_rouge_horloges | 1 | 0 | 0 | 4,42 s |
| native152_rouge_lancement | 1 | 0 | 0 | 3,34 s |
| native152_rouge_gel_mixte | 1 | 0 | 0 | 3,33 s |
| native152_horloges | 2 | 0 | 0 | 3,26 s |
| native152_vue | 1 | 0 | 0 | 3,27 s |
| native152_refus, RED T029 | 0 | 2 | 101 | 10,26 s |

### Clôtures et preuves exactes

| Cases nouvellement cochées | Preuve exécutée ; complément de lecture si indiqué |
|---|---|
| T005, T018 | native152_rouge_horloges_mission_suivi_et_request_wait : attente native open pendant le budget, suivi sans palier ni expiration, témoin ordinaire timed_out. Les cas T039/T049 prouvent reconnexion et origine durable du gel. |
| T006, T020 | native152_rouge_lancement_apres_pause : queued retenu 600 s puis starting, date récente et bail de 300 s ; octets de la saga déjà soumise identiques et aucun deuxième lancement. |
| T039, T045 | native152_rouge_gel_mixte_reconnexion_et_rejeu_enveloppe : date de demande issue de l’admission, suivi open sans timeout et rejeu admis en working. La sauvegarde de la première date avant send_as et la compatibilité de l’enveloppe déjà admise sont aussi vérifiées en lecture ciblée, sans exécution distincte. |
| T049 | Les deux native152_rouge_pause_sans_tick passent : journal relu par une autre connexion et date brute conservée ; admission durable avec checkpoint mission_pending et suivi open après reconnexion. |
| T012 | T005 conserve working malgré l’échéance brute passée ; native152_pause_aucun_lancement_ni_remise conserve queued/mission_pending pendant 300 ticks. Lecture ciblée : garde commune avant les mutations pour queued/starting/mission_pending/working. |
| T013, T044 | native152_rouge_pause_chemin_a_suivi_conserve_sans_avis et native152_pause_refus_hors_pause_notifie_toujours : vrai DeliveryRejected, silence et suivi conservé sous pause ; échec, suivi retiré et avis reçu hors pause. |
| T016 | native152_pause_annulee_pendant_la_pause_n_est_pas_ressuscitee : arrêt observé, cancelled, interruption consommée et aucune continuation après la levée. |
| T021 | native152_vue_suspended_additive_et_lineage_identique : suspended/resume_attempts additifs, status conservé, octets Lineage identiques ; task_view pur sans suspended. |

Ces douze tâches sont cochées. Les six anciennes cases sont conservées :
**18/50 (36 %)**. Les autres lignes, les dépendances et T032 restent identiques.
Lectures de production complémentaires, qui ne constituent pas des tests :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1327 — contrôle lu une fois et garde de pause avant les mutations du tick.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1522 — première date sauvegardée avant send_as ; enveloppe déjà admise conservée.

### Données et limites

T005 montre open, timed_out_wait=true, waited_secs=1 en 1,004623959 s.
Le témoin ordinaire reste timed_out, timed_out_wait=false. Le suivi natif reste
open au palier 0, sans timeout ; la tâche reste working et sa date brute ne bouge
pas. T003/A montre working, suivi=true et aucun avis parental : son WouldBlock
est le silence attendu dans ce test PASS. T003/B passe aussi.

Trois tests US1/US2 restent en échec :

- native152_rouge_pause_reprise_attend_interrupted : working, mais trame enfant
  vide après WouldBlock ; le décodage JSON échoue.
- native152_pause_chaine_suivie_apres_reprise : lecture enfant WouldBlock ;
  les assertions de remise et de nouvelle feuille ne sont pas atteintes.
- native152_pause_resultat_rendu_une_fois : lecture de reprise WouldBlock ;
  les assertions finales de résultat unique et de cleanup ne sont pas atteintes.

T002/T003/T014/T015 restent décochées pour ces séquences de reprise.
T019 reste décochée pour la remise du résultat, malgré le compteur nul
pendant 300 ticks. T017 reste décochée pour la borne d’échéance d’un descendant :
les horloges et pauses cumulées passent, mais le lot n’exerce pas son admission.
Ces limites relèvent des cas déjà prévus ; aucun nouveau test ou ID n’est créé.

T029 reste RED : le premier model_unavailable devient cwd_unavailable au rejeu
après changement de répertoire ; le registre expose un texte dynamique avec
chemin et liste de types au lieu de agent_type_unknown.

### Provenance US1/US2

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r1-results.json — SHA-256 87b9c5055ab8fb043ed0283477d416451d47e7184e18bc4e63499acdc23b11e3.

Racine privée /tmp/b.jwnycicf, mode 0700 observé. Le JSON atteste la stabilité
de cinq empreintes source pendant le run ; il ne fige pas le code suivant.
La lecture actuelle des sources ne vaut pas une nouvelle exécution.
Les huit commandes exactes sont conservées dans ce JSON.

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r1-native152_refus.log — SHA-256 fbf712c2b9b0b409cb9ea95cab7fea9789a6c56bccad9cced9da24aa618e5015.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r1-native152_pause.log — SHA-256 ce9ed975682294fa3ba12ede3bd24cc7dd6a737df2b3c8f5a2561e07a1526e12.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r1-native152_rouge_pause.log — SHA-256 a7886e6303fea1f22d32582c4262fd1097cc6639c1a87956c1dae92c273a1eb7.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r1-native152_rouge_horloges.log — SHA-256 6dcb6d042433a7d1bc366de3eab6fd9c2e9d59e7362dac23c9536dc0da727847.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r1-native152_rouge_lancement.log — SHA-256 c83e380ad139c062511b4cc4e7fcf8896d501e208cf65fce757b190e8e27ba6f.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r1-native152_rouge_gel_mixte.log — SHA-256 c4d42ec80458555789c38015e712def4a1c338e723e3e43cbf826e6eebd5c624.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r1-native152_horloges.log — SHA-256 bef51d5e3ea98ade6f2ca86b2448a07b4704a795a5887bf147ea9c77bac6706c.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r1-native152_vue.log — SHA-256 9128a311a58aafa8a79403143f4bcbf61a7f71a06516964343e0c8e77ab929e3.

## Diagnostic de réception R1 — ACK150 absent dans la fixture

Le principal a reproduit le test de chaîne avec une donnée réelle :
tâche working, phase150 dispatching, livraison durable présente et exécution
starting, génération 1. La lecture de reprise échoue par WouldBlock.
Résultat : **0 PASS/1 FAIL, RC101**, durée totale 8,14 s, tests 0,14 s.

Ce diagnostic est distinct des trois FAIL US1/US2. Le principal confirme
l’absence d’ACK dans la fixture. La façade corrige uniquement celle-ci par
le vrai événement WrapperToDaemon::DeliverAcked, avant de passer à US3.
Aucun état de livraison n’est forcé pour contourner le protocole. Ce reçu ne
déclare aucun GREEN après la correction ; les tâches de reprise restent ouvertes.

Racine privée /tmp/b.xb_vgs9j, mode 0700 observé. Cinq empreintes source stables
pendant ce run selon son JSON. Preuves brutes conservées :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/diagnostic-reception-152-r1-results.json — SHA-256 2dadb4f985b5dcc0dcadf3604b37f637141ee15f52c8045586a678ff4b80728b.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/diagnostic-reception-152-r1-native152_pause_chaine_suivie_apres_reprise.log — SHA-256 fe90c4884177be0efdc5ac8edb610f4d093c6c1142e7ddefd7534e2f4842a0b3.

## Magasin de refus R1 — 12 PASS, intégration API encore ouverte

| Filtre du magasin | PASS | FAIL | RC | Durée totale |
|---|---:|---:|---:|---:|
| native152_refus_rejeu_identique | 1 | 0 | 0 | 3,46 s |
| native152_refus_autre_enveloppe_mismatch | 1 | 0 | 0 | 3,16 s |
| native152_refus_transitoire_jamais_garde | 10 | 0 | 0 | 3,22 s |

Le magasin compte **12 PASS/0 FAIL**, trois commandes Cargo RC0.
Le rejeu identique et envelope_mismatch passent. Les dix cas transitoires
passent séparément : delegation_grant_required, parent_execution_ambiguous,
parent_task_unavailable, identity_unavailable, permission_attestation_unavailable,
permission_not_inherited, referent_paused, reply_timeout_exceeds_parent,
retention_saturated et task_limit.

**T030 reste décochée.** Ces tests prouvent le magasin, mais pas encore son
branchement à l’API. Le principal confirme que record_refusal et la normalisation
T029 dans le cœur natif, ainsi que la purge T031 dans le daemon, ne sont pas
encore branchés à ce stade. T029/T030/T031 restent ouvertes jusqu’à leurs preuves
d’intégration et leur GREEN. L’avertissement purge_expired_refusals inutilisée
est conservé dans les journaux. Les douze PASS ne remplacent pas les deux
RED API T029 et ne sont pas additionnés aux anciens lots comme total global.

Recette privée conservée : /tmp/b.fa2erh20, mode 0700 observé ; cinq empreintes
source stables pendant les trois runs selon le JSON. Les racines temporaires
des lots restent distinctes. Preuves brutes conservées :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/store-refus-152-r1-results.json — SHA-256 74c05986678d27196c483dce1f781f64d560648911f0f4a1c6117de7c996b1ff.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/store-refus-152-r1-native152_refus_rejeu_identique.log — SHA-256 4b35a8a7b494f48fe51b4a9500a41eb013599b585641f324d1529b0cc482cc49.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/store-refus-152-r1-native152_refus_autre_enveloppe_mismatch.log — SHA-256 992d00d7939506264c703ef5698c83091e8ccda2feca7b15d473225dd67991df.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/store-refus-152-r1-native152_refus_transitoire_jamais_garde.log — SHA-256 026ef4651b147985c2aae5c9126956ee401de6fc558276ee85d96b8af1bcf59d.

## Alignement du quickstart sur H6 — décision du principal

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/152-reprise-missions/implementation.md#extrait-2).

L’ordre du contrôle est précisé : suspended=true se lit pendant la pause ;
après la levée, suspended=false précède la vérification du résultat.
Cette précision conserve la vue et le contrat, sans nouvel outil ou scénario.
La contradiction H6/quickstart décrite dans les reçus antérieurs est donc
résolue documentairement ; ces captures historiques ne sont pas réécrites.

Fichier aligné :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/quickstart.md.

Aucune relance n’a lieu maintenant. Les futurs reçus doivent distinguer
sources Git, fichiers installés et code réellement actif. Aucun état actif
n’est affirmé avant observation. T032 conserve ses oracles de droits et d’identité.
T033/T035 restent préparées en mémoire, sans publication avant les GREEN
et les finitions. Rien n’est livré ni activé par le propriétaire documentaire.

## US1/US2 R2 — trois reprises GREEN et borne du descendant

Les sept filtres R2 comptent **17 PASS/1 FAIL**, aucun ignoré.
Le JSON atteste cinq empreintes source stables pendant le lot.
La correction préalable porte sur la fixture : vrai DeliverAcked initial,
avec phase150 acked et suivi=true observés dans les journaux. La phase
dispatching du diagnostic précédent reste conservée comme donnée avant fix.

| Filtre | PASS | FAIL | RC | Durée totale |
|---|---:|---:|---:|---:|
| native152_pause | 5 | 1 | 101 | 6,07 s |
| native152_rouge_pause | 5 | 0 | 0 | 3,53 s |
| native152_rouge_horloges | 1 | 0 | 0 | 4,35 s |
| native152_rouge_lancement | 1 | 0 | 0 | 3,60 s |
| native152_rouge_gel_mixte | 1 | 0 | 0 | 3,32 s |
| native152_horloges | 3 | 0 | 0 | 3,32 s |
| native152_vue | 1 | 0 | 0 | 3,21 s |

Les trois anciennes reprises échouées sont GREEN :
native152_rouge_pause_reprise_attend_interrupted,
native152_pause_chaine_suivie_apres_reprise et
native152_pause_resultat_rendu_une_fois. L’ordre inverse lit maintenant une
vraie trame DeliverIdempotent, génération 2 ; le test vérifie l’identifiant
d’origine. La chaîne est suivie jusqu’à sa nouvelle feuille ; la remise
unique du résultat et le cleanup atteignent leurs assertions.

Le test native152_horloges_delegation_imbriquee_bornee_par_echeance_effective
passe. Données réelles : échéance brute 1791729965, effective 1791730085 ;
un enfant de 60 s est admis et un enfant de 120 s est refusé.
Cette preuve clôt la réserve T017 de R1.

T002/T003/T014/T015/T017 sont donc nouvellement cochées.
Le seul FAIL R2 est le cas T019 ajouté dans la tâche déjà prévue,
native152_pause_resultat_acquis_reste_remis : après capture_reply, son oracle
attend immédiatement result_available alors que le contrat rend
waiting_for_children. Les assertions finales sous pause ne sont pas encore
atteintes dans R2. T019 n’est pas clôturée par ce run.

Racine privée /tmp/b.dd4p1zm1, mode 0700 observé.
Preuves R2 conservées sans remplacement :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r2-results.json — SHA-256 fd4e32c1a795d844a01abb9cf8bd384a3340f983a534f3d889b14f4e61076d1a.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r2-native152_pause.log — SHA-256 d4464038f50baac6665505f6f538ae94d7a9989cebea80a7a039c2e54a91cd4a.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r2-native152_rouge_pause.log — SHA-256 205677120c5cbc62d917fe17aad47ba36160d0575d880e8da550355d5961f160.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r2-native152_rouge_horloges.log — SHA-256 ecb35642a8c5874844b1e61b65a0dc82f63eef438c3413573c0a6af5623cc55b.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r2-native152_rouge_lancement.log — SHA-256 a988dc00a498d37b73c86fbb3a8924aaa3807c327b5972d36faa31cb27c76c54.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r2-native152_rouge_gel_mixte.log — SHA-256 f98b6abe59ab19672bc89a54af13563f5344b0ab15916a01012b077d949b159a.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r2-native152_horloges.log — SHA-256 098038f81660c77c09161fcba278b2edeade26743b0d14881a25cc117b9f63ea.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r2-native152_vue.log — SHA-256 a844f818a004c763755b47d6ebe8981fb9555f56157eb41df92835413855cfa5.

## US1/US2 R3 T019 — oracle intermédiaire corrigé, exigences finales conservées

Run ciblé : **1 PASS/0 FAIL, RC0**, durée totale 4,06 s, tests 0,11 s.
Cinq empreintes source stables pendant ce run selon son JSON.
La correction porte uniquement sur l’oracle immédiat de la fixture :
capture_reply garde waiting_for_children avec le corps durable du résultat.
L’état result_available n’est attendu qu’après le tick.

L’auto-relecture documentaire du test courant confirme les assertions finales :
vraie trame DeliverIdempotent au parent, identifiant native-result-{task_id},
corps exact, contrôle toujours paused, result_sent=true, cleanup_done=true
et état result_available. Aucune assertion finale de remise ou de cleanup
n’est retirée. Le journal montre waiting_for_children avec résultat retenu,
puis résultat remis sous pause et cleanup=true ; le test passe.

Référence de la lecture, sans nouvelle exécution :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:2921.

T019 est désormais cochée. Les six clôtures nouvelles de R2/R3 portent
la progression à **24/50 (48 %)**. Les autres lignes et cases restent
identiques. Les preuves actuelles US1/US2 réunissent **17 PASS R2 + 1 PASS R3**,
soit 18 PASS ciblés ; il ne s’agit pas d’un run unique ni d’une suite globale
réexécutée après R3.

Racine privée /tmp/b.pqcun1a2, mode 0700 observé.
Preuves R3 conservées sans remplacement :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r3-t019-results.json — SHA-256 efe752510d30297f379f264c91dfb0e6cf8c6af751d7a45a7d90aeff7bef4b11.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r3-t019-native152_pause_resultat_acquis_reste_remis.log — SHA-256 4dbfea7d63566cbbac9c5cdd709475452fecba6fb95ae91b595c4fb794c18b7b.

Les **19 RED historiques de la reproduction complète** (14 natifs + 5 E2E)
restent distincts de ces GREEN. R1 de compilation, les trois reprises en échec
avant ACK et le premier oracle T019 ne sont pas réécrits. Les finitions ne
prétendent pas que ces étapes avaient déjà validé tout le correctif.

T030 reste ouverte malgré ses douze PASS de magasin : intégration API et GREEN
des refus encore attendus ; T029/T031 restent ouvertes. Le principal donne GO
à US3 avant US4. T033/T035 restent préparées en mémoire avant les GREEN
US3/US4/US5 et les finitions. Aucune livraison ou activation n’est constatée.

## Limite US3 T004a/T047 — candidat récupérable non prouvé par le RED initial

Le principal constate que fleet.recover_commands ferme le Starting initial
sans desiredfleet.json. Les fixtures RED T004a/T047 rouvraient avant cette
persistance : leurs échecs native unreachable sont conservés, mais ne prouvent
pas un enfant Launching récupérable.

La frontière réelle sera représentée par register_connected_observed existant :
desired sauvegardé, puis Err avant l’issue SQLite, tâche encore Starting.
Les futurs GREEN devront fournir les observations desired/phase/lease/lien.
Ce sont les cas déjà prévus de T004/T047 ; aucun nouveau hook ou périmètre.
Ces tâches restent ouvertes avant leurs preuves complètes. Le détail et la
portée avant/après sont consignés dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/research.md.

Les 19 RED initiaux restent des résultats bruts compilés en échec. Cette note
limite l’interprétation de leurs deux cas de lancement ; elle ne transforme
pas les anciens montages en candidats de reprise attestés.

## US3 R1 — huit PASS lib, deux PASS E2E et trois courses d’oracle

Le principal a exécuté les cinq filtres du JSON US3 R1.
Les quatre filtres lib comptent **8 PASS/0 FAIL**, tous RC0.
Le filtre E2E compte **2 PASS/3 FAIL, RC101**. Ce lot n’est pas déclaré
entièrement vert. Les tests filtrés et la suite globale restent hors de
ces compteurs. Aucune exécution par le propriétaire documentaire.

| Cible et filtre | PASS | FAIL | RC | Durée totale |
|---|---:|---:|---:|---:|
| lib, native152_panne | 4 | 0 | 0 | 10,72 s |
| lib, native152_rouge_panne | 2 | 0 | 0 | 3,56 s |
| lib, native152_rouge_trois_crashes_avant_envoi | 1 | 0 | 0 | 3,44 s |
| lib, native152_rouge_identite_frontiere_instance_neuve_flotte_ancienne | 1 | 0 | 0 | 3,34 s |
| native_delegation_e2e, native152_rouge | 2 | 3 | 101 | 25,25 s |

Les PASS lib prouvent les refus d’échéance/flotte non vérifiable, la conservation
byte-identique des états protégés, le sur-comptage après crash, queued avec
propriétaire perdu, la panne pendant une pause active ou levée, la frontière
instance neuve/flotte ancienne, et les gardes du lancement repris.

### Frontière récupérable T004a/T047 désormais observée

Les journaux fournissent desired=Running, phase Starting, bail et lien attestés
avant réouverture. Le candidat de reprise est retrouvé après réouverture.
T004a montre starting, aucune erreur, une reprise et bootstrap avec instance_id
et mission_id. Les témoins restent fermés : révocation →
permission_not_inherited ; source modifiée → settings_revision_changed ;
bail expiré → spawn_timeout ; chaque refus a zéro reprise et aucun bootstrap.

T047 passe pour starting puis mission_pending : les deux premiers crashes
conservent l’état avec compteurs 1 puis 2 ; le troisième rend
native_resume_exhausted, compteur 2 et zéro reprise. La panne pendant pause
garde working et réconcilie interrupted, pause active ou déjà levée.

Ces données complètent la limite de la fixture initiale décrite plus haut.
Elles attestent la nouvelle frontière réelle ; elles ne prétendent pas que
l’ancien RED avant desiredfleet.json avait déjà prouvé la récupérabilité.
Les anciens RED restent intacts.

### Trois FAIL E2E conservés avant correction des oracles

Les deux PASS E2E sont lien absent et marqueur natif lisible différent.
Les trois FAIL sont :

| Test | Arrêt réellement inscrit au journal |
|---|---|
| native152_rouge_panne_mission_working | comparaison du child_agent_id trop tôt : ancien ID encore lu |
| native152_rouge_identite_frontieres_flotte_neuve_tache_ancienne | started non encore trouvé pour la nouvelle instance attendue |
| native152_rouge_identite_frontieres_marqueur_deja_reconcilie | group_exists encore vrai lors de l’assertion immédiate |

Le principal a ensuite diagnostiqué empiriquement ces courses d’oracle.
Il rapporte des started réels des nouvelles générations, puis la disparition
finale du groupe sans PID orphelin. Ces observations sont celles du principal ;
le propriétaire documentaire ne les attribue pas à une observation personnelle
des processus. Les trois racines isolées indiquées sont :

- panne : /private/tmp/n148-8c139604cf.
- flotte : /private/tmp/n148-4af8c04beb.
- marqueur réconcilié : /private/tmp/n148-40107f7ecc.

Le propriétaire qualité corrige les E2E, sans modifier la production pour
ces courses. Le futur GREEN devra atteindre les assertions sur la nouvelle
génération et la disparition du groupe ; les FAIL R1 ne sont pas effacés.
T022/T023/T024 et les autres tâches US3 ne sont pas clôturées sur le seul
compteur agrégé : la correspondance de leurs cas complets et le GREEN restent
nécessaires. Aucune nouvelle case n’est cochée dans cette mise à jour.

Racine privée du lot /tmp/b.m8bln9q2, mode 0700 observé.
Le JSON atteste sept empreintes source stables pendant les cinq runs.
Commandes et résultats exacts, sans remplacement :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r1-results.json — SHA-256 17a261dd46ab9ab58b4e723f9f2f64cfd3372d5831e8f204611dbe6e5b3e4ac9.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r1-native152_panne.log — SHA-256 3fc45f09a5c5c571887c4079c8e01512ffd96544a4ccb822c6b9c7b33832c397.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r1-native152_rouge_panne.log — SHA-256 3eca2515ef1403c25f53f30c89c2caee3466f5ad4993e120c2efd5f48c49c850.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r1-native152_rouge_trois_crashes_avant_envoi.log — SHA-256 fe93e7b96a804af2ba3e935d41c73624d5c466b19381c331474e187d82083b0c.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r1-native152_rouge_identite_frontiere_instance_neuve_flotte_ancienne.log — SHA-256 be8e85493f38d6de90e6ab9cdcf902f5ac0bacb8983e5d87f69a77f7c4081e04.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r1-native152_rouge.log — SHA-256 df981e55e2a33eca6cb31e120e3e57bf491951229a382de335f3cdf4ee972adb.

## T022 régression148 R1 — comparaison de deux projections différentes

Filtre native148, cible lib : **22 PASS/1 FAIL/0 ignoré, RC101**, durée totale
4,26 s, tests 0,81 s. Le FAIL est
native148_cut_after_admission_retains_exact_ids_and_frozen_model.

Le journal compare un task_view pur à la réponse actuelle de l’API.
Les deux objets ont les mêmes valeurs pour les champs présents des deux côtés ;
la réponse API ajoute suspended=false et resume_attempts=0, conformément à
FR-024/T021. Le modèle custom/model et les identifiants sont identiques dans
les deux objets. Ce FAIL de comparaison n’est pas classé comme échec hérité.

Le principal corrige uniquement l’oracle existant : current_task_view est
comparé strictement avant/après l’admission puis après réouverture. Les contrôles
sur les IDs, le modèle gelé et les droits restent exigés. La correction ne retire
pas les deux champs API et ne remplace pas l’égalité stricte par une comparaison
partielle. Le texte T032 est étendu à ce cas148, sans nouvel ID ou test.
T022/T032 restent décochées avant leur preuve GREEN complète.

Racine privée /tmp/b.knyvrji8, mode 0700 observé.
Le JSON atteste les mêmes sept empreintes source pendant ce run.
Preuves brutes conservées :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/t022-regression-152-r1-results.json — SHA-256 02c6627fb2e66848b2086a32fb0edbc6a82194a32e2317edcd919a4cf330331f.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/t022-regression-152-r1-native148.log — SHA-256 f624ab54b06862f7e3ca280ba99d0b9fd5564b7b2905d597ac317ec722b3fa63.

La progression reste **24/50 (48 %)**. T033/T035 restent préparées en mémoire.
Les précédents GREEN, les RED initiaux et les FAIL R1 gardent chacun leur portée.
Aucune livraison ni activation de production n’est constatée.

## US3 R2 — régression148 GREEN, quatre frontières E2E GREEN, mission imbriquée ouverte

Le principal a exécuté les deux filtres de cette tranche. Le JSON atteste
sept empreintes source stables pendant les deux runs. La racine privée est
/tmp/b.19cdj_qi ; son mode 0700 est observé lors de la lecture documentaire.

| Filtre et cible | Résultat du run | Code de retour | Durée commande / tests |
|---|---|---|---|
| native148, lib | 23 PASS / 0 FAIL / 0 ignoré | 0 | 5,23 s / 0,80 s |
| native152_rouge, native_delegation_e2e | 4 PASS / 1 FAIL / 0 ignoré | 101 | 24,14 s / 20,96 s |

Les commandes consignées dans le JSON sont exécutées dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions :

```bash
python3 scripts/build.py cargo test --locked --offline -p bridget-daemon --features test-support native148 --lib -- --test-threads=4 --nocapture
python3 scripts/build.py cargo test --locked --offline -p bridget-daemon --features test-support native152_rouge --test native_delegation_e2e -- --test-threads=4 --nocapture
```

Le test native148_cut_after_admission_retains_exact_ids_and_frozen_model est
GREEN dans le journal R2, ligne 24. Le FAIL R1 de comparaison entre task_view
pur et current_task_view reste conservé dans sa preuve historique. Cette
régression verte ne clôture pas T022/T032 dans leur totalité.

Les quatre cas E2E GREEN sont :

- native152_rouge_identite_frontieres_lien_absent ;
- native152_rouge_identite_frontieres_marqueur_lisible_different ;
- native152_rouge_identite_frontieres_flotte_neuve_tache_ancienne ;
- native152_rouge_identite_frontieres_marqueur_deja_reconcilie.

Le seul FAIL est native152_rouge_panne_mission_working. Le journal, lignes
30–34, montre l’assertion nested152 absent pour l’instance
3bc90d3a-56d5-4af4-949d-de015e47e530, dans
/private/tmp/n148-d5b7866569. Le diagnostic réel reste en cours. Le principal
rapporte que le fournisseur Python de la fixture refuse l’argument --resume ;
une adaptation de fixture est en préparation. Cette information est attribuée
au principal. Elle ne constitue ni un défaut de production établi ni un GREEN
de la mission imbriquée. Le cas Working reste ouvert au-delà des étapes déjà
atteintes ; les quatre PASS d’identité ne le clôturent pas.

La limite des gardes reste explicite : révocation, source modifiée et bail
expiré sont GREEN sur le vrai candidat Starting dans
native152_rouge_panne_starting_bootstrap_et_gardes. Les refus de révocation et
de source modifiée sur le chemin déjà connecté n’ont pas encore été exécutés.
Le principal autorise leur intégration au même cas T004/T024, sans nouveau
scénario. Le bail expiré concerne uniquement Starting.

La preuve R1 de la frontière Starting récupérable reste distincte du RED
initial sans desired sauvegardé. Ce RED initial ne prouvait pas un candidat
récupérable. Les reprises, les états SQLite et les observations de processus
gardent leurs preuves et leur portée respectives. R2 fournit des tests lib et
des E2E en daemon isolé avec fixtures ; aucun modèle réel ou runtime de
production n’est validé par cette tranche. La lecture documentaire n’exécute
aucun test ni contrôle de processus.

Preuves R2 conservées sans remplacement :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r2-results.json — SHA-256 7cd942967e31abb8464f712c565cee77c25fed8eaacd78a3e40fb648198faab7.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r2-native148.log — SHA-256 207e33711b229c0f64fe6babea0359e824debb1f0fb023df4e75c2ef8fd3745e.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r2-native152_rouge.log — SHA-256 92462cfcbe1271c09b95c36d9c57383bf571d4cf9d8dcae5bc02c33f12ca5156.

Aucune nouvelle case US3 n’est cochée. La progression reste **24/50 (48 %)**.
T022/T023/T024/T032 restent ouvertes. T033/T035 restent préparées en mémoire.
Les preuves antérieures et les fichiers de validation restent inchangés.
Aucune livraison ni activation de production n’est constatée.

## US3 R3 gardes — sept sous-cas des deux branches, sans clôture E2E

Le filtre lib native152_rouge_panne compte **2 PASS/0 FAIL/0 ignoré**, RC0.
La commande dure 7,20 s ; les tests durent 0,68 s. Le JSON atteste six
empreintes source stables pendant ce run. La racine /tmp/b.ydm6o17s porte
le mode 0700 lors de la lecture documentaire.

Le même test native152_rouge_panne_starting_bootstrap_et_gardes contient
maintenant sept sous-cas. Aucun scénario autonome n’est ajouté. Le second
test GREEN est native152_rouge_panne_pendant_pause : pause active puis pause
levée, tâche working, exécution interrupted, une reprise préparée.

| Branche | Garde | État / erreur constatés | Compteur restart_resumes | Reprises préparées | Ligne du journal |
|---|---|---|---|---|---|
| Starting | valide | starting / aucune | 1 | 1 | 20 |
| Starting | révocation | failed / permission_not_inherited | 0 | 0 | 26 |
| Starting | source modifiée | failed / settings_revision_changed | 0 | 0 | 33 |
| Starting | bail expiré | failed / spawn_timeout | 1 | 0 | 36 |
| Connected | valide | mission_pending / aucune | 1 | 1 | 39 |
| Connected | révocation | failed / permission_not_inherited | 0 | 0 | 42 |
| Connected | source modifiée | failed / settings_revision_changed | 0 | 0 | 45 |

Source relue :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs,
lignes 2786–2902. La liste des sept couples est en 2788–2792. Les deux
frontières sont réellement distinctes dans la fixture : register_connected
pour Connected ; register_connected_observed avec erreur après sauvegarde
desired, avant issue SQLite, pour Starting (2831–2841). La lecture conserve
donc la limite de l’ancien RED sans desired sauvegardé.

Les assertions 2872–2898 imposent une reprise et le bootstrap corrélé pour
les branches valides ; la branche Connected prépare la génération 2.
Le rôle native_delegate et le delegation_id sont conservés. Les refus
imposent failed et zéro reprise préparée. Les refus révocation/source imposent
leur code exact et le compteur zéro. Le refus bail impose spawn_timeout ;
son compteur 1 est une observation du journal, sans assertion dédiée dans
ce bloc. Le bail expiré n’est testé que sur Starting.

Les gardes Connected précédemment non exécutées dans R2 sont désormais
GREEN dans cette preuve lib. Le test vérifie les réservations, le bootstrap
et les états sauvegardés. Il ne réalise pas le lancement d’un fournisseur
ou la validation de la mission imbriquée en daemon réel. Les quatre PASS
E2E et le FAIL nested152 de R2 gardent leur portée ; l’E2E imbriqué reste ouvert.

Le principal atteste le préfixe de production inchangé. La lecture courante
des 97 374 premiers octets de native_delegation.rs donne SHA-256
ae35743ca9baa3a6bbe1e6b22dccbc4107c76d677f47bb42daf65307347a3746.
Cette capture courante ne remplace pas la comparaison avant/après du principal.

Preuves de la tranche, conservées séparément :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r3-gardes-results.json — SHA-256 2720f73edb73e3ca50b83697b8f3cf15fc91dbe4444760527d1b866e2497cf4e.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r3-gardes-native152_rouge_panne.log — SHA-256 4e9e2c141cf3ab35e4e79cfffb1033c8b724ef9d79d032cf6c7b553c26bd0120.

Commande exécutée par le principal dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions :

```bash
python3 scripts/build.py cargo test --locked --offline -p bridget-daemon --features test-support native152_rouge_panne --lib -- --test-threads=4 --nocapture
```

La progression reste **24/50 (48 %)**, sans nouveau cochage US3.
T033/T035 restent en mémoire. Aucun test, processus ou service n’est exécuté
par le propriétaire documentaire. Aucune livraison ni activation n’est constatée.

## US3 R3 E2E — fournisseur initialisé, reconstruction de mission encore manquante

Le run E2E native152_rouge compte **4 PASS/1 FAIL/0 ignoré**, RC101.
La commande dure 23,98 s ; les tests durent 20,85 s. Le JSON atteste sept
empreintes source stables pendant ce run. La racine /tmp/b.5nzzyd80 porte
le mode 0700 lors de la lecture documentaire. Ce run reste distinct des
2 PASS lib R3 gardes et des tranches R1/R2.

Les quatre frontières d’identité sont GREEN : marqueur lisible différent,
lien absent, marqueur déjà réconcilié, flotte neuve/tâche ancienne.
Le seul FAIL reste native152_rouge_panne_mission_working. Le journal, lignes
30–34, indique nested152 absent pour l’instance
d0f0af8f-29da-40dc-93d2-b87b7aae8688 dans
/private/tmp/n148-6b23b800e8. Cette instance et cette racine appartiennent
à R3 ; elles ne remplacent pas celles du FAIL R2.

Le nouveau diagnostic réel est transmis et confirmé par le principal,
le propriétaire qualité et le propriétaire façade :

- les arguments --resume du fournisseur sont corrects ;
- le message d’initialisation est reçu et la session est enregistrée ;
- stderr est vide, mais aucun prompt de mission n’est remis à la génération 2 ;
- la tâche est Working ; son exécution reste Starting, génération 1,
  révision 0, instance NULL ; ces deux états appartiennent à deux objets distincts ;
- la remise est réaffectée à la génération 2 puis passe indeterminate ;
- aucune continuation n’est créée.

Ce diagnostic ne retient plus le refus d’argument Python comme cause du
FAIL R3. Le constat précédent reste historique pour R2. Le principal et
les deux propriétaires identifient le chemin schedule_execution_recovery
du daemon : après la mise en quarantaine indeterminate, il sort au lieu
de reconstruire la mission. Le propriétaire documentaire rapporte cette
lecture et ces données ; il ne les produit pas par une nouvelle exécution.

Le principal donne le GO à une correction de production minimale dans ce
chemin du daemon. L’E2E doit rester inchangé pour la vérification suivante.
L’hypothèse de correction est en cours ; aucun GREEN de reconstruction
ou de mission imbriquée n’est encore déduit du diagnostic.

Preuves brutes R3 E2E conservées séparément :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r3-e2e-results.json — SHA-256 3dedfa1e0343fd1d69557c29e7764138c39605c36303896ecf7ee837bd72dee3.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r3-e2e-native152_rouge.log — SHA-256 7a707deb2aca0768a05f1c401a9fd3ae336c039d426d1979f073d577e418b72b.

Commande exécutée par le principal dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions :

```bash
python3 scripts/build.py cargo test --locked --offline -p bridget-daemon --features test-support native152_rouge --test native_delegation_e2e -- --test-threads=4 --nocapture
```

Cette preuve utilise un daemon isolé et un fournisseur de fixture. Elle
ne valide aucun modèle réel ni runtime de production. Les RED initiaux,
R1, R2, le vrai candidat Starting et R3 gardes restent conservés avec
leur portée. Aucun nouveau cochage US3 : **24/50 (48 %)** reste inchangé.
T033/T035 restent préparées en mémoire. Rien n’est livré ni activé.

## US3 R4 — mission réellement remise, petit-enfant refusé par les permissions de fixture

Le principal a exécuté les trois filtres du JSON R4. Les sept empreintes
source sont stables pendant cette tranche. La racine /tmp/b.fdupm76i porte
le mode 0700 lors de la lecture documentaire.

| Filtre et cible | Résultat | Code de retour | Durée commande / tests |
|---|---|---|---|
| native152_rouge, native_delegation_e2e | 4 PASS / 1 FAIL / 0 ignoré | 101 | 10,07 s / 5,78 s |
| native152_pause, lib | 6 PASS / 0 FAIL / 0 ignoré | 0 | 6,89 s / 0,47 s |
| native152_rouge_pause, lib | 5 PASS / 0 FAIL / 0 ignoré | 0 | 3,84 s / 0,30 s |

Les 6 PASS et 5 PASS sont deux runs de pause distincts. Ils ne transforment
pas le FAIL E2E en GREEN. Les quatre frontières d’identité E2E passent encore.
Le seul FAIL est native152_rouge_panne_mission_working. Le journal, lignes
30–36, contient cette fois un événement nested152 : la demande MCP de
petit-enfant rend native_delegation_refused/permission_source_unavailable
avec isError=true, pour l’instance 22443631-8caa-4360-9fa9-ba31ce677af6.
Le blocage n’est donc plus l’absence de prompt de R3.

Le principal et les propriétaires de code/qualité constatent la vraie remise
après le correctif minimal du daemon : exécution génération 2, état running,
provider_accepted. Ce constat runtime est attribué à leurs diagnostics.
La tâche native et l’exécution restent deux objets distincts. Le FAIL actuel
porte sur l’admission du descendant. La fixture héritée possède
permission_snapshot=null ; son PATH ne fournit pas de CLI claude et son
provider.py est un script avec shebang. permission_source_unavailable est
un refus correct de cette source non attestable. Aucune garde de production
n’est affaiblie pour le faire disparaître.

### Montage contraint T004 avant le prochain run

Décision du principal : réutiliser le montage149 avec racine .git privée,
profil de permissions privé et CLI directe nommée claude dans bin/PATH.
Le propriétaire qualité possède seul l’E2E. Il complète uniquement le
fournisseur de test privé : un binaire compilé incorpore le Python existant,
puis l’exécute avec l’interpréteur existant. L’empreinte du CLI couvre ainsi
le texte incorporé. Ce montage reste dans le cas T004 déjà prévu.

Les références149 ont été relues dans ce worktree :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs:42 — montage privé, CLI directe, borne .git, bin/PATH et CLAUDE_CONFIG_DIR, jusqu’à la ligne 61.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs:133 — capture_context_with_env réelle, ligne 134.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs:405 — permissions obtenues par les faits et l’admission ; le snapshot est constaté après admission, ligne 412.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/native_permissions149_tests.rs:26 — même CLI directe privée et borne .git, jusqu’à la ligne 35.

Le montage courant préparé en source est relu dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/tests/native_delegation_e2e.rs :
répertoires/profil à partir de 152, PATH privé à partir de 210,
compile_provider152 à partir de 438 et init/settings à partir de 508.
Cette lecture ne prouve pas sa compilation ni son exécution.

Le principal confirme /usr/bin/cc et /usr/bin/clang disponibles. Le
compilateur C fait déjà partie des besoins de rusqlite bundled :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/Cargo.toml:18,
hérité par
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/Cargo.toml:29.
Aucune nouvelle dépendance, commande produit ou installation d’outil n’est
introduite. Aucun snapshot de permissions n’est injecté. La source de
permissions doit être capturée par la voie réelle de CLI/settings/init.

La prochaine recette doit fournir le chemin du CLI, sa SHA, les réglages
privés et leurs révisions, puis l’initialisation réelle du fournisseur.
Elle doit obtenir le snapshot par l’admission et conserver les contrôles
de droits, d’identité et de filiation. Aucun succès de ce nouveau montage
n’est déclaré avant le run du principal.

Preuves R4 conservées séparément :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r4-results.json — SHA-256 0afc00a82b441108a240c7c29558f62bbf0ec707081b912aed13e4a02175db25.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r4-native152_rouge.log — SHA-256 340eb93fe93a2918356c0c189c8445fd43839a52bdfbc2a5ef8992f19ab9d62a.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r4-native152_pause.log — SHA-256 41ca37f1d72abd51499f5e37fea58c2d9b211efb91f622ac62ec9311c1eb67ab.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r4-native152_rouge_pause.log — SHA-256 b4b24523e3d676ed6c83998ad3d94953d37dd1330ac295dc647fb9638911fc84.

Les commandes exactes restent dans le JSON R4. Les RED initiaux et toutes
les tranches R1/R2/R3 sont conservés. US3 reste ouverte, sans nouveau
cochage : **24/50 (48 %)**. T033/T035 restent en mémoire. Cette recette
isolée ne valide aucun modèle réel ni runtime de production. Rien n’est
livré ni activé par le propriétaire documentaire.

## US3 R5 puis R6 — transport attesté corrigé, E2E cinq PASS

Les deux tranches exécutent le même filtre E2E native152_rouge. Chacune
atteste sept empreintes source stables pendant son run.

| Tranche | Résultat | Code de retour | Durée commande / tests | Racine privée |
|---|---|---|---|---|
| R5 | 4 PASS / 1 FAIL / 0 ignoré | 101 | 9,41 s / 5,92 s | /tmp/b.z_ec8ufn |
| R6 | 5 PASS / 0 FAIL / 0 ignoré | 0 | 9,83 s / 6,19 s | /tmp/b.2gqgi6lx |

Les deux racines portent le mode 0700 lors de la lecture documentaire.
Les JSON montrent un seul changement parmi leurs sept empreintes :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs.
L’E2E conserve exactement la SHA
298e0f13699815da7bc12c401590c802a708a4a13a4d069823e9d06c3f469b91.
Le GREEN R6 ne résulte donc pas d’un nouvel affaiblissement de cet oracle.

### R5 — montage privé observé, fact non accepté

Le FAIL R5 est native152_rouge_panne_mission_working. Son journal, lignes
30–36, contient nested152 pour l’instance
7b6da87e-1dcb-4855-84e6-70cc108b97d2, avec
native_delegation_refused/permission_attestation_unavailable et isError=true.
Il reste distinct du permission_source_unavailable de R4.

Les fichiers de la racine /private/tmp/n148-57662a4e22 ont été relus :

- /private/tmp/n148-57662a4e22/bin/claude : 33 576 octets, SHA-256 22bee2ca9c270d329d09e975fe9efa06a83047de4ec0c44b5785c0b32a9a6269.
- /private/tmp/n148-57662a4e22/provider/.claude/settings.json : 107 octets, SHA-256 a015ea64b13ebeee20199cc113e5b9c1d1caca73d6af15906ecc68e6f19d26f3 ; mode plan, Bash/Edit/Write/NotebookEdit/WebFetch/WebSearch refusés.
- /private/tmp/n148-57662a4e22/state/agents.json : SHA-256 bb8c5aaa4bec6c3020824c5ea4b4cb41e0bd866783d82395f1821f1c769dcc15 ; command pointe sur ce bin/claude et claude_config_dir sur ce profil privé.
- /private/tmp/n148-57662a4e22/evidence.jsonl : SHA-256 88c8230cc8bb333adf24cc3fae0004abecdd93b619d42b8cea1453f463e4aec6 ; init152 aux lignes 3 et 6, mode plan, outils Read/Glob/Grep, cwd privé exact, argv --resume pour la génération 2.

Ce sont des empreintes de fichiers R5 et des observations de fixture.
Le fact de permissions est en mémoire, non persisté publiquement. Ces SHA
ne sont pas celles d’un fact publiquement accepté et ne sont pas extrapolées
au CLI d’une autre racine ou au run R6.

La contre-revue de la façade identifie le refus dans la garde du daemon.
definition_presence_fields restitue le protocole réel claude_stream_json,
alors que la liste de transports admis de R5 l’excluait. Le bootstrap/init
du fournisseur était conservé et lu ; le défaut n’était plus celui du montage
privé. La correction minimale R6 admet ce transport réel dans la liste.

Lecture du code courant :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:6311,
protocole en ligne 6315 ; garde aux lignes 16849–16855.
La clause claude_stream_json est en ligne 16852. Les contrôles d’instance,
de driver du registre, de request_id, de révision et de structure des
permissions restent présents aux lignes 16845–16872. Aucun snapshot de
permissions n’est injecté pour obtenir le GREEN.

### R6 — deux reprises réelles et descendant après chaque reprise

Le journal R6, ligne 30, donne GREEN pour
native152_rouge_panne_mission_working, avec les quatre frontières d’identité.
La source du cas a été relue dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/tests/native_delegation_e2e.rs:710.
La boucle exécute deux crashes/redémarrages du même daemon et de ses magasins.
Elle exige task_id et message_id inchangés, une nouvelle instance à chaque
reprise, compteur 1 puis 2, propriétaire racine et filiation conservés.
assert_native_identity152 contrôle l’identité/bootstrap. Le MCP du fournisseur
admet le descendant après chaque reprise ; le petit-enfant garde son task_id,
sa mission, son propriétaire initial et son parent. Sa deuxième instance
est reprise avec son propre compteur. Ces assertions distinguent mission,
exécution, identité de processus et état de tâche.

Le montage privé T004 est donc exécuté avec succès pour ce contrat E2E.
Le fournisseur reste une fixture fermée ; aucun modèle réel ou runtime de
production n’est validé. Le fact mémoire n’est pas présenté comme un reçu
public sérialisé de ses permissions.

### Correspondance US3 avant cochage

Cette table prépare la clôture des onze tâches US3. Elle ne coche aucune case.

| Tâche | Réalisation et preuve ciblée | État de preuve |
|---|---|---|
| T004 | R3 gardes : Starting/Connected, permissions et bail ; R6 E2E : deux magasins relus, bootstrap, filiation et petit-enfant. | Cas ciblés GREEN. |
| T022 | native_child_env, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:915 ; native148 R2 23 PASS ; gardes R3 des deux branches et bootstrap R6. | Extraction et usages couverts ; régressions finales de T032 encore distinctes. |
| T023 | prepare_restart, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1372 ; R1 panne/états protégés/bornes, R3 refus, R6 Working repris. | Cas ciblés GREEN. |
| T024 | reserve_managed_recoveries, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:4189 ; Starting réel et Connected en R3 gardes, descendant et second redémarrage en R6. | Deux branches ciblées GREEN. |
| T025 | R1 : borne Starting/mission_pending et Working, sur-compte avant effet ; R4 : reprise de pause sans incrément. Le résultat unique après panne reste à vérifier. | Partielle ; confirmation finale du principal attendue. |
| T038 | native152_rouge_panne_pendant_pause, R3 gardes : pause active puis levée, working/interrupted et une reprise préparée. | GREEN lib, distinct de l’E2E. |
| T041 | R3 panne pendant pause ; R4 native152_rouge_pause_sans_tick_admission_crash_reconnexion et native152_rouge_pause_sans_tick_echeance_retrouvee. | Pause/échéance/sans tick GREEN. |
| T042 | Transition pause_restart_reconciled, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1402 ; R3 constate execution=interrupted ; R4 attente interrupted puis chaîne reprise. | Cas ciblés GREEN. |
| T043 | current_command/restart_identities, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1259 ; R6 filiation/descendant et frontières d’identité, R1 instance neuve/flotte ancienne. | Cas d’identité lisible ciblés GREEN ; corruption de marqueur hors contrat. |
| T047 | native152_rouge_trois_crashes_avant_envoi, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:2732 ; R1 Starting puis mission_pending, deux comptes avant épuisement. | GREEN lib. |
| T048 | Quatre E2E d’identité R6 et frontière inverse lib R1 ; marqueur lisible différent et déjà réconcilié conservés. | Cas ciblés GREEN. |

Le principal confirme les oracles déjà verts de T025 : reprise de pause
avec continuation différente et compteur zéro, dans
native152_pause_chaine_suivie_apres_reprise et
native152_pause_resultat_rendu_une_fois (6 PASS du filtre pause R4).
Les fonctions courantes sont relues à partir des lignes 3137 et 3190 de
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs ;
les compteurs zéro sont assertés aux lignes 3151 et 3224.
La borne Working et le sur-compte avant tout effet sont relus aux lignes
2996–3015 du même fichier et restent reliés au GREEN R1.

Le complément native152_panne_resultat_rendu_une_fois est maintenant posé
par le propriétaire de code dans le cas prévu, à partir de la ligne 3019
du même fichier. Sa lecture montre mission reprise, ACK réel, résultat
corrélé, second ACK réel, absence de deuxième remise et flags durables.
Cette lecture n’est pas une exécution : sa preuve GREEN manque encore
à cette étape. Seul cet oracle après panne retient la clôture complète T025.
Le compteur **35/50 n’est pas encore un statut validé**. T032 reste une
régression de finition distincte, pas la somme de ces tranches partielles.

Preuves brutes conservées :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r5-results.json — SHA-256 613fe1bdc4b24c66455b7ee3e8035f9a56df2d69e6b7f7f1dc117df92fe9e8bc.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r5-native152_rouge.log — SHA-256 163a4aefcf82d274a65296d9d2a0dd1ecdde3bb5ee94eb89a1bc26503951b936.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r6-results.json — SHA-256 d2ad250222a072da17d3485204d273fa782fa72ea0cc6f17fd389f2d6c5e2373.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r6-native152_rouge.log — SHA-256 a2fa1f16ae64e88f9320d2aac7da5374c6186faf5196bda2265b3988e3571ae8.

Les commandes exactes restent dans les JSON. Les RED, les diagnostics et
les tranches antérieures ne sont ni remplacés ni reclassés. Les tâches
restent **24/50 (48 %)**. T033/T035 restent en mémoire. Aucune écriture de
code, exécution de test ou opération Git/service n’est faite par cet owner.

## Clôture US3 — T025 R4 GREEN, onze tâches cochées

Le principal autorise la clôture des onze tâches US3 après vérification de
T025. Le filtre native152_panne_resultat_rendu_une_fois compte désormais
**1 PASS/0 FAIL/0 ignoré**, RC0. Ce résultat est distinct des **5 PASS E2E
R6**, des **2 PASS R3 gardes avec sept sous-cas** et des runs de pause.
Il ne constitue pas une suite globale de tous les tests152.

### T025 : quatre passes conservées, corrections de fixture seules

| Passe | Résultat / retour | Durée commande / tests | Racine privée | Diagnostic transmis par le principal et constat du journal |
|---|---|---|---|---|
| R1 | 0 PASS / 1 FAIL, RC101 | 7,33 s / 0,11 s | /tmp/b.36os54t7 | StaleGeneration à la connexion : la fixture présente la mauvaise instance ; utiliser celle du bail repris. |
| R2 | 0 PASS / 1 FAIL, RC101 | 7,56 s / 0,20 s | /tmp/b.5wofez3m | WouldBlock à la lecture de mission : le contrôle différé après Registered reste non drainé ; exécuter les contrôles comme le serveur. |
| R3 | 0 PASS / 1 FAIL, RC101 | 4,78 s / 0,21 s | /tmp/b.edm6wyin | La reprise Working/génération2 est observée ; les flags finaux échouent. Le principal constate la remise SQLite acked, mais la fixture omet maintain_stores. |
| R4 | 1 PASS / 0 FAIL, RC0 | 4,67 s / 0,21 s | /tmp/b.m9dafyip | Contrôles post-Registered exécutés, puis entretien normal du magasin après ACK ; remise unique, ack=true, cleanup=true et compteur1. |

Les quatre JSON attestent sept empreintes source stables pendant chaque
run. Les quatre racines portent le mode 0700 lors de la lecture documentaire.
Les trois FAIL compilés sont conservés ; ils ne sont ni effacés ni présentés
comme des bugs de production corrigés. Le principal confirme des corrections
de fixture seules. La lecture du fichier final confirme la SHA
609f6d902152b57eff853c1927ff6769c6ec127d0f47ea9c7da278695f33eee9
de native_delegation.rs. Son préfixe de production de 97 374 octets garde
la SHA-256 ae35743ca9baa3a6bbe1e6b22dccbc4107c76d677f47bb42daf65307347a3746,
identique à la capture antérieure au complément T025.

### Oracle final relu et exécuté

Le cas final se trouve dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:3019.
Il reprend un seul enfant depuis la flotte, utilise l’instance du bail et
exécute le contrôle différé après Registered. Il lit la vraie mission
DeliverIdempotent et appelle le vrai handler DeliverAcked. La tâche reste
working, compteur1, instance reprise ; l’exécution feuille est nouvelle,
génération2. Le résultat corrélé est capturé après running/completed.

Le parent reçoit le message native-result lié à la mission, avec le corps
exact. Le vrai ACK de ce résultat est suivi de maintain_stores. Trois ticks
supplémentaires ne produisent aucune deuxième remise. Les assertions finales
imposent result_available, corps durable exact, result_sent=true,
delivery_acked=true, cleanup_done=true et restart_resumes=1.
Le journal R4, lignes 21–25, fournit la reprise puis la remise unique GREEN.

La borne deux, le sur-compte avant effet et la reprise de pause sans
incrément restent reliés aux preuves déjà vertes R1/R4 et à la matrice US3
du reçu R6. Ce dernier manque de T025 est maintenant fermé. Les réserves
T025 de la section précédente décrivent l’étape historique avant ce GREEN.

### Cochage autorisé et progression

Les seuls nouveaux cochages sont :

- T004, T038, T047 et T048 : cas rouges US3 désormais GREEN ;
- T022, T023, T024 et T025 : environnement, décision, chemins de reprise et borne ;
- T041, T042 et T043 : pause, interruption réconciliée et identité après reprise.

Ces onze cases portent la progression de **24/50 à 35/50 (70 %)**.
Le texte des cinquante tâches, leurs IDs, leur ordre et leurs dépendances
restent inchangés. Les vingt-quatre cases déjà cochées restent cochées.
Les quinze autres cases restent ouvertes. La correction d’appellation du
contrat current_task_view reste conservée. T032 et les finitions restent
distinctes ; US4 commence chez le propriétaire de code.

### Captures T025 conservées

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-t025-r1-results.json — SHA-256 2ee03ca4731b6cd595824743b7c0378ea343ce915380efbf7beeea1b4c629904.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-t025-r1-native152_panne_resultat_rendu_une_fois.log — SHA-256 f9e542f930acef1ce4f6bdd1a8e0e1e1dbfcbdb1b1dea243ceb1664602be97fd.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-t025-r2-results.json — SHA-256 841d1e37ddf42c0e65f5c63bef77f408a1ed8607db4171a77ae13bc95e0c0377.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-t025-r2-native152_panne_resultat_rendu_une_fois.log — SHA-256 2424c823a8755d60ae8914ab1a84c01f88d6c07a155633ea6df1d1d21beae8cc.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-t025-r3-results.json — SHA-256 7120f96666e8c4b1fe57df29b0f86d12d0471061bfc825d8028785ca85afc746.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-t025-r3-native152_panne_resultat_rendu_une_fois.log — SHA-256 013219a296371e1aabcb56907bd9dc6abe6a354c7f8c3179561a90a618dad3ab.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-t025-r4-results.json — SHA-256 88c813afd5595855f322c26b3f489b347f7b97702e011a2523b680281825aaa3.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-t025-r4-native152_panne_resultat_rendu_une_fois.log — SHA-256 8c3e25b68af7bd92663aba0c28479814a56ba879ad017588c630202500fce369.

Les commandes exactes restent dans leurs JSON. Les RED et toutes les
tranches précédentes sont conservés en append. La preuve T025 utilise les
magasins et handlers réels avec canaux de fixture ; elle ne valide
aucun modèle réel ni runtime de production. T033/T035 restent en mémoire
avant les GREEN US4/US5 et les finitions. Aucun code, test, Git ou service
n’est exécuté ou modifié par le propriétaire documentaire.

## Clôture US4 — précontrôle global, reçus durables et 41/50

Le principal autorise six nouveaux cochages après les trois filtres US4 R1.
Le JSON atteste sept empreintes source stables pendant les runs. La racine
/tmp/b.sxminf50 porte le mode 0700 lors de la lecture documentaire.

| Filtre lib | Résultat | Retour | Durée commande / tests |
|---|---|---|---|
| native152_annulation | 4 PASS / 0 FAIL / 0 ignoré | 0 | 11,43 s / 0,39 s |
| native152_rouge_annulation | 2 PASS / 0 FAIL / 0 ignoré | 0 | 3,16 s / 0,10 s |
| native148_cancel_stops_native_descendants_before_parent | 1 PASS / 0 FAIL / 0 ignoré | 0 | 3,19 s / 0,11 s |

Ces trois runs restent distincts. Le témoin148 conserve sa portée de
non-régression ; aucun résultat global du workspace n’est déduit.
La capture source du JSON avant le GO US5 donne native_delegation.rs
667b6751529429293ac2b37159e565d8e85623d324932fce357b81d580b78512
et daemon.rs 4a62da1e1c1b5e55b8bcc6c8ea66493ef6514ff4dd920b8499e726b0b44386b6.
Elle décrit ces runs, pas le code US5 en cours.

### Tâches et oracles acquis

| Tâche | Preuve du contrat |
|---|---|
| T007 | native152_rouge_annulation_partielle : membre non arrêtable, arrêts=[false,false], descendants starting/working inchangés, racine cancelling avec native_cancel_unverifiable. |
| T040 | native152_rouge_annulation_multi_ticks : trois ticks, aucun arrêt et mêmes états ; les descendants ne deviennent pas des racines autonomes. |
| T026 | classify_stop relu, classification sans mutation ; Desired Stopped sans superviseur conservé dans le cas crash ; témoin148 des arrêts profonds GREEN. |
| T027 | Refus global sans mutation des descendants ; nouvel essai après correction ; états terminaux failed/result_available avec erreur et résultat conservés. |
| T028 | Réessai ne rejoue que les échecs, ordre profond d’abord, confirmations durables et réouverture après l’échec SQL du reçu. |
| T046 | Descendant cancelling non coordonnateur indépendant ; reprise depuis la racine, sans changement des terminaux acquis. |

Le journal des quatre cas annulation constate :

- native152_annulation_reessai_apres_correction_et_reprise_coordonnee_depuis_la_racine : terminal failed puis result_available, erreur/résultat conservés ;
- native152_annulation_reessai_ne_rejoue_que_les_echecs_et_ordre_profond_d_abord : demandes [1,2], ordre [1,0,1], confirmations complètes ;
- native152_annulation_crash_entre_deux_arrets_confirmes : confirmations durables après réouverture ;
- native152_annulation_descendant_non_racine_independante : GREEN distinct.

### Limite exacte du crash et des arrêts

Le cas crash est relu dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:2926.
Il crée un vrai trigger SQLite RAISE(FAIL) sur le payload du reçu, après
confirmation d’un arrêt et avant sa persistance. Un reçu antérieur reste
durable. La fixture retire le trigger, rouvre DelegationStore, retire
la table de superviseur du membre confirmé, puis reprend par Desired Stopped.
Le tick suivant n’ajoute aucune demande d’arrêt ; les trois confirmations
sont présentes, état cancelled et cleanup_done=true.

Ce test prouve la frontière SQL et le rejeu durable avec des confirmations
de fixture. Ces unitaires ne lancent ni PID à arrêter ni SIG réel. Ils ne
constituent pas une recette de mort de processus. Le témoin148 est un run
distinct ; il ne transforme pas ces confirmations en preuve PID/SIG.

### Coût annoncé et périmètre de cette analyse

Le commentaire de cancel_tree est relu dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1927.
Le précontrôle et ses index locaux coûtent O(m + f + p + n log n), où
m désigne les lancements, f la flotte, p les présences et n les membres
(borne 4096 indiquée). Ce coût exclut les lectures existantes des descendants,
des liens et des marqueurs. La flotte et les records sont lus une seule fois.

La sauvegarde après chaque arrêt sérialise la liste croissante : O(n²)
octets cumulés, explicitement commentés aux lignes 1929–1930. Cette borne
décrit la persistance durable, pas une performance mesurée ou une promesse
de coût total linéaire. Aucun benchmark n’est exécuté par ce reçu.

La classification pure est relue dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:9677.
Record/Desired proviennent du même index sous verrou. Le commentaire
indique O(1) pour un membre supervisé et O(b) pour un marqueur de b octets.
La branche Desired Stopped sans superviseur est présente aux lignes 9694–9698.

### Cochage et captures

Les seuls nouveaux cochages sont T007/T040/T026/T027/T028/T046.
La progression passe de **35/50 à 41/50 (82 %)**. Les cinquante textes,
IDs, ordre et dépendances sont conservés ; les trente-cinq cases précédentes
restent cochées. Les neuf autres tâches restent ouvertes. US5 commence
chez le propriétaire de code ; T033/T035 restent en mémoire avant son
GREEN et les finitions.

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us4-152-r1-results.json — SHA-256 b3a805919117bf386397191e3a8b918daa54c82819500a4a79bc3f2885a369c7.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us4-152-r1-native152_annulation.log — SHA-256 e0bb44c3cbd79e866b4ec623b08d2c3afdd30af917912e63f0cb0c7f23a63dec.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us4-152-r1-native152_rouge_annulation.log — SHA-256 d9900b9c9ee37cd2a0cdbb946e3c0052ce5da25bd8713e78e9c7353c52d7237d.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us4-152-r1-native148_cancel_stops_native_descendants_before_parent.log — SHA-256 7320623103dc4a5b56d8d425a75bab0d9b217a14a9bd9d2ffa65ea336040eaae.

Les commandes exactes restent dans le JSON. Les RED et toutes les tranches
US1/US2/US3 restent conservés en append. Aucun modèle réel, runtime de
production ou livraison n’est validé par ces unitaires. Le propriétaire
documentaire ne modifie que les artefacts152 ; aucun code, Git ou service.

## Clôture US5 — reçus branchés à l’admission et 44/50

Après GO du principal, la capture US5 R1 prouve **15 PASS, 0 FAIL,
0 ignored, RC0** sur le filtre existant `native152_refus`. La commande a
duré 7,47 s ; les tests ont duré 0,11 s. La racine privée
/tmp/b.c4_pgsab a le mode 0700. Les sept sources ont les mêmes SHA avant
et après la commande. La relecture documentaire retrouve ces SHA.

Commande exécutée par le principal dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions :

```bash
python3 scripts/build.py cargo test --locked --offline -p bridget-daemon --features test-support native152_refus --lib -- --test-threads=4 --nocapture
```

Cette capture complète la preuve magasin antérieure de 12 PASS. Elle ne
la remplace pas. Le chemin complet d’admission et le passage d’entretien
sont maintenant exercés par les trois tests supplémentaires.

| Tâche | Réalisation relue et preuve |
|---|---|
| T029 | Le registre normalise le type inconnu en `agent_type_unknown` dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:769. Le rejeu est lu avant la pause (:719). `native152_refus_rejeu_change_de_reponse` (:3804) garde `model_unavailable` après disparition du cwd et pause ; une autre enveloppe rend `envelope_mismatch`. `native152_refus_texte_dynamique_jamais_stocke` (:3840) lit SQLite et ne trouve que le code fermé. |
| T030 | La liste exacte de cinq codes et le premier reçu immuable sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/delegation.rs:213. Le résultat de la nouvelle admission appelle ce magasin depuis /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:918. Les tests magasin prouvent le rejeu après réouverture, l’isolation du parent, la priorité de la tâche, le mismatch et dix refus transitoires sans reçu. |
| T031 | L’entretien existant appelle `maintain_retention` dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:7581. Celui-ci appelle la purge des reçus dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:2369. `native152_refus_purge_30_jours` (:3869) appelle réellement `maintain_stores` : un reçu reste à 30 jours exactement, puis disparaît à 30 jours + 1 s. |

Seuls `cwd_outside_parent_project`, `cwd_unavailable`,
`model_unavailable`, `effort_unavailable` et `agent_type_unknown`
peuvent être enregistrés. Le magasin refuse toute autre chaîne ; la
normalisation n’enregistre pas le texte dynamique du registre.

Les dix cas séparés vérifient l’absence de reçu pour `referent_paused`,
`task_limit`, `retention_saturated`, `identity_unavailable`,
`reply_timeout_exceeds_parent`, `parent_task_unavailable`,
`parent_execution_ambiguous`, `delegation_grant_required`,
`permission_not_inherited` et `permission_attestation_unavailable`.
La liste fermée exclut également tous les autres codes et textes libres ;
aucune exécution d’un cas supplémentaire n’est revendiquée.

Les refus et leur purge ne créent aucune tâche, ne changent pas Lineage
et n’émettent pas de notification d’attente. Les oracles magasin et
d’entretien vérifient ces invariants. La purge balaie les reçus ; cette
lecture de complexité O(R) n’est pas un banc de performance.

T029/T030/T031 sont cochées après cette preuve complète :
**44/50 (88 %)**. Les six finitions T032 à T037 restent ouvertes à cette
étape. T033/T035 sont désormais autorisées ; leur publication et leur
relecture seront consignées séparément. Aucune livraison ni activation
de production n’est déduite de ces tests.

Captures conservées :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us5-152-r1-results.json — SHA-256 4333222c80d976e743a94bffd932f5371fa05af8419ee12176a8d839e3269a7a.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us5-152-r1-native152_refus.log — SHA-256 ab2d206cc6afb0ec8557fc115e12315ff6e710f735f39d33208693fca1406f31.

La source native de ce run a le SHA-256
09d0eee1d14cb63d69432c94c2210f27ff14f013e60b4867c25b242a4bb77aa1.
La source daemon reste
4a62da1e1c1b5e55b8bcc6c8ea66493ef6514ff4dd920b8499e726b0b44386b6.
Les historiques RED, US1/US2/US3/US4 et magasin initial restent intacts.
Le propriétaire documentaire n’exécute aucun test et ne modifie aucun
code, Git, modèle ou service.

## Finitions documentaires T033/T035 — 46/50, compatibilité encore ouverte

GO du principal reçu après US5 GREEN. La relecture exacte des trois documents
utilisateur et de l’ADR est faite contre le contrat152 et les sources présentes.
T033 et T035 sont cochées : **46/50 (92 %)**. Les 50 identifiants sont conservés.
T032/T034/T036/T037 restent ouvertes ; aucun autre cochage n’est autorisé par
cette clôture documentaire.

Fichiers publiés et relus :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/docs/delegation-native.md.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/skills/bridget/SKILL.md.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/skills/bridget/references/commandes.md.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/docs/decisions/152-reprise-missions-natives.md.

T033 remplace l’ancien échec systématique après panne et le paragraphe de pause
non encore traitée. Elle documente la suspension calculée, la date brute stable,
le délai effectif hors pause, le budget d’attente qui continue, les 300 s d’un
lancement déjà soumis, la borne de deux reprises, les contrôles d’identité et de
droits, le précontrôle global d’annulation et les cinq reçus de refus.

La relecture de `cancel_tree` distingue le précontrôle refusé (zéro arrêt et
zéro marquage de descendant) d’un échec d’arrêt ultérieur. Les confirmations
déjà acquises restent durables dans ce second cas. Les docs ne promettent pas
des arrêts de processus atomiques. La réparation générale d’un marqueur
illisible reste hors152 ; les protections de la flotte sont conservées.

T035 contient exactement D1 à D22, chacune une fois, avec leur réalisation.
Elle inclut les changements nécessaires révélés par les recettes : frontière
Starting réellement durable, reconstruction après quarantaine, transport
`claude_stream_json` attesté, fournisseur de test privé et corrections de
fixture T025. L’ADR garde le statut **Accepté** et distingue décision,
validation ciblée, suite complète, livraison et activation.

Le contrat existant est clarifié sans ajouter de comportement : les formes
d’appel et les canoniques restent compatibles, tandis que les comportements152
changent comme prévu. Le titre du champ `error` inclut aussi `cancelling`,
au lieu de limiter toutes les erreurs à `failed`.
Fichier :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/contracts/reprise152.md.

Le plan reprend les commentaires de complexité réellement présents. Il ne
revendique aucun banc ni optimisation. T034 attend le GREEN R3 et la relecture
finale demandés par le principal. Les observations Python EXPLAIN sur SQLite
3.53.4 ne prouvent ni le plan de la SQLite bundled Rust ni une performance.

### T032 R1/R2 — défauts réels et adaptations de contrat distingués

Les commandes ont été exécutées par le principal. Les logs et JSON sont
relus, sans réexécution par le propriétaire documentaire. Les racines privées
/tmp/b._um08st7 (R1) et /tmp/b.w_kt5i92 (R2) sont en mode 0700.
Chaque capture indique des SHA sources stables pendant ses commandes.

| Capture | Filtre | PASS | FAIL | ignored | RC |
|---|---|---:|---:|---:|---:|
| R1 | `native148` | 27 | 0 | 1 | 0 |
| R1 | `native149` | 60 | 12 | 0 | 101 |
| R1 | `native150` | 47 | 4 | 0 | 101 |
| R1 | `delegation149` | 10 | 0 | 0 | 0 |
| R1 | `spec_087` | 11 | 0 | 0 | 0 |
| R1 | `native151` | 62 | 0 | 0 | 0 |
| R2 | `native149` | 65 | 7 | 0 | 101 |
| R2 | `native150` | 51 | 0 | 0 | 0 |

R1 : **217 PASS/16 FAIL/1 ignored** sur les six filtres, contre les
239 PASS/0 FAIL/1 ignored de la référence. Les 16 FAIL sont nouveaux.
L’échec du lib149 a empêché ses six intégrations ; celles-ci ne sont pas
comptées comme exécutées. R2 rejoue seulement149 et150 : il ne remplace pas
la comparaison complète et n’est pas une suite workspace.

Les cinq F2 stricts qui échouaient en R1 sont PASS en R2 après correction
de production. Leur oracle n’a pas été adapté :

- `native149_f2_descendant_perdu_en_running_est_ferme_puis_la_racine_livre_son_vrai_resultat_une_fois`.
- `native149_f2_execution_d_un_autre_agent_est_refusee_sans_etre_modifiee`.
- `native149_f2_execution_terminale_n_est_jamais_rouverte_au_redemarrage`.
- `native149_f2_proprietaire_hors_ligne_garde_le_resultat_en_attente_puis_livre_une_seule_fois`.
- `native149_f2_reponse_reelle_capturee_survit_au_redemarrage_et_au_descendant_perdu`.

Cause : des descendants déjà `failed/unreachable` gardaient une exécution
active. Ils bloquaient le résultat acquis du parent. Le chemin existant de
clôture exacte est réutilisé ; les contrôles d’autorité et CAS restent stricts.

Les sept échecs149 de R2 se répartissent en cinq adaptations de contrat
autorisées et deux défauts réels de garde. Ces deux cas restent stricts :

- `native149_o4_restart_identities_refuse_toute_incoherence_saga_flotte_marqueur` :
  `may_resume` admettait une tâche sans identité enfant/lien attesté. La source
  finale relue réutilise `restart_identity` et exige l’instance et la commande
  du wrapper géré courant. Elle n’impose pas l’égalité naïve Task g1 = Fleet g2.
- `native149_e2_filiation_contradictoire_ou_incomplete_est_refusee_sans_fermer_l_enfant` :
  `prepare_restart` avalait l’erreur du propriétaire d’un `queued` en
  `failed/unreachable`. La source finale relue sépare `Ok(false)`,
  `Ok(true)` et `Err` : l’erreur nommée est propagée sans changer cette tâche
  ni fermer son enfant.

Repères de lecture :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:63
et
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1524.

Les cinq adaptations149 restent limitées aux cas existants de reprise
validée. Leurs oracles avant/après et leurs FR doivent être consignés à la
preuve finale stable. Aucun contrôle de révocation, sources, identité,
filiation ou état protégé n’est levé. Elles ne sont pas des échecs hérités.

Les quatre adaptations150 ont une correspondance explicite et passent en R2 :

| Cas150 existant | Avant / attendu152 |
|---|---|
| `native150_delai_suivi_aligne_sur_admission_apres_lancement_retarde` | Origine ancienne qui comptait le retard du lancement ; origine152 `mission_issued_at`, délai explicite et omis, FR-010/T045. |
| `native150_delai_suivi_realigne_apres_echec_d_ecriture` | Le suivi déjà aligné n’exerce plus le trigger ; préparer le suivi divergent de l’ancien contrat, conserver le vrai UPDATE refusé puis le succès du retry. |
| `native150_pause_refus_par_le_gestionnaire_complet_ne_fait_rien_avancer` | L’ancien témoin attendait un Start pendant la pause ; le contrôle positif ne l’attend qu’après la levée, FR-008. Le refus ne fait rien avancer. |
| `native150_pause_rejeu_cle_admise_rend_la_tache` | L’égalité stricte inclut désormais le seul `suspended` attendu à `true` pendant la pause, FR-024 ; les autres champs restent comparés. |

La clause existante T032 porte ces correspondances et les deux gardes
strictes. Aucun ID, scénario ou waiver nouveau n’est créé.
R3 est en cours sur les sources finales : T032 reste ouverte jusqu’aux
six filtres GREEN et à la comparaison complète ; T036 couvrira la suite
finale. Les résultats R3 encore partiels ne sont pas un verdict final.

Captures R1/R2 conservées :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r1-results.json — SHA-256 02c82485f1b4e34a3a7465c071829cb427d3fc17c3b1545c6e21d4b5ba9b9173.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r2-results.json — SHA-256 e7bbb6131682e4c3b18aca78ecacb5b8cc589b3413b0c4065b869413d213b409.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r2-native149.log — SHA-256 02544fc38bbbdbe0bfda962cf86199cf3ab8ef54b0d6e7d18b8e834795dee3c7.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r2-native150.log — SHA-256 f2f1e74e49d454ae53de8395b6a01aa3b868b5e8c4f7232c30e118c89bddfa5a.

Tous les logs R1 restent conservés. R2 a duré 24,85 s pour149
(5,79 s de tests), puis 25,22 s pour150 (9,57 s de tests).
Les lignes de résultat à zéro sont conservées mais ne sont pas ajoutées
au nombre de tests réussis. Aucun total global ni couverture n’est inventé.

### Articles XIX/XX — état de la relecture documentaire

Ces réponses décrivent la documentation et les preuves disponibles.
Elles ne ferment ni T034 ni la revue complète T037.

1. **Pourquoi nécessaire ?** Les règles anciennes contredisaient le contrat152.
   La reprise doit garder les identifiants et la filiation. Les gardes refusent
   une identité non attestée et une mutation sur un propriétaire contradictoire.
2. **Pourquoi plus simple ?** Les documents et sections existants sont réutilisés.
   Le code garde la garde unique, les magasins, la chaîne de continuations,
   les CAS et les chemins de reprise existants. Aucun outil, paramètre, module
   Rust, journal autonome ou dépendance nouvelle.
3. **Quelles hypothèses ?** Le journal durable des pauses est lisible.
   Le lien, la flotte et le marqueur natif lisible peuvent attester l’identité
   courante. Les sources de permission sont encore attestées. Une donnée
   corrompue n’est jamais remplacée par une permission supposée.
4. **Qu’est-ce qui est vérifié ?** Relecture des docs, D1–D22, chemins de référence
   existants, sources ciblées et captures ciblées. Le principal a exécuté les
   commandes et les diagnostics DB/EXPLAIN/rustfmt qu’il rapporte. Les E2E
   couvrent de vrais daemons privés avec un fournisseur de test, pas un modèle
   réel. Le propriétaire documentaire n’a exécuté aucun Cargo ou service.
5. **Qu’est-ce qui ne l’est pas ?** Le bilan final R3, la suite complète finale,
   la revue du diff complet, Converge, l’audit, le modèle réel, la performance,
   la couverture/mutation et le runtime de production. Python EXPLAIN n’est
   pas le plan SQLite bundled Rust. Les commentaires de complexité restent
   des analyses, sans mesure.
6. **Qu’a-t-on supprimé ou conservé ?** Les paragraphes de comportement ancien
   sont remplacés. Le principal confirme le retrait du helper de fixture
   `parent_en_cours_bloque_150`, de son booléen et du trigger mort ; le test
   de retry garde son unique vrai UPDATE refusé. Les deux extractions
   `native_child_env` et `classify_stop` ont deux usages réels. Elles gardent
   une même règle d’environnement/permissions et un classement sans effet.
   Le journal, les dates brutes et les reçus sont conservés pour la reprise
   et la compatibilité ; les autres comportements restent hors périmètre.

Sources finales relues à la finition documentaire, avant le verdict R3 :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs — SHA-256 dbd7612d85f27c9ccf4a2a85add97916e018638a663e767c4a1e491df0d47931.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs — SHA-256 94bca662b8488d37d5826b2127fb6b6e585f33655950b720d1a4915194f1641a.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/execution_store.rs — SHA-256 43cacff23bb954c8ed207bc9a1d3bc398c58705623560933cfccb7e9d3c91302.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs — SHA-256 6d2b14e7e04f2ca73213cf4cf66e5cd1c06e49d2f1ba8f5f6852d67a4e0e48e0.

Les captures historiques restent intactes. Le périmètre d’écriture reste
documentaire ; aucun code, Git, Cargo, modèle ou service n’est modifié ou
exécuté par ce propriétaire. Les preuves Git/disque/runtime restent séparées.

## T032 R3 puis R4 — captures distinctes, gates encore ouverts

Le principal exécute les commandes ; le propriétaire documentaire relit
les compteurs et les SHA, sans réexécution. Les captures signalent des
sources stables dans leurs racines privées : /tmp/b.wtn84x1o pour R3,
puis /tmp/b.w6cn9b6q pour R4.

| Capture | Filtre | PASS | FAIL | ignored | RC |
|---|---|---:|---:|---:|---:|
| R3 | `native148` | 27 | 0 | 1 | 0 |
| R3 | `native149` | 71 | 1 | 0 | 101 |
| R3 | `native150` | 51 | 0 | 0 | 0 |
| R3 | `delegation149` | 10 | 0 | 0 | 0 |
| R3 | `spec_087` | 11 | 0 | 0 | 0 |
| R3 | `native151` | 62 | 0 | 0 | 0 |
| R3 | Complément `native152` | 55 | 1 | 0 | 101 |
| R4 | `native149` complet | 78 | 0 | 0 | 0 |
| R4 | `native152 --lib` | 51 | 0 | 0 | 0 |

R3 termine les six filtres de référence à 232 PASS/1 FAIL/1 ignored.
L’échec du lib149 empêche ses six intégrations ; celles-ci ne sont pas
comptées comme exécutées en R3. Le complément152 est distinct :
51 unitaires PASS puis 4 E2E PASS/1 FAIL. Aucun total workspace n’est déduit.

Les sept défauts de production149 corrigés restent PASS. Le dernier
témoin tick149 adapté tentait une remise réelle sans propriétaire connecté
et recevait `routing:agent introuvable`. La fixture existante rétablit les
routes réelles. R4 valide ensuite les 72 unitaires et les six intégrations149
(2 décès de fournisseur et 4 arrêts de wrapper), sans waiver.

Les cinq filtres compatibles de R3, plus149 remplacé par R4, donnent
**239 PASS/0 FAIL/1 ignored**. C’est une agrégation de captures distinctes,
pas une seule commande. T032 attend les sources finales après la correction
du chemin Connected et leur comparaison.

L’E2E152 en échec initialise g2 sans nouveau prompt. Sa racine privée est
/private/tmp/n148-6fefd72eaa. Le diagnostic de l’ordre Register / Fleet
complete précède la correction ; aucun GREEN E2E final n’est déduit de R4,
qui ne lance que les unitaires152.

Captures conservées :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r3-results.json — SHA-256 1df7d426d8e1368eb7030029181a23c24712433dfaa2dfe96fb8c6d264bac366.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r4-results.json — SHA-256 e2b21eac77d49c11b9fc44fbd8d381f7837c3e9644c8677c2a9ad1322ba7642f.

Logs des deux échecs R3 et des reprises R4 :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r3-native149.log — SHA-256 470117ac7323ab594854665e293910aede1407971f7ea12df1ce69f5a0a47a7e.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r3-native152.log — SHA-256 689c38d798ad4c0099bd8bffddd727ab74d87d07451056b762f928409c5970a2.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r4-native149.log — SHA-256 bbeddfec4fc20511d315477a8be0dbbdffaf567315fdbe6898f63847da56f306.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r4-native152.log — SHA-256 7aea06c5d800275e2ceb870902014366a9438ee623da4980b0b682dac1c950a8.

### Compléments des oracles SC-002 et SC-003

La préparation de Converge relève une preuve de durée incomplète :
300 ticks ne prouvent pas 300 s. Le principal autorise le renforcement
des mêmes tests prévus, sans scénario supplémentaire ni changement de
production dans cette tranche.

- SC-002 : journal de pause décalé, durée simulée d’au moins 300 s et zéro effet.
- SC-003 : `reply_timeout` 120 s, pause fermée de 300 s, restant 100 s à ±2 s
  avant, pendant et après ; date brute intacte, exactement deux événements
  et crash avant tick conservés dans le cas `echeance_sans_tick`.

Le principal confirme ces oracles dans les 51 unitaires GREEN R4.
La phrase de T019 est alignée sur la durée contractuelle ; aucune case
supplémentaire ne change.

### Revue de code hostile — décision encore ouverte

Haiku1 rend REQUEST_CHANGES par lecture seule. Ses cinq objections ne sont
pas reproduites par le reviewer. Le principal rejette l’interprétation H1 :
FR-029 vise commande/instance/lien natifs et marqueur, pas une exécution
SQLite étrangère. Les contrôles149 d’autorité stricts existent et passent
en R4 ; ils doivent être conservés. Les quatre autres objections restent
en contre-analyse avant toute décision de correction.

Résumé et contre-décisions :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/adversarial-review-claude.md.
Capture exacte :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/code-review-152-r1.json.

Le compteur reste **46/50 (92 %)**. T032/T034/T036/T037 restent ouvertes.
T034 attend la finalisation et la relecture actuelle des coûts/minimalisme.
Aucun code, Git, Cargo, service ou modèle n’est touché ici.

## Alignement après le correctif terminal — R5 en cours, 46/50 conservé

GO du principal pour les corrections nécessaires au même contrat :
Connected retente la reprise après identité Fleet attestée ; M2/M3/M4
corrigent les erreurs de lecture et l’origine d’échéance. Aucun helper,
scénario, tâche ou exigence nouvelle. Le module de tests N est annoncé
byte-identique pendant ce correctif de production.

Relecture actuelle des sources gelées :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs — SHA-256 c46904d3623faa2c1799893db257b027aa94b460ed32cd5e3f3f72b53da99b6a.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs — SHA-256 2f401b64b0fc6a5b8f85be0a56516cef315b6c32f46a9ed216f00e7273df7685.

Les passages lus sont N58–85, N1285–1312, N1525–1541, N1623–1633,
N1808–1825, N1857–1870, D3096–3135 et D18654–18661. Ce sont des preuves
de code, pas une réexécution. R5 est lancé par le principal ; aucun bilan
GREEN final n’est encore acquis à cette écriture.

- M2 : `rejected(st, conn, mission, reason) -> Result<bool, String>`.
  `Ok(true)` qualifie la pause ; `Ok(false)` poursuit le refus ordinaire.
  `Err` remonte. L’unique appelant journalise puis revient avant les
  mutations du suivi et l’avis au parent. Le tick diffère aussi si la
  qualification de pause est illisible.
- M3 : seuls neuf refus de décision deviennent un échec :
  `native_restart_identity_mismatch`, `unreachable`,
  `mission_reply_timeout`, `native_resume_exhausted`,
  `permission_not_inherited`, `invalid_record`, `cwd_unavailable`,
  `cwd_changed`, `settings_revision_changed`.
  Les autres erreurs sont propagées ; un texte SQL opaque n’est pas
  transformé en refus métier.
- M4 : la date d’admission vient uniquement de
  `get_request(mission).created_at`. Une demande absente ou illisible
  diffère le traitement, sans échéance fabriquée depuis une tâche ou
  une exécution. Le repli de l’enveloppe d’envoi des anciens payloads
  reste distinct de cette origine d’échéance.
- Connected : l’instance courante, sa commande gérée, la connexion
  primaire, la route exacte et l’état connecté/libre sont vérifiés après
  identité Fleet. Le chemin `schedule_execution_recovery` existant est
  retenté ; il ne crée aucun nouveau compteur de reprise.

T013/T044 et leur dépendance décrivent maintenant `Result<bool, String>`.
Le prototype T015 est aligné sur ses quatre arguments. T023 et T045
clarifient les erreurs et l’origine durable, sans changer leurs tests
prévus ou leurs cases. D4/D5/D10/D14/D17/D19 du plan et les décisions
correspondantes de l’ADR sont alignées. La complexité `may_resume` reprend
le commentaire N58, avec flotte JSON/index/chaîne et le coût supplémentaire
des sagas d’une génération décalée ; aucune mesure n’est inventée.

H1 est rejetée dans sa lecture trop large de FR-029. Les gardes F2/F3
de cible étrangère149 restent strictes. M5 reste une réserve LOW :
le précontrôle n’a pas d’effet ; l’échec partiel après celui-ci relève
de FR-020 et des cas déjà prévus. Aucun patch L5 ni waiver.

**46/50 (92 %)** reste inchangé. T032/T034/T036/T037 restent ouvertes.
L’inventaire d’audit préparatoire doit être rafraîchi après les derniers
documents, T037, gel final et Converge. Aucun audit thématique n’est lancé.

## R5 final ciblé, suite R1 et revue R2 — T032/T034 clôturées

Le principal autorise la clôture de T032/T034 après R5 GREEN et relecture.
Seules ces deux cases changent : **48/50 (96 %)**. T036/T037 restent ouvertes.
Le détail requis par T032 est maintenant dans :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/tests-152.md.

R5 : sept filtres, **295 PASS/0 FAIL/1 ignored**, RC0 pour chaque commande.
Les six filtres compatibles donnent **239 PASS/0 FAIL/1 ignored**, exactement
comme la référence. Native152 apporte **51 unitaires +5 E2E PASS**. Les six
intégrations149 sont réellement exécutées. Les JSON, logs, compteurs et SHA
sont relus ; aucun test n'est réexécuté par le propriétaire documentaire.
Capture R5 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r5-results.json.
SHA-256 : 423fd5036659fe9a68826ad60b9899bca930ca2a8457a2c006cf76ec99a0fb63.

La correspondance T032 distingue **quatre oracles149 réellement adaptés**
et le cinquième témoin finalement byte-identique, ligne866. Les cinq F2
stricts et les deux défauts de garde corrigés restent distincts. Les quatre
adaptations150 et la comparaison stricte current_task_view148 sont conservées.
Les 16 FAIL de R1 ne deviennent ni hérités ni waivers.

### T034 — coûts et minimalisme relus

Preuves de code :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/execution_store.rs:1021 — latest_in_chain O(c × (log n + log c)), espace O(c) ; ORDER BY/LIMIT 1 reste O(c).
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/execution_store.rs:1540 — abandon_pause_interruption O(log p).
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:58 — may_resume inclut flotte JSON, tri/index, chaîne et pauses ; génération décalée inclut les sagas.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:9713 — classify_stop sans mutation ni signal, O(1) supervisé ou O(b) marqueur.

Les deux extractions ont deux usages réels : native_child_env dans spawn_task
et la reprise daemon ; classify_stop dans l'arrêt simple et le précontrôle
d'arbre. Le helper de fixture parent_en_cours_bloque_150, le booléen et
l'ancien trigger mort sont retirés. Le vrai UPDATE refusé du retry reste.
Le précontrôle local cancel_tree reste O(m + f + p + n log n), hors lectures
descendants/liens/marqueurs ; sauvegardes cumulées O(n²) octets. Les dates
brutes ne sont pas réécrites. Aucun banc ni optimisation mesurée. Python
EXPLAIN3.53.4 n'est pas la SQLite bundled Rust.

### Suite complète R1 — preuve séparée, pas succès global

**94 cibles extérieures, 2103 PASS/8 FAIL/72 ignored**, RC101, 434,66 s.
La comparaison trouve exactement les mêmes cinq cibles et huit noms que
W0041512046/8/72. La qualité rapporte les cinq fichiers byte-identiques au
socle et les mêmes refus d'identité089/HTTP404098 ; aucune cause nouvelle
n'est inventée. La validation finale doit être rejouée après le patch suivant.
Preuve de comparaison :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/workspace-152-r1-comparison.json.
SHA-256 : ef806aa33540aaee73eaa8d6d75814a59d0e556e53bbc9918b9e4de02effdc83.

### Revue R2 et contre-lecture H1/L5

Capture brute conservée :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/code-review-152-r2.json.
SHA-256 : 1f9c502b2ac7658523af3ab097207c4c17d3d0dcbbbd0f03520fa3a321fb89c4.
Claude Haiku5.5, tâche152-code-review-20261011-r2, état completed.
Summary et latestTerminalSummary sont identiques. Verdict REQUEST_CHANGES.
La revue est une lecture seule ; elle ne reproduit ni tests ni modèle.

M2/M3/M4 sont reconnus résolus dans R2, avec le cas Ok(None) signalé à part.
La limite H1 est maintenant explicite dans spec, plan, contrat, guide et ADR.
FR-029 conserve commande/instance/lien natifs et marqueur lisible ; une
cible d'exécution étrangère, un refus CAS ou une erreur de stockage peuvent
bloquer le démarrage. Les gardes149 ne sont pas assouplies.

La qualité réfute la causalité L5 proposée : close_lost_execution est dans
la boucle métier APRÈS les arrêts et uniquement si success=true. Son continue
ne commande pas l'arrêt d'un parent. La limite vraie est une clôture métier
partielle sur refus CAS après arrêts confirmés. Le précontrôle refusé garde
zéro arrêt et zéro marquage descendant ; FR-020 rejoue seulement les arrêts
non confirmés. Aucun patch L5 ni nouvelle exigence d'atomicité.

### Demande absente — patch posé, R6 encore en diagnostic

Le principal confirme le cas Ok(None) après admission, ou working sans
échéance. Les deux branches maintenant posées appellent close_lost_execution
avec ses gardes AVANT de sauvegarder failed/request_unavailable. Si la
clôture est refusée, elles reportent sans changer la tâche. Une erreur SQL
de lecture reporte également. Le chemin avant admission et le cas Some
restent conservés ; aucune échéance supposée depuis la tâche ou l'exécution.
T045/plan/ADR/reuse-audit décrivent ce comportement, sans nouvel ID ni test.

Code lu :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1623
et
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1820.
SHA observé : 194be156a6873feb4761c332fe03548a72feed033a10d854edc3e5f13f00f0a5.
D reste 2f401b64b0fc6a5b8f85be0a56516cef315b6c32f46a9ed216f00e7273df7685.

R6 après patch : **48 PASS/3 FAIL/0 ignored**, RC101 ; E2E non exécutés.
Les cas sont `native152_annulation_descendant_non_racine_independante`, `native152_rouge_annulation_multi_ticks`, `native152_rouge_annulation_partielle`.
Ils attendent working, mais observent failed. Le diagnostic indépendant
compare fixture sans suivi durable et produit. Aucun oracle adapté ni
correctif supplémentaire n'est acquis. Aucun GREEN après ce patch.
Capture :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r6-results.json.
SHA-256 : 4c36e274e0752e7f0d2cb2f6a5e61bfc51511d14cd5513c03e9c3604870612b2.

### Six réponses Article XX — clôture documentaire ciblée

1. Pourquoi nécessaire ? Aligner les preuves finales ciblées et les documents
   sur le contrat152 ; conserver les refus sûrs plutôt que fabriquer une date.
2. Pourquoi plus simple ? Réutilisation des documents, gardes, CAS, magasins,
   close_lost_execution et chemins d'arrêt/reprise existants. Aucun outil neuf.
3. Quelles hypothèses ? Sources de droits encore attestées, données durables
   lisibles, clôture exacte de l'exécution autorisée. Les erreurs ne sont pas
   transformées en permission ou date supposée.
4. Qu'est-ce qui est vérifié ? Sources ciblées, quatre adaptations149 effectives,
   témoin byte-identique, commentaires/coûts, R5 GREEN et suite R1 avec mêmes
   échecs151 ; R6 compile et atteint trois assertions en échec. Le principal
   exécute les tests ; le propriétaire documentaire lit les preuves seulement.
5. Qu'est-ce qui ne l'est pas ? GREEN après patch None, suite finale, revue du
   diff complet/T037, Converge, audit, modèle réel, performance, couverture,
   mutation et activation. Les E2E ont un fournisseur de test privé.
6. Qu'est-ce qui est supprimé ou conservé ? Quatre oracles149 adaptés et leur
   raison sont explicites ; le cinquième témoin, les cinq F2 et deux gardes
   restent stricts. Pas de nouvelle tâche/scénario, pas de waiver. Les preuves
   historiques sont conservées. Le compte rendu tests-152.md est requis par T032.

L'audit reste une préparation. Ses cinq artefacts doivent être rafraîchis
après les documents finaux, T037, gel et Converge. Aucune analyse thématique,
scoring ou livraison n'est déclarée par cette mise à jour.

### Diagnostic R6 confirmé et contrôles finaux bornés

Le principal et la qualité ont lu les trois bases privées des FAIL R6.
Elles contiennent zéro tracked_request, zéro exécution et des échéances NULL.
Ces fixtures forçaient Working sans admission durable. Aucun défaut
FR-018/FR-019 n'est démontré : les arrêts observés restent [false,false]
et [false,true]. R6 atteint bien le nouveau refus request_unavailable.

La correction autorisée porte seulement sur les données des trois tests :
create_request existant, vraies mission/owner/child/délai. Aucun oracle ou
préfixe produit n'est modifié, aucune échéance artificielle. R7 n'est pas
encore GREEN à cette écriture.

La suite complète prévue est déjà exécutée en R1 sur son propre hash :
2103 PASS/8 FAIL hérités/72 ignored. Après l'unique correctif produit des
deux absences de suivi, les contrôles prévus sont R7 (six filtres compatibles
et native152 toutes cibles), puis098 seul, sensible à la charge, prévu T036.
Une nouvelle répétition globale n'est pas décidée si ces preuves suffisent.
Les runs et empreintes restent séparés ; R1 n'est pas présenté comme exécuté
sur le hash final. T036 se clôture seulement après R7/098 ; T037 après le
diff complet et les Articles XIX/XX. Compteur actuel :48/50.

## Mise à jour courte avant R8 — R3, pause et fixtures

R3 reconnaît H1/M2/M3/M4/L5 résolus mais retient la pause ouverte après
request_unavailable. Correctif posé N1464–1469 : une condition existante
admet cette raison après les gardes cible/CAS ; abandon_pause_interruption
consomme la ligne. Aucun état, continuation, helper ou scénario ajouté.
R7 donne56PASS152,51PASS150,10PASSdelegation149,11PASS087,62PASS151 ;14822/1
et14971/1 ont des Working sans Request. Deux fixtures réutilisent create_request
avec vrais IDs/délai, oracles stricts inchangés. R8 puis098 restent attendus.

Rectification du comptage précédent : le corps O4 adapte aussi le refus global
Err en Ok vide avec tâche failed/mismatch, FR-029. Il y a cinq adaptations149,
plus le témoin reprise_d_execution_ignore… byte-identique. Sa garde de reprise
reste stricte. La liste antérieure de quatre avait omis cette portion O4.
Le compte rendu actuel et T032 sont corrigés ; les historiques sont conservés.

| Question XIX/XX | Réponse ciblée |
|---|---|
| Pourquoi nécessaire ? | Ne pas laisser une pause ouverte rendre récupérable une mission failed/request_unavailable. |
| Pourquoi plus simple ? | Une condition dans le chemin existant, même garde, CAS et abandon_pause_interruption ; aucun nouveau mécanisme. |
| Quelles hypothèses ? | Cible autorisée, données lisibles et clôture contrôlée ; fixtures Working avec admission durable réelle. |
| Qu'est-ce qui est vérifié ? | Code N1464–1469 et hash final, R3 lu, compteurs R7 et diagnostics DB indépendants rapportés au principal. |
| Qu'est-ce qui ne l'est pas ? | GREEN R8/098, revue complète/T037, Converge, audit, performance, couverture/mutation, modèle réel et activation. |
| Qu'a-t-on conservé ou supprimé ? | Gardes SQL/foreign/CAS et oracles conservés ; données de deux fixtures complétées, aucune tâche/scénario nouveau. |

N observé :c19496f3eeba93e1234f04e23a0fc4ff5735cbf9efcfe578fff0520d5a740cf3.
P149 observé :4e10d22f5f52a1b9719a52089c202a44d6c6c60b60c760f9be564d14af6fb2ca.
Preuves : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/tests-152.md.
48/50 inchangé ; aucune caseT036/T037 fermée. Aucun code/test/service exécuté
par le propriétaire documentaire. L'inventaire audit attend Converge et le GO.

## T036 exécutée — R8 final et098 seul, preuves séparées

Le principal autorise T036 après la suite prévue et son rejeu098. **49/50**.
T037 reste ouverte avant le verdict final du reviewer et son GO de cochage.

| Preuve | PASS/FAIL/ignored | RC | Portée |
|---|---|---|---|
| Suite complète R1 | 2103/8/72 | 101 | 94 cibles, avant les correctifs finaux ; mêmes cinq cibles/huit noms que151, pas GREEN global. |
| R8 final | 295/0/1 | 0 pour sept commandes | Six filtres compatibles239/0/1, plus native15256PASS ; sources finales stables. |
| 098 seul prévu | 0/1/0 | 101 | Même test/assertion HTTP404 que151 et R1 ; aucune causalité de charge démontrée. |

R8 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r8-results.json.
SHA-256 : 162ee39e5d29668b25178a6839540b0f6e35d4fbfbdb2d4eadd29d8b4b795038.
098 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/t036-098-r1-results.json.
SHA-256 : 05167a7366bcbc219c04505d2cc3b1077193581b290baf10221ebe3f72d2fae4.
Log098 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/t036-098-r1-spec098-standalone.log.
SHA-256 : 229e31fb20ca04f1d5d9493962c60d14a77d8c94290dce238425746e79d0badf.

R8 termine native14827/0/1, native14978/0/0, native15051/0/0,
delegation14910/0/0, spec_08711/0/0, native15162/0/0, native15256/0/0.
Les intégrations148/149/151/152 sont effectivement exécutées ; la capture
atteste les SHA stables des dix sources. Le replay098 conserve les HTTP404
répétés et son échec `spec098_pont_expose_remet_repond_renouvelle_et_retire`.

La suite R1 n'a pas été réexécutée sur les hashes R8. Aucun total global
final n'est fabriqué. Les huit échecs hérités ont leurs noms, sources
byte-identiques et diagnostics indépendants conservés. Les préoccupations
natives finales sont couvertes par R8 ; aucun nouveau scénario ni répétition
globale supplémentaire demandé. E2E avec fournisseur privé, pas modèle réel.

La lecture complète indépendante T037 est rapportée achevée sans nouveau
blocage par le principal. La consignation du verdict détaillé reste attendue
avant cochage. Converge, audit, livraison et activation sont encore distincts.

### T037 — relecture indépendante du diff et six réponses XIX/XX

Le reviewer a lu les30 fichiers/155 hunks de la capture a81e5df561a63de77766b52be54eb6a2861da572a3811fe350044791732f9046
(701461 octets,10613 lignes,+8473/-681), ainsi que les trois correctifs finaux
et les cinq F2 en contexte. Aucun blocage nouveau ni ligne supprimable
prouvée. Cette capture précède les seuls ajouts documentaires T036 ci-dessus ;
leur relecture finale reste le gate avant cochageT037. Pas de test personnel.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/152-reprise-missions/implementation.md#extrait-3).

Coûts relus : may_resume inclut JSON/tri/liens/chaîne/sagas ; latest_in_chain
O(c(log n+log c)), mémoireO(c) ; pauseO(log N+P) ; collecteD7300
O(R(log t+log r+log N+P)) ; précontrôle annulation localO(m+f+p+n log n),
hors descendants/liens/marqueurs, sauvegardes cumuléesO(n²) octets ; purgeO(refus).
Analyses de code seulement, aucune mesure de performance.
