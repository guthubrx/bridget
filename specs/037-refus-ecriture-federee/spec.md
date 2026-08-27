# Session 037 — Refus d’écriture fédérée

**Branche** : `session-037-refus-ecriture-federee`
**Base mesurée** : `90802b0377741b509f3743c5675544315b6f0f29`
**Statut** : implémentée et vérifiée ; garantie bornée par M1
**Contrat consommé** : `DaemonIdentityRequest` /
`DaemonIdentityReport { host, db_path }`, `bridget_core::local_host()` et
`bridget_core::host_is_attested()`.

## Défaut

Une commande Maicie peut ouvrir `database_path` sur la machine qui l’exécute,
puis dialoguer avec un daemon Bridget joint par une socket fédérée. Elle peut
alors créer ou modifier le registre Maicie local tout en envoyant des messages
au daemon distant : le CLI annonce un succès, mais l’objectif et la délégation
n’existent pas dans le greffe de référence.

Le défaut est structurel, pas limité à `delegate`. Dans
`plugins/maicie/src/main.rs`, `open_store_with_reconciliation` ouvre la base
locale puis lance les réconciliations qui utilisent `bridget_socket`. Ses
appelants sont `status`, `objective`, `delegate`, `profile` et `routine`.
`objective close` recueille en plus des coûts via Bridget avant la clôture
locale. Poser six gardes distinctes laisserait six chemins à oublier.

`plage reserve`, `registre add`, `registre close` et `migrate` écrivent
localement sans parler à Bridget dans la même commande ; ils ne relèvent donc
pas de ce défaut. `preflight` est une lecture seule.

## Propriété

Avant toute ouverture ou écriture de SQLite Maicie dans le chemin commun,
Maicie demande l’identité du daemon sur une **connexion Client préliminaire**
et compare son hôte attesté à l’hôte local attesté. Le prédicat local exige les
deux attestations et leur égalité d’hôte.

- Le même hôte attesté autorise l’ouverture puis le comportement actuel.
- Une machine différente est refusée, quel que soit le chemin de ses bases.
- Une identité absente, invalide ou non négociée est refusée fermée : elle ne
  devient jamais une localisation implicite.
- La sentinelle partagée `bridget_core::HOTE_NON_ATTESTE` n’est pas une
  identité : même présente des deux côtés, elle est refusée comme non attestée.

Le refus est produit avant `open_maicie_store`, avant toute réconciliation et
avant toute création d’objectif, délégation, outbox ou écriture de clôture. Il
dit que le registre appartient au daemon joint et demande d’exécuter la
commande sur sa machine ou d’utiliser l’opération fédérée autorisée.

## Limite résiduelle M1 — attestation préliminaire remplaçable

La garde n’atteste pas les connexions qui produisent les effets ultérieurs :
elle ferme sa connexion préliminaire avant `open_maicie_store`, tandis que les
clients d’effet ne peuvent être créés qu’après l’ouverture du store, car ils
consomment `store.issuer_scope()`. Entre les deux, le listener de la socket peut
être remplacé. Le nouveau daemon peut alors exécuter les échanges de guichet,
coordination, annuaire et remise alors que SQLite a déjà été ouvert localement.

Cette session garantit donc seulement le refus avant SQLite d’un daemon déjà
fédéré **au moment de l’attestation préliminaire**. Elle ne garantit pas que le
daemon attesté est celui des effets. Le témoin dynamique de revue le démontre :
un daemon local est attesté, le socket Unix est remplacé avant l’effet par un
daemon distant, la commande réussit et crée SQLite. Son résultat brut sur la
tête de cette session est `FAILED. 0 passed; 1 failed; 0 ignored; 5 filtered
out` ; ce n’est pas une garantie que cette session prétend lever.

La fenêtre concerne les six ouvertures d’effet inventoriées, puis l’ouverture
supplémentaire de l’annuaire :

- `plugins/maicie/src/reconcile.rs:468` ;
- `plugins/maicie/src/reconcile.rs:601` ;
- `plugins/maicie/src/reconcile.rs:745` ;
- `plugins/maicie/src/reconcile.rs:1065` ;
- `plugins/maicie/src/greffe_service.rs:154` ;
- `plugins/maicie/src/greffe_service.rs:360` ;
- l’annuaire appelé depuis `plugins/maicie/src/main.rs`, dont
  `BridgetClient::list_agents` rouvre une connexion à
  `plugins/maicie/src/bridget_client.rs:744-745`.

Un lot distinct doit introduire une identité qui change à chaque démarrage du
daemon, l’attester sur chaque connexion d’effet et faire exiger cette
provenance par les signatures de réconciliation et de greffe. `BUILD_ID` ne
convient pas : il désigne le commit compilé et deux instances successives du
même binaire partagent sa valeur. Le témoin de remplacement local-vers-local
(même hôte, binaire et chemin de base, daemon redémarré) est l’oracle de levée
de cette limite ; il doit échouer sur cette session et réussir uniquement avec
ce lot distinct.

L’opérateur reçoit un refus explicite dans les trois cas non attestés : daemon
muet ou antérieur, `ClientRejected` et réponse de protocole invalide. Ces cas
ont le même effet métier (aucune écriture), mais restent distingués dans le
diagnostic afin qu’une sonde défectueuse ne ressemble jamais à un daemon local.

## Critère et chemins non comparables

Les chemins absolus ne prouvent pas une machine : deux hôtes Linux peuvent
porter le même `database_path`. Le protocole client observé avant cette session
ne transporte que version, build, horizon, tolérance et capacités ; il ne
fournit aucune identité de machine. Une garde fondée sur le chemin, le nom
d’un agent ou une convention de tunnel serait donc muette face au défaut.

Cette session consomme exclusivement `DaemonIdentityRequest` sur une
`BridgetClient` déjà négociée avec le rôle `Client`, puis attend
`DaemonIdentityReport { host, db_path }` au second aller-retour. Le rôle
`Service` avait initialement rejeté cette requête : une sonde déplacée vers le
guichet aurait alors refusé toutes les écritures tout en ressemblant à une
garde saine. La matrice du protocole a été fermée depuis, mais le témoin reste
sur le chemin `Client` réellement utilisé et toute migration de rôle exige un
nouveau témoin de bout en bout.

Le champ `db_path` est volontairement **hors décision Maicie**. Mesure sur la
référence : il est `DaemonState.db_path`, donc le `bridget.db` du service
(`crates/bridget-daemon/src/daemon.rs:7027-7032`), alors que
`MaicieConfig.database_path` désigne la `maicie.sqlite3` privée et rejette
explicitement le nom `bridget.db` (`plugins/maicie/src/config.rs:333-360`). Les
comparer refuserait une installation locale saine. Ces chemins nomment deux
produits différents ; l’hôte attesté répond seul à la question d’autorité de
cette session.

`daemon_store_is_local` reste privé au daemon. Maicie porte son prédicat
d’hôte à partir de `bridget_core::host_is_attested()` ; `local_host` provient
de `bridget_core::local_host()` afin que l’annuaire, les refus de lancement et
la garde portent le même nom de machine.

Cette session ne modifie ni le daemon, ni le transport, ni le store Maicie, ni
le schéma. Si le contrat ou la source partagée n’est pas disponible au moment
de l’intégration, elle reste bloquée plutôt que d’inventer une heuristique.

## Vérification

1. **Oracle fédéré et d’ordre** : un daemon atteste une machine différente.
   La commande est refusée dans un délai borné et le fichier SQLite Maicie
   cible reste absent : déplacer la garde après l’ouverture tue ce témoin.
2. **Contrôle positif local** : le même hôte attesté franchit la garde. Il
   exclut une implémentation qui refuserait tout.
3. **Contrôle croisé** : hôtes attestés différents, daemon non attesté, puis
   hôte local non attesté sont tous refusés avec un motif explicite.
4. **Oracle de contrat** : identité manquante ou invalide. Le résultat est un
   refus fermé, jamais un succès par défaut ; les diagnostics distinguent
   absence de réponse, rejet de rôle et réponse invalide.
5. **Mutant rejoué après correctif** : retirer le comparateur d’hôte ou
   déplacer la garde après l’ouverture SQLite fait échouer l’oracle fédéré ;
   le contrôle positif local continue d’empêcher une implémentation qui
   refuserait tout.
6. **Limite M1 documentée** : l’oracle permanent de remplacement de socket,
   d’abord local puis distant, reste rouge sur cette session car la provenance
   n’est pas liée structurellement aux connexions d’effet. Il appartient au lot
   d’identité d’instance et ne peut pas valider cette garantie partielle.

Chaque attente du harnais est bornée et signale explicitement son dépassement.
L’univers des tests est listé avant exécution, puis le nombre de résultats doit
être rapproché sans reste.

## Fixtures de contrat adaptées

Les fixtures suivantes simulent désormais le véritable enchaînement `Client`
(identité) puis `Service` (guichet et coordination), avec un hôte local
attesté. Elles n’utilisent pas une réponse décorative : chacune lit la requête
`DaemonIdentityRequest` et renvoie le rapport correspondant à son daemon
témoin.

- `plugins/maicie/tests/integration/cli_delegate.rs` : cas nominal complet et
  oracle fédéré ; le second atteste un hôte différent et vérifie le refus avant
  création de SQLite.
- `plugins/maicie/tests/integration/status_sources.rs` : les scénarios sans
  budget attestent d’abord l’hôte, puis rendent la relève indisponible afin de
  conserver leur état `unavailable`.
- `plugins/maicie/tests/integration/status_benchmark.rs` : les 22 passages de
  la mesure ignorée traversent aussi l’identité ; la mesure explicite est
  restée verte.
- `plugins/maicie/tests/integration/cli_objective.rs` : la première reprise
  d’outbox reçoit une issue `accepted`, terminale mais sans décision métier ;
  les passages suivants ne reçoivent plus cette connexion.
- `plugins/maicie/tests/integration/cli_profile_activation.rs` : la fixture
  ferme son listener après les Services, de sorte que la reprise d’activation
  tardive reste réellement indisponible et que l’outbox demeure `pending`.

`plugins/maicie/tests/integration/coordination_dispatch.rs` n’est pas
modifiée : ses témoins appellent directement les fonctions de réconciliation,
sans passer par l’ouverture commune. `plugins/maicie/tests/integration/guichet_gate.rs`
n’est pas modifiée non plus : ses chemins pertinents possèdent déjà leur daemon
réel. Leur adaptation aurait changé un autre contrat que cet aller-retour.

## Résultats mesurés

Sur la base indiquée en tête, l’univers `cargo test -p maicie -- --list`
contient 413 tests. La campagne complète ferme ce compte à **406 passed**,
**7 ignored**, **0 failed**. Le benchmark ignoré SC-008 a aussi été exécuté
explicitement : 1 passed.

Les mutants ont été rejoués après le correctif : remplacer le prédicat par
`true` fait échouer le contrôle des hôtes distincts ; déplacer la garde après
`open_maicie_store` fait échouer l’oracle d’absence de SQLite. Les deux rouges
portent chacun 0 passed et 1 failed ; le code sain restauré repasse ses sept
oracles (trois de garde, quatre de protocole).

## Hors périmètre

- écrire directement dans le SQLite du daemon depuis une machine fédérée ;
- monter ou partager un fichier SQLite ;
- changer les autorisations du daemon, le protocole ou le schéma sans le lot
  qui en est propriétaire ;
- bloquer les commandes locales qui ne parlent pas à Bridget dans le même
  chemin.
