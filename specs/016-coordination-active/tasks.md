# Tasks 016 — Coordination active : messagers, dépendances et réassignation

**Input** : `spec.md` et `plan.md` gelés au commit `0489be0`, après deux
re-reviews hostiles.
**Statut** : préparation d'exécution uniquement. Aucune tâche de code ne peut
commencer avant G-1600 : la session 015 doit être mergée, gelée et ses fixtures
publiques doivent être épinglées.

## Règles de couloir et de contrat

La sémantique F27/F28/F29 est gelée. Une tâche peut préciser un format, un
refus ou un oracle déjà décidé ; tout changement de seuil, qualification,
priorité de lot ou autorité repasse par revue hostile.

| Lot | Propriétaire | Fichiers exclusifs |
|---|---|---|
| A — Contrat Bridget | codeur A | `crates/bridget-transport/src/protocol.rs`, portions coordination de `crates/bridget-daemon/src/daemon.rs` et `crates/bridget-daemon/src/store.rs`, `crates/bridget-daemon/tests/coordination_events_test.rs`, `specs/016-coordination-active/contracts/evenements-coordination.md`, `specs/016-coordination-active/contracts/fixtures/` |
| B — Domaine et transactions Maicie | codeur B | `plugins/maicie/src/domain.rs`, `plugins/maicie/src/store.rs`, `plugins/maicie/tests/contract/coordination_domain.rs`, `plugins/maicie/tests/integration/coordination_store.rs` |
| C — Application, relève et gate | codeur C | `plugins/maicie/src/config.rs`, `plugins/maicie/src/app.rs`, `plugins/maicie/src/bridget_client.rs`, `plugins/maicie/src/reconcile.rs`, `plugins/maicie/src/main.rs`, `plugins/maicie/tests/contract/coordination_client.rs`, `plugins/maicie/tests/integration/coordination_gate.rs`, `plugins/maicie/tests/fixtures/coordination-active/` |
| Intégration | codeur C après gel A/B | `plugins/maicie/Cargo.toml`, `plugins/maicie/src/lib.rs` uniquement si un branchement final l'exige |

Le lot B possède explicitement dans `plugins/maicie/src/store.rs` la couture
FR-1602 de **tous** les chemins de clôture d'objectif existants vers la
transaction de notifications. Le lot C ne duplique aucune transition. Il peut
préparer tôt son harnais sur les fixtures 015 épinglées, mais il n'importe aucun
DTO 016 avant la gate A.

Chaque commit valide `git diff --check`, ses tests ciblés et `cargo clippy
--all-targets -- -D warnings` sur le périmètre touché. Le workspace complet et
la revue hostile sont réservés à T1614. Chaque lot est relu par un agent autre
que son auteur.

## Phase 1 — Gate d'entrée et corpus amont

- [X] T1601 Vérifier G-1600 et figer dans `specs/016-coordination-active/contracts/amont-015.md` les commits mergés de la session 015, les versions du contrat guichet et les empreintes de toutes ses fixtures publiques ; créer `specs/016-coordination-active/contracts/evenements-coordination.md` et `specs/016-coordination-active/contracts/fixtures/` uniquement depuis ces artefacts gelés. **Observable** : un script de vérification échoue si 015 n'est pas ancêtre de la branche, si son statut n'est pas gelé ou si un octet d'une fixture épinglée diverge ; aucun DTO provisoire n'est accepté.
- [ ] T1602 [P] Préparer côté lot C, sans code de production ni type 016 local, le harnais consommateur dans `plugins/maicie/tests/contract/coordination_client.rs` et `plugins/maicie/tests/fixtures/coordination-active/` à partir des fixtures 015 épinglées : dépôt, fraîcheur, `delivery_report`, `answered`, `timed_out`, `Gap` et `Unavailable`. **Observable** : le harnais relit les bytes amont exacts et porte les résultats attendus des trois incidents, mais refuse de compiler une adaptation 016 tant que la gate A n'a pas publié son corpus.

**Gate G-1600 — entrée non contournable** : T1601 prouve que le contrat 015
est mergé et gelé et que ses fixtures sont pinnées. T1602 peut alors préparer
le harnais C en parallèle, mais aucun fichier de production A, B ou C n'est
modifié avant cette preuve. Si `reminder_sent` manque au contrat 015 final, son
ajout suit T1603 comme extension 016 versionnée ; il n'est jamais déduit d'un
texte ou du ledger.

## Phase 2 — Lot A : événements attestés Bridget

- [ ] T1603 Implémenter dans `crates/bridget-transport/src/protocol.rs` et `crates/bridget-daemon/src/daemon.rs` l'extension versionnée minimale des événements de coordination absents de 015, notamment `reminder_sent`, avec corrélation demande/message/destinataire, instant attesté, génération et canon fermé ; compléter `specs/016-coordination-active/contracts/fixtures/`. **Observable** : round-trip producteur↔consommateur, refus de champ/type/corrélation inconnus et compatibilité des clients 015 ; un texte contenant « relance » ne produit aucun événement (FR-1606, FR-1611, SC-1606).
- [ ] T1604 Persister et exposer dans `crates/bridget-daemon/src/store.rs`, `crates/bridget-daemon/src/daemon.rs` et `crates/bridget-daemon/tests/coordination_events_test.rs` le curseur, la fraîcheur et la déduplication des événements 016 sans lecture directe de base par Maicie. **Observable** : processus réellement tué avant puis après persistance, reprise depuis curseur avec mêmes bytes et même `event_id`, et chaque observation `Gap|Unavailable|non fraîche` reste visible sans être convertie en fait métier (FR-1610, FR-1613, FR-1614).

**Gate G-1601 — contrat Bridget** : producteur A et harnais C relisent le même
corpus versionné ; crash et rejeu ne perdent ni ne doublent un événement. La
gate doit échouer si l'événement attesté est remplacé par un texte libre ou si
une observation incomplète est déclarée fraîche.

## Phase 3 — Lot B : socle transactionnel commun

- [ ] T1605 Définir dans `plugins/maicie/src/domain.rs` et migrer dans `plugins/maicie/src/store.rs` les événements, attentes, arêtes qualifiées, actes de clôture évaluée, politiques épinglées `(version,N,M,chaîne,faits d'appartenance)`, lignées, épisodes de relance, décisions et notification-outboxes. **Observable** : migration historique puis seconde ouverture sans perte ; `N=0`, `M` hors `1..=8`, candidat non déclaré, pilote, doublon ou référence inter-objectifs sont refusés avant mutation ; modifier la configuration courante ne change aucun snapshot existant (FR-1603, FR-1607, FR-1609).
- [ ] T1606 Implémenter le réducteur pur dans `plugins/maicie/src/domain.rs` et l'application transactionnelle commune dans `plugins/maicie/src/store.rs` : déduplication `event_id`, contrôle de génération, décision, transition et outboxes dans une transaction immédiate, sans réseau, horloge, texte, profil, approbation ou spawn. **Observable** : 100 replays du même triplet produisent les mêmes bytes ; une faute entre décision et mutation entraîne rollback total ; les mutations qui consultent le texte, acceptent `Gap` ou séparent transition/outbox rendent `plugins/maicie/tests/contract/coordination_domain.rs` rouge (FR-1611, FR-1612, SC-1605, SC-1606).

**Gate G-1602 — autorité transactionnelle** : B publie les DTO et signatures
du store. C peut alors brancher ses fixtures, sans éditer `domain.rs` ni
`store.rs`. Une mutation retirant le contrôle de génération ou autorisant une
I/O avant commit doit échouer avant les phases utilisateur.

## Phase 4 — User Story 1 : messager de clôture (F27, P1)

**Goal** : notifier une fois chaque destinataire déclaré lorsqu'une clôture
durable rend le message dû, sans prétendre produire cette clôture.

**Test indépendant** : trois destinataires, quatre frontières de crash et
aucun référent actif produisent trois notifications acceptées, jamais six.

- [ ] T1607 [US1] Fermer dans `plugins/maicie/src/store.rs` la couture FR-1602 : toutes les API existantes qui clôturent durablement un objectif passent par une primitive unique qui écrit clôture et notification-outboxes dans la même transaction ; ajouter les oracles dans `plugins/maicie/tests/integration/coordination_store.rs`. **Observable** : 100 clôtures × 3 destinataires donnent 300 clés distinctes ; une mutation qui restaure « clôturer puis notifier » ou contourne un appelant fait échouer un test nommé, tandis qu'un texte « terminé » n'écrit rien (SC-1601).
- [ ] T1608 [US1] Brancher dans `plugins/maicie/src/app.rs`, `plugins/maicie/src/reconcile.rs` et `plugins/maicie/src/bridget_client.rs` le dispatch borné et le replay exact des notification-outboxes F27. **Observable** : crash réel avant commit, après commit avant envoi, après write avant Ack et après Ack avant consommation converge vers les mêmes trois messages et identifiants ; la couverture est annoncée honnêtement comme clôture et ouverture seulement (FR-1601, FR-1602, FR-1613).

## Phase 5 — User Story 2 : déblocage du DAG (F28, P1)

**Goal** : ouvrir une délégation lorsque tous ses prérequis satisfont leur mode
épinglé, puis notifier son affectataire une fois.

**Test indépendant** : un losange terminé dans tous les ordres ouvre chaque
dépendant une fois, sans ouverture prématurée.

- [ ] T1609 [US2] Implémenter dans `plugins/maicie/src/domain.rs` et `plugins/maicie/src/store.rs` le DAG borné, l'index `prérequis → dépendants`, les refus cycle/doublon/inter-objectif/rétroactif et les modes `hash_greffé|clôture_évaluée_exigée`, puis écrire ouverture et notification dans la même transaction. **Observable** : chaîne, losange, 100 nœuds/300 arêtes, concurrence du dernier prérequis, hash seul sous mode strict, annulation et échec couvrent SC-1602 ; muter le sens d'une arête ou retirer l'`ActeClôtureÉvaluée` requis provoque une ouverture prématurée détectée.
- [ ] T1610 [US2] Raccorder dans `plugins/maicie/src/app.rs` et `plugins/maicie/src/reconcile.rs` les événements qualifiants et le dispatch d'ouverture, sans scan d'un autre objectif ni logique DAG dupliquée. **Observable** : le harnais C reçoit exactement une notification après le dernier prérequis, conserve le motif bloquant sinon et ne touche jamais profil/approve/spawn ; deux ordres d'événements rendent le même terminal.

## Phase 6 — User Story 3 : réémission et réassignation (F29, P1)

**Goal** : conserver une demande suivie active sous une borne `M`, puis
arbitrer atomiquement livraison, successeur préautorisé ou intervention
humaine au seuil ou au `timed_out`.

**Test indépendant** : pour chaque classe, `N-1` relances n'ont aucun effet ;
la `N`e ou un `timed_out` sous le seuil produit un seul arbitrage, y compris
avec deux processus et redémarrage.

- [ ] T1611 [US3] Implémenter dans `plugins/maicie/src/store.rs` la normalisation de lot (`delivery_report` avant `reminder_sent|timed_out`), le compteur idempotent, l'inhibition administrative et l'arbitrage transactionnel par génération : annulation source, successeur, nouvelle demande suivie et notifications aux deux participants, ou `intervention_humaine_requise`. **Observable** : deux connexions SQLite et les deux ordres filaires produisent une seule génération ; une livraison gagne toujours dans son lot ; un `timed_out` actif avec moins de `N` relances arbitre, tandis qu'un timeout tardif reste un fait sans autorité. Les mutations « exiger N avant timeout », « oublier l'annulation source » et « ne pas contrôler la génération » font chacune échouer un oracle nommé (FR-1608, FR-1608a, FR-1608b, SC-1603, SC-1604).
- [ ] T1612 [US3] Implémenter dans `plugins/maicie/src/store.rs` et `plugins/maicie/src/domain.rs` la réémission après `answered` terminal : nouvelle demande de même génération et échéance propre tant que `request_ordinal <= M`, puis arbitrage unique à la borne. **Observable** : avec `M=2`, deux `answered` sans `delivery_report` créent exactement deux demandes distinctes et le troisième un seul arbitrage ; muter la borne, réutiliser une demande terminale ou omettre l'outbox dans la transaction rend le test rouge ; aucune génération active ne reste sans demande ni terminal (FR-1606a, SC-1604).
- [ ] T1613 [US3] Valider et brancher dans `plugins/maicie/src/config.rs`, `plugins/maicie/src/app.rs`, `plugins/maicie/src/reconcile.rs` et `plugins/maicie/src/main.rs` les politiques par classe, la relève bornée et les projections d'audit séparant fait local, événement Bridget et fraîcheur. **Observable** : `N`, `M`, version et chaîne sont épinglés avant I/O ; chaîne épuisée, candidat absent, pilote ou observation non fraîche donnent `intervention_humaine_requise`, avec zéro spawn/approbation ; les quatre frontières de crash réelles rejouent les mêmes octets (SC-1603 à SC-1606).

**Gate G-1603 — parcours utilisateur** : F27, F28 et F29 passent séparément
leurs tests indépendants. Les mutations d'oracle couvrent au minimum la
couture FR-1602, le cycle DAG, la fraîcheur, la priorité du rapport, la borne
`M`, l'arbitrage `timed_out`, la génération active et l'interdit FR-014.

## Phase 7 — Gate final, mesures et revue hostile

- [ ] T1614 Intégrer les modules uniquement après gel des interfaces A/B/C, écrire `specs/016-coordination-active/quickstart.md` et consigner dans `specs/016-coordination-active/implementation.md` les preuves SC-1601 à SC-1608 : 300 notifications, corpus DAG, 100 répétitions F29, courses/crashs, refus FR-014, mutations FR-022, trois incidents fondateurs et 20 campagnes de 1 000 événements/300 arêtes sous une seconde. Exécuter `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` puis la revue hostile finale. **Observable** : chaque SC pointe une commande et une sortie reproductibles ; aucun message manuel n'intervient dans le gate SC-1607, le coût reste proportionnel aux événements/arêtes concernés et chaque mutation hostile échoue avec son oracle.

## Dépendances et stratégie

- T1601 → G-1600 → T1603–T1604 → G-1601.
- T1602 démarre après G-1600 en parallèle de A et de B, mais ne consomme les
  DTO 016 qu'après G-1601.
- G-1600 → T1605 → T1606 → G-1602 ; T1607, T1609, T1611 et T1612 restent sous
  le même propriétaire B et sont ordonnés par leurs migrations/transactions.
- G-1601 + G-1602 débloquent T1608, T1610 et T1613 dans le couloir C.
- T1607–T1613 → G-1603 → T1614.

Le MVP est T1601 à T1610 : contrat attesté, transaction commune, messager F27
et DAG F28. F29 (T1611–T1613) ajoute la réassignation, la réémission bornée et
l'arbitrage d'expiration sans élargir FR-014 ni introduire d'horloge Maicie.
