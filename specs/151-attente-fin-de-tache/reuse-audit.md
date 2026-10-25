# Audit de reutilisation de l'existant — 151-attente-fin-de-tache

État courant après T027 : Implemented, 27 tâches cochées sur 27.
Le gate de réutilisation reste PASS ; T027 prouve SC-009 par un test distinct
de W004. Le GO des captures r2 est reçu après les18 corrections de style.

## Decision

Statut: PASS
Date: 2026-10-11
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache

Conclusion courte: Le plan étend les modules existants et ne crée ni module de
production ni table. La boîte de fin de tâche est une vue calculée sur la tâche et sur les
phases de `send_deliveries`. L'attente est une lecture pure : elle réutilise
l'identité de `Status`, le thread par connexion, la sonde d'identité du daemon et le
regroupement du pont. Trois créations sont justifiées : le compteur d'événements, la
colonne `reply_summary`, et (US7, supprimable) le champ `acknowledged_at`. Aucune
duplication évidente.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 16 |
| Items audites | 16 |
| Reutilisations deja prevues | 10 |
| Existants potentiellement pertinents | 4 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 7 |

## Reutilisations correctement identifiees

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/151-attente-fin-de-tache/reuse-audit.md#extrait-1).

## Existant potentiellement pertinent non mentionne

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/151-attente-fin-de-tache/reuse-audit.md#extrait-2).

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| aucun | - | - | - |

## Memoires et regles applicables

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/151-attente-fin-de-tache/reuse-audit.md#extrait-3).

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 148-identite-delegation | contrat `bridget_delegate`, `task_view`, clés de statut | clés conservées, ajout seulement |
| 149-sous-agents-lineage | états fermés, projection Lineage, `visible_changed` | `acknowledged_at` invisible de Lineage |
| 150-socle-delegation | `delivery_acked`, lecture de phase, `reply_timeout`, rétention | prérequis ; 151 ne pose jamais `delivery_acked` |
| 114 / 141 (messages groupés) | lots du pont | hors attente, inchangé |
| 129 (remise en cours de tour) | steer pour Claude et Cursor | explique le doublon de FR-030 |
| 132 (attente du lancement) | connexion avec délai propre | même technique pour `wait_timeout + 10 s` |
| 087 (pause) | pause et reprise | l'attente reste permise pendant la pause |

## Journal de recherche

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/151-attente-fin-de-tache/reuse-audit.md#extrait-4).

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Boîte de fin | réutiliser la tâche et `send_deliveries`, vue calculée | trois sources de vérité sinon (research R5) | 2026-10-11 |
| Compteur d'événements | creer nouveau, dans `delegation.rs` | le canal Lineage est à un consommateur (research R3) | 2026-10-11 |
| Réponse d'une demande | creer nouveau : colonne `reply_summary` | aucun lien fiable n'existe ; le ledger est remplaçable ; le balayage des enveloppes est O(n) (research R10) | 2026-10-11 |
| Mode `wait` dans `bridget_delegate` | ne pas faire | deux appels suffisent (research R1) | 2026-10-11 |
| L'attente écrit la livraison ou supprime le message | ne pas faire | contre-revue : réponse préparée n'est pas reçue (research R6) | 2026-10-11 |
| `bridget_send` avec attente | ne pas faire | `id` et `issued_at` seraient cachés pendant l'attente (research R1) | 2026-10-11 |
| Accusé de lecture explicite | creer nouveau, en dernier, supprimable | « lu » exige un acte du parent (research R12, question 2) | 2026-10-11 |
| Doublon d'une réponse d'envoi | accepter et documenter | supprimer la copie touche le routeur (FR-030) | 2026-10-11 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Revalidation dans ce tour (2026-10-11)

Statut : PASS. Un unique Explore en lecture seule a vérifié le plan contre le
socle 150 importé. Les outils d'attente n'écrivent aucun état de livraison.

| Item vérifié | Décision | Preuve actuelle |
|---|---|---|
| phase de remise | REUTILISER `send_delivery_phase` | crates/bridget-daemon/src/idempotency/send_delivery.rs:323 |
| propriétaire | REUTILISER `can_own_task`, jamais `recover_owner` | crates/bridget-daemon/src/daemon/native_delegation.rs:139 |
| compteur multilecteur | CREER `WaitEpoch` dans le module existant ; canal Lineage monolecteur inadéquat | crates/bridget-daemon/src/delegation_lineage.rs:130 |
| mutations hors save | ETENDRE aussi `human_cancel` et `delete_tree` pour réveiller les lecteurs | crates/bridget-daemon/src/delegation_lineage.rs:313 et :223 |
| réponse figée | ETENDRE la transition commune `mark_answered_in_transaction` ; aucune table ajoutée | crates/bridget-daemon/src/store/ledger_requests.rs:679 |
| demande attendue | REUTILISER l'index primaire id et le filtre sender ; les collisions testées sont des IDs de réponses, pas deux demandes du même ID | crates/bridget-daemon/src/store/ledger_requests.rs:251 |
| clôture par délai | REUTILISER `claim_timeout`, notifier après commit en plus des quatre clôtures déjà prévues | crates/bridget-daemon/src/daemon.rs:7014 |
| annulation MCP | CREER le registre dans la façade existante : fermer un socket est une règle de fiabilité, aucune couche nouvelle | crates/bridget-daemon/src/mcp.rs:111 |
| socket avant authentification | REUTILISER `connect_until`, `read_stream` et `authenticate_auxiliary` séparément | crates/bridget-daemon/src/communication/client.rs:939 et :1171 |
| frontière UTF-8 | REUTILISER `floor_char_boundary` pour les deux résumés | crates/bridget-daemon/src/store/ledger_requests.rs:1193 |
| accusé de lecture en lot (US7) | ETENDRE `DelegationStore` par `acknowledge_tasks`, dans son module existant ; une transaction pour toute la liste validée | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/delegation.rs |
| mesure SQLite SC-008 | REUTILISER un seul garde de trace réservé aux tests pour TaskWait et RequestWait ; deux usages réels, contexte stable requis par l'API C | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/daemon.rs |
| accès SQL de fixture | REUTILISER `DelegationStore::test_connection()` ajouté en 151 pour SC-008 ; aucun second accesseur pour l'ACK | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/delegation.rs |
| fin des appels ordinaires sur EOF | ETENDRE le registre MCP existant avec une Condvar et une grâce de 4,5 s ; ses membres et son verrou portent déjà le prédicat. `WaitEpoch` du daemon créerait un couplage métier injustifié | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/mcp.rs |

Aucune nouvelle dépendance, aucun module de production ni aucune table. Les artefacts de
validation sont propres à 151 et suivent les noms déjà employés dans 150.

### Arbitrage FR-038 : connexion enregistrée avant connect

Le scénario EOF mixte Wait + Status a été vu rouge à 10,000712583 secondes,
au-delà de la limite de 6 secondes. Le correctif réutilise le registre MCP et
étend le connecteur commun. `jsonl::connect_nonblocking` reste un wrapper
compatible ; `connect_nonblocking_with` accepte un callback générique.
Le transport n'importe pas le module MCP. Aucun module ni dépendance nouveau
n'est créé pour ce correctif.

Le callback enregistre le socket avant l'appel système connect. Il vérifie
aussi l'annulation pendant poll et après connexion. Le socket enregistré
couvre ensuite l'authentification. Un contexte local au thread accompagne
tous les appels MCP et est retiré par son garde. Le même registre ferme leurs
sockets sur EOF. Une notification d'annulation nonwait conserve l'opération
et masque seulement sa réponse. Aucun nouveau budget, retry ou dial n'est
ajouté ; l'échéance absolue existante reste commune à toutes les phases.

Le RED du callback fait zéro observation avant connect. Le GREEN observe une
fois et interrompt avant dial. Sur macOS, le backlog Unix réel plein rend
`ECONNREFUSED` : la branche `EINPROGRESS`/poll est vérifiée par lecture de code
seulement. Le test ne prouve pas son comportement runtime.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/151-attente-fin-de-tache/reuse-audit.md#extrait-5).

Sources étendues :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-transport/src/jsonl.rs
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/communication/client.rs
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/mcp.rs

RED/GREEN conservés :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-eof-statut151.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-fd-connect151.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-mcp-eof151-final.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-transport-eof151.log

### Arbitrage FR-034/FR-038 : grâce EOF dans le registre existant

Statut : PASS pour la réutilisation. La grâce a ses GREEN ciblés. Le principal
clôt T022 avec « échecs expliqués » et T026, puis autorise le snapshot T023.
Cet owner ne modifie aucune case. Le full final reste obligatoire après ACK.

Fermer tous les sockets dès EOF perd la réponse des clients qui ferment stdin
après leur envoi. Le RED observe un Status authentifié, un backend prêt après
100 ms et une sortie limitée à initialize id1. La réponse id2 manque.
FR-034 impose de conserver cette réponse rapide ; FR-038 borne la fin à 6 s.

Le registre MCP est étendu par une Condvar pour une grâce de 4,5 s des appels
ordinaires. Les attentes sont annulées immédiatement. Les membres du registre
forment déjà le prédicat de fin ; leur retrait et leur notification utilisent
le même verrou. Aucun état parallèle, thread ou polling n'est ajouté.
`WaitEpoch` reste réservé aux faits métier du daemon. Le réutiliser ici
couplerait le cycle de vie du processus MCP au stockage et aux réveils du daemon.
Aucun budget de connexion ou d'opération n'est réduit pendant l'entrée ouverte.
Aucun module, type de registre concurrent ou dépendance nouveau n'est créé.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/151-attente-fin-de-tache/reuse-audit.md#extrait-6).

La recette E2E réutilise son constructeur existant : ancien148 garde 2 s,
les trois151 ont 15 s. La borne SC-001 reste au plus 2 s de capture à réveil.
Le nouveau journal151 donne 3 PASS, 0 FAIL, deux tests filtrés : SC-001 à
945 ms, capture encadrée sur 7 ms, zéro Status ; SC-006 avec un avis unique
et nettoyage durable ; EOF de quatre attentes de 3600 s en 11 ms. Le test148
séparé donne 1 PASS, sans échec. Le délai de recette et la mesure de réveil
restent deux contraintes distinctes.

Preuves relues :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-eof-nonwait151.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/diagnostic-e2e-deadline151.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-mcp-final.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-client.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-core_089_content_test.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-core_089_isolation_test.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-t3_session_identity_env.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-e2e.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-eof-compatible151-e2e148.log

### Arbitrage SC-008 : observateur SQLite réservé aux tests

Statut : PASS pour la réutilisation. Les mesures sont vertes ; les suites
complètes restent historiques en échec. Le snapshot T023 est autorisé.

Le garde RAII `SelectTrace` est dans un module `cfg(test)` du daemon. Ses deux
usages observent les SELECT de TaskWait et RequestWait sur leurs connexions
réelles. Aucun compteur artificiel ni hook de production n'est ajouté.
`sqlite3_trace_v2` impose un callback C et un contexte à adresse stable : le Box
et la désinstallation dans Drop répondent à cette contrainte. Les tests gardent
la connexion vivante jusqu'après le retrait de la trace. Le callback empêche
une panique Rust de traverser la frontière C. Un garde partagé évite deux
copies de ce montage FFI. Aucune dépendance nouvelle n'est nécessaire.

`DelegationStore::test_connection()` est l'unique accès ajouté pour cette
mesure. Il est limité à `cfg(test)` et absent de la référence 150
`1f2c661d39a9ce106723a93ac883f76ab9fe52c6`. L'arbre partagé 151 le contient
après SC-008. Les tests d'ACK le réutilisent ; aucun second accesseur n'est créé.

Les journaux prouvent 2 PASS au calme et 1 PASS sous charge. Ils observent trois
SELECT par attente, autour de 0, 5 et 10 secondes. Sous charge de 64 attentes,
Status prend 2,430 ms et Cancel 19,056 ms avec un écrivain encore actif.

Preuves :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/SC008-151-calme-final.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/SC008-151-charge.log

### Décision et implémentation de l'accusé en lot

Statut : PASS pour l'arbitrage puis l'implémentation de Phase 9. Le RED réel
compile puis donne 0 PASS / 11 FAIL ; le GREEN donne 11 PASS / 0 FAIL.
T025 est clôturée par le principal : 26 tâches cochées sur 26 avant Converge 1.
Les E2E sont verts. W003 reste historique avec
2045 PASS, 9 FAIL et 72 ignorés, RC101 sur 94 cibles. Huit FAIL sont connus150 ;
le neuvième est l'assertion T025 du catalogue MCP, attendu26 contre réel27.
La façade corrige ce seul littéral, de 26 à 27. Le filtre MCP complet donne
79 PASS / 0 FAIL / 1 ignoré ; l'EOF mixte ferme les deux sockets en 4505 ms.
Aucun autre code ne change dans ce correctif. W004 est terminé : 2046 PASS,
8 FAIL et 72 ignorés, RC101 sur 94 cibles. Les huit noms correspondent
exactement à la référence150, sans nouveau FAIL151. La bibliothèque daemon
entière donne 1347 PASS et 14 ignorés.

Preuve :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-mcp-ack-catalogue151.log

La recherche par nom et responsabilité a précédé la nouvelle méthode.
`acknowledge_tasks` n'existait pas. Le seul `acknowledged_at` antérieur était
celui de `delegated_runtime_events`, dans l'idempotence des liens d'agents ;
il ne porte pas la lecture d'une tâche native. Les fixtures150 de tâche,
résultat et remise, ainsi que l'accesseur SC-008, sont réutilisés. La garde
`validate_task_ids` existante porte les bornes et l'unicité.

Le magasin existant `DelegationStore` porte déjà le payload JSON des tâches.
La méthode métier `acknowledge_tasks` l'étend sans module, type ou dépendance
nouvelle. Le handler valide toute la liste de 1 à 16 identifiants sous le verrou
d'état : propriété par `can_own_task`, puis état `result_available` ou `failed`.
Aucune écriture ne précède la validation complète.

La méthode ouvre une seule transaction pour le lot. Elle relit les faits
d'accusé, puis écrit un timestamp commun seulement là où `acknowledged_at`
est absent ou nul. Le premier timestamp reste immuable. Toute erreur SQL annule
le lot. Les listes `acknowledged` et `already` reflètent les faits committés.
La mise à jour JSON réutilise JSON1, déjà utilisé par ce magasin.

`save_projected` est écartée du chemin d'ACK. Elle ouvre et commit une transaction
par tâche : une boucle ne serait pas atomique. Son `stamp` peut aussi remplir
`started_at` ou `completed_at` sur une ancienne tâche. Cette modification peut
alors incrémenter Lineage. L'ACK ne doit modifier ni ces dates, ni `updated_at`,
ni la génération ou la séquence Lineage, ni les faits de livraison ou de purge.

La méthode existante `save_projected` conserve désormais exactement
`acknowledged_at` depuis la ligne durable. Une tâche chargée avant l'ACK, puis
sauvegardée après, ne doit ni effacer ni remplacer le premier timestamp.
Le test vert charge une copie ancienne, accuse la tâche, sauvegarde cette copie,
puis essaie un timestamp différent. Il prouve aussi qu'un objet forgé ne peut
créer un ACK avant la lecture. Les erreurs du deuxième UPDATE et du COMMIT
ont des tests verts de rollback total et d'absence de réveil. Un commit changé
réveille une fois ; un rejeu ne réveille pas. Aucun module, table, helper ou
dépendance n'est créé pour ce lot.

Preuves :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-ack151.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-ack151.log

Source relue pour cette exclusion et cette protection :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/src/delegation_lineage.rs

### Arbitrage du banc des exemples publiés après le premier T022

Le premier passage complet relève neuf JSON dans le skill au lieu des six
attendus par le banc089. La responsabilité du banc reste la communication138.
Le helper `examples(section)` existant est étendu ; aucun helper, fichier ou
framework nouveau n'est créé. La portée commence au titre stable
`### Privilégier le même projet — contrat 138` et comprend les exemples des
chapitres suivants d'envoi, de réponse et de ledger. Les six objets gardent
leur ordre et sont tous exécutés. Aucun exemple inconnu n'est filtré.

Le skill complet doit toujours porter neuf objets ; les trois du contrat151
sont extraits séparément et comparés intégralement. RequestWait publié est
ajouté au scénario089 après réponse connue, avec la clé de demande du parent.
TaskWait publié600 puis Ack publié sont exécutés dans le banc E2E existant.
Leurs paramètres sont lus depuis les exemples publiés. La date privée est
observée avec la connexion SQLite en lecture seule déjà utilisée par SC-001,
sans nouveau helper ni champ public. Le RED précédent de Status était une
assertion hors contrat, pas une régression métier. Le GREEN donne 4 PASS et
1 ignoré, dont trois cas151 et l'ancien148 ; permissions Claude : 4 PASS.
Leur preuve d'exécution est distincte d'une simple validation JSON.
Le principal a exécuté le correctif089 : 1 PASS, 0 FAIL et 1 ignoré, code
retour 0. RequestWait publié est réellement exécuté. La liste fermée Claude
corrigée donne séparément 1 PASS. Aucun Cargo n'est lancé par cet owner.

Preuves E2E publiés :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/red-e2e-ack-champ-public151.log
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/green-e2e-ack151.log

Source réutilisée :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/crates/bridget-daemon/tests/core_089_skill_test.rs

Le principal clôt T022/T026 et donne le GO T023. Les comptes corrigés par dernier
bilan des 94 cibles Cargo sont W001 : 2029 PASS, 10 FAIL, 72 ignorés ; W002 :
2030 PASS, 12 FAIL, 72 ignorés, tous deux au code101. Les comptes bruts
préliminaires restent historiques : W001, 97 bilans, 2032 PASS/10 FAIL ; W002,
98 bilans, 2033 PASS/13 FAIL. Ils comptaient trois PASS de sous-processus en
double, plus le FAIL imbriqué d'identité dans W002. Ce FAIL ne constitue pas
un treizième échec de cible.
Les huit FAIL hérités restent inchangés ; les quatre nouveaux ont maintenant
des GREEN isolés. Cela ne rend pas W002 vert. La relecture indépendante fraîche
ne trouve aucun défaut concret. La compilation normale est sans warning
selon le principal. À cet instant avant ACK, il confirmait 23 cases sur 26,
T023/T024/T025 ouvertes. Il a ensuite vérifié et coché T023, puis T024 après RED.
T025 attendait alors sa clôture. Après W004, le principal la coche : 26/26.
Aucune suite globalement verte n'est annoncée ; huit échecs connus demeurent.
Le prochain full final est obligatoire après ACK ; aucune
nouvelle répétition intermédiaire n'est présentée comme requise. Aucun snapshot
ne précède le GO explicite du principal. Ce GO est maintenant reçu.

### Gate des snapshots finaux après ACK

Recherche en lecture seule des équivalents dans les preuves151 : seuls
`diff-151.patch`, `delta151-vs150.patch` et `snapshot-151-manifest.json`
existent. Ils décrivent le point T023 avant ACK. Aucun artefact portant les
trois noms finaux demandés n'existe à ce contrôle.

Ces fichiers existants sont réutilisés comme références d'étape, sans être
écrasés. Leurs tailles et SHA-256 ont été vérifiés après W003 : mêmes octets
que le reçu T023. Les remplacer effacerait la preuve de revue avant ACK et
changerait les empreintes déjà vérifiées par le principal.

Après GO final seulement, les nouveaux artefacts feront référence à cette
étape et reprendront les mêmes responsabilités : diff cumulatif suivi150+151,
delta151 contre le socle150 reçu, puis manifeste des tailles, SHA et fichiers
non suivis. La comparaison revérifiera les 19 empreintes et le patch importé
du socle. Une divergence live ne sera pas présentée comme une baseline identique.
Les fichiers de sortie seront exclus de leur inventaire pour éviter la récursion.
Aucun script, module ou format de preuve parallèle n'est créé.

Le GO final est reçu après W004. Les sorties finales distinctes sont :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/diff-151-final.patch
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/delta151-vs150-final.patch
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/snapshot-151-final-manifest.json

L'immutabilité distingue les étapes avant et après ACK. Ces trois sorties
complètent les artefacts T023, sans remplacer ni requalifier W001/W002/W003.
Le résultat W003 reste 2045 PASS / 9 FAIL / 72 ignorés, RC101. W004 est terminé ;
le nouvel échec d'oracle151 ne fait pas partie des huit échecs connus150.
Le compte W004 est 2046 PASS / 8 FAIL / 72 ignorés, RC101 sur 94 cibles.
La comparaison indépendante des huit noms avec le relevé150 est exacte.
Les 19 SHA du socle et son patch reçu sont identiques. Le manifeste final
conserve les trois SHA T023 et distingue explicitement les deux étapes.

### Gate du reçu Converge 1 et des captures r2

La recherche des fichiers `*converge*` et `*r2*` dans validation151 ne trouve
aucun reçu équivalent avant création. Le reçu de passage1 archive le verdict
et l'extension de tâches. Il ne remplace ni un test ni les snapshots précédents.
Les captures T023 et finales avant Converge conservent leurs six SHA exacts.

T027 réutilise la fixture, les observateurs et les handlers existants pour
mesurer SC-009 après le vrai DeliverAcked normal. Les tests antérieurs couvraient
séparément les faits ; ce test mesure maintenant la borne après ACK : 1 PASS,
406/357 µs. La barrière open, les résumés stables entre scopes, le compteur
et la capacité sont relus et verts. Aucun hook de production, helper, module,
table ou dépendance n'est ajouté pour cette preuve.

Le principal confirme que seule tasks a changé pendant le gel47 : préfixe
historique de 21424 octets intact, puis une tâche Convergence ajoutée.
Le GO du principal est reçu : les snapshots r2 ont trois noms distincts.
Aucun audit ou Cargo n'est lancé dans ce lot documentaire et de capture.

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/converge151-passage1.md

La correction de format T027 ne change que le newline du while, tokens
identiques. Son bloc final est sans diff ; le ciblé n'est pas rejoué pour
ce newline. Format global :151 RC1 ;150 RC1, 39 fichiers/561 hunks ;151
avant newline568 hunks. Une comparaison des seuls noms ne prouve pas que
tous les écarts sont hérités. La comparaison réelle après newline donne567
hunks. La façade corrige exactement les18 du delta151 dans10 fichiers ;
normalisations rustfmt avant/après byte-identiques, diffcheckRC0. Les547 corps
anciens ne sont pas reformattés. Le final reste RC1/549, pas globalPASS.
Le principal qualifie aussi les2 contextes anciens : lineage_page byte-identique
150, et insert_projected identique après retrait du seul bump déjà formaté.
Leurs tailles et SHA sont tracés dans implementation ; le journal façade
n'est pas réécrit. Les captures r2 reprennent le même format de manifeste et
gardent les six artefacts précédents comme références immuables d'étape.
Le reçu de Converge 1 reste historique : une tâche ajoutée, sans réécriture.
W004 reste2046/8/72 avant T027 ; aucun workspace2047 n'est annoncé.

Sorties r2 après GO, trouvées absentes à la recherche préalable :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/diff-151-final-r2.patch
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/delta151-vs150-final-r2.patch
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/snapshot-151-final-r2-manifest.json
