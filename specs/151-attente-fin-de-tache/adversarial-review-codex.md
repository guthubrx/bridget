# Contre-revue adverse - Session151

## Ronde 1 (plan)

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, lecture seule
- **Question** : sûreté de l'attente, de la boîte et de l'attente d'envoi.
- **Verdict reçu** : BLOCKED (8 points)
- **Vérification** : chaque point relu dans le checkout principal
  (`/Users/moi/Nextcloud/10.Scripts/64.bridget`, base `1f2c661d`). Aucun code, test,
  build ni commit.

### Piste de simplification évaluée en priorité (Art. XIX)

Question : `bridget_task_wait` peut-il devenir une lecture pure (jamais « lu », jamais
`delivery_acked`, jamais de suppression du message, jamais de nettoyage) ?

- **Décision : oui, adoptée.** `bridget_task_wait`, `bridget_request_wait` et
  `bridget_task_status` n'écrivent rien de ce qui touche la livraison.
- **Pourquoi** : le seul fait que le daemon puisse prouver est l'accusé de la remise
  par message (phase `acked`). Une réponse d'attente est écrite sur une connexion
  après le retour du gestionnaire (`crates/bridget-daemon/src/daemon.rs:5164-5173`) :
  « préparée » n'est pas « reçue ». Une suppression fondée sur l'écriture de la
  réponse peut perdre un résultat (points 1, 2, 6).
- **Coût assumé** : un doublon possible (retour de l'attente, puis message). Jamais zéro
  fois. Documenté (spec FR-021, question 3).
- **Supprimé** : `result_read_at`, table `task_waits`, branche de `tick`,
  `include_result`, tests de suppression, « transition de remise partagée ».
- **Gardé en option** : un accusé explicite du parent `bridget_task_ack` (P2, dernier,
  supprimable), informatif. Il ne supprime aucun message et ne prouve aucune remise.
  La suppression du doublon, si elle revient un jour, devra reposer sur lui.

### Sort de chaque objection

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| 1 | Réponse préparée n'est pas reçue : T012 marquait lu et `delivery_acked` avant l'écriture ; une coupure supprimait le message de secours et autorisait la purge 150 | lu `daemon.rs:5164-5173` (la réponse est écrite après `handle_wrapper_message`) et `tasks.md` de la première version (T012 écrivait sous verrou) | oui | fermée par construction : l'attente n'écrit plus rien (FR-004, FR-019) ; test `native151_attente_lecture_pure` |
| 2 | `notifications/cancelled` ne ferme pas l'attente ; le daemon pouvait consommer un résultat jamais reçu | lu `mcp.rs:147-158` (annulation enregistrée seulement si l'appel est en vol), `:185-205` (le fil d'exécution termine, puis jette sa réponse) | oui | la partie « consommer » est fermée par la lecture pure. La partie ressource reste réelle : une attente d'une heure resterait ouverte. Annulation transmise : socket daemon fermé (FR-013, R8, T012) ; test |
| 3 | `reply_message_id` ne garantit pas le bon texte : ledger `(id, target)` + `INSERT OR REPLACE`, identifiants uniques par scope | lu `store/ledger_requests.rs:687` (`INSERT OR REPLACE`), `daemon.rs:8636` (clé `IdempotencyKey::new(scope, Send, id)`), test existant à deux scopes avec le même identifiant (`daemon.rs:23320-23335`) | oui | `reply_message_id` supprimé. Résumé figé dans le `UPDATE ... SET state='answered'` (FR-028, R10, T004) ; test de deux scopes, même identifiant, même destinataire, et de remplacement du ledger |
| 4 | `answered` peut exister sans réponse (guichet, lignes migrées) alors que le contrat exigeait `reply` | lu `store/service_events.rs:380` (marque `answered` sans message de réponse) ; la colonne est nouvelle, donc toute ligne antérieure n'a pas de résumé ; branche `envelope_missing` de `send_delivery.rs:741-768` | oui | `answered` avec `reply: null` et `reply_reason: "reply_unavailable"` (FR-028, contrat) ; tests dans T015 |
| 5 | Un ancien daemon ignore sans répondre une trame non décodable (variante `Wait`, capacité `RequestWaitV1`) : attente longue | lu `daemon.rs:5140-5156` (`Err(_)` : journal, `continue`, réponse seulement pour une trame du guichet), `protocol.rs:187-190` (`ClientCapability` énumération fermée) | oui | négociation de capacité supprimée. Sonde par `DaemonIdentityRequest` (trame ancienne), champ `features` ajouté à `DaemonIdentityReport`, budget 3 s, `wait_unsupported` (FR-031, R11, T005) ; test avec le vrai binaire de la base 150 (T018) |
| 6 | « Jamais envoyé » impossible après remise engagée : `tick` peut envoyer avant le second appel d'attente | lu `native_delegation.rs:313-319` (`tick` après `Delegate`/`Cancel`) et envoi du résultat dans le même passage que `result_available` (`:1190-1230`) | oui | fermée par construction : aucun message n'est supprimé, l'attente ne connaît pas `tick` (FR-020) ; tests résultat publié, remise en vol, remise indéterminée dans `native151_attente_lecture_pure` |
| 7 | `bridget_send` avec attente : `id` et `issued_at` générés par le MCP restent cachés ; après coupure, un nouvel appel crée un second envoi | lu `mcp.rs:725-740` (identifiant généré si absent, rendu seulement au retour) | oui | `bridget_send` ne change pas. L'attente est l'outil séparé `bridget_request_wait(request_id)`. Imposer `id` et `issued_at` au parent est écarté : `issued_at` doit rester proche de l'heure du daemon (R1). Test : rappel après coupure, aucun envoi |
| 8 | La borne 64 ne prouve pas l'absence de famine : verrou global partagé ; 8 attentes occupent tous les créneaux de la façade | lu `mcp.rs:33` (`MAX_IN_FLIGHT_TOOL_CALLS = 8` par processus), `:259` (`busy` au neuvième appel), `native_delegation.rs:959` (`tick` garde le verrou d'état pendant toute sa boucle) | oui | 4 attentes au plus par processus, créneaux de contrôle gardés (FR-012, T012). Curseur relu avant chaque lecture, verrou du compteur jamais tenu pendant l'accès à l'état, ordre état puis compteur écrit (FR-005, R3). Test de progression à 64 attentes avec écritures étrangères et appels de contrôle sous 500 ms (SC-008) |

Retenues : 8. Rejetées : 0. Sans objet : 0.
Fermées par construction (lecture pure) : 1, 6, et la partie « consommation » de 2.
Corrections ciblées : 2 (ressource), 3, 4, 5, 7, 8.

### Impact sur 153

La session 153 comptait sur une « transition de remise partagée » écrite par l'attente.
Fichiers lus : `specs/153-rondes-contexte/{spec,plan,data-model,tasks,research}.md`.
Points à corriger dans 153 (je n'ai pas modifié ces fichiers) :

| Fichier de 153 | Passage | Correction attendue |
|---|---|---|
| `spec.md` FR-009 | « quel que soit le chemin : envoi du message par `tick` ou lecture complète par l'attente de SPEC-151 » | seul `tick` pose `result_sent` donc la garde |
| `spec.md` FR-033 | « Une écriture de lecture de l'attente (SPEC-151) ne vaut remise que pour la ronde lue » | supprimer : l'attente n'écrit rien |
| `plan.md` I6 | « fonction unique `mark_result_sent` utilisée par `tick` et par l'attente 151 » | `tick` seul |
| `data-model.md` « Pose de la garde » | « `tick` et l'attente de SPEC-151 (lecture complète) l'appellent » | `tick` seul ; retirer « une écriture de l'attente porte la ronde lue » |
| `tasks.md` T030 et sa dépendance « T030 dépend de T012 de 151 » | pose partagée de la garde dans l'écriture de lecture de l'attente | supprimer T030 et la ligne de dépendance (T012 de 151 n'écrit plus rien) |
| `plan.md` risque « Attente 151 sur une tâche inactive rend aussitôt le résultat de la ronde précédente » ; `tasks.md` T018 | ajout de `round` aux entrées d'attente | conservé : `round` dans les vues reste utile ; l'attente de 151 est une lecture, donc le risque reste vrai |

Conséquence de comportement pour 153 : `continuable` exige `result_sent` posé par `tick`
(`data-model.md` ligne 43 de 153). Un parent qui lit le résumé par l'attente et appelle
`bridget_task_continue` dans la même seconde peut voir un refus tant que `tick`
(cadence 1 s, `daemon.rs:4695-4705`) n'a pas envoyé le message. 153 doit rendre ce refus
réessayable. Le comportement sans attente est inchangé. La garde de 153 n'est jamais
posée par une lecture : c'est plus sûr (l'enfant n'est pas gardé pour un résultat non
remis). L'accusé explicite de 151 (`acknowledged_at`) ne modifie pas la garde.

### Statut final du plan

Les huit objections sont traitées dans les artefacts. Restent ouverts : les
vérifications de T001 (délais d'outil des hôtes, inactivité des connexions, tests qui
figent les clés, nom de la lecture de phase de 150, listes de trames), la correction de
la spec 153, et une seconde contre-revue souhaitable sur l'ordre des verrous et la
fermeture du socket à l'annulation MCP.

## Ronde 2 (plan révisé)

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, lecture seule
- **Verdict reçu** : objections 1, 3, 4, 5, 6, 7 fermées ; 2 partiellement ; 8 sous
  réserve du test SC-008. L'ordre état puis `WaitEpoch` et la relecture du curseur sont
  validés. Quatre points restants, vérifiés dans le checkout principal.

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| R2-1 | Annulation avant enregistrement du socket : la notification peut précéder l'exécuteur ; l'exécution continue | lu `mcp.rs:147-158` (le drapeau n'est posé que si l'appel est en vol, aucun socket n'existe encore à ce moment), `:185-205` (le fil lancé exécute puis jette sa réponse) ; ma version enregistrait le socket après les échanges bloquants | oui | registre unique `{annulé, socket}` sous un verrou ; enregistrement dès la connexion, avant l'authentification et la sonde ; vérification du drapeau dans la même section critique (FR-013, R8, T026) ; tests annulation avant, pendant, après |
| R2-2 | EOF de l'entrée MCP : la boucle sort puis `thread::scope` attend les exécutions ; processus et sockets gardés jusqu'à 1 h | lu `mcp.rs:131-134` (`break` sur `read_line == 0`, `?` sur erreur), `:213` (fin de portée qui joint les fils) | oui | garde `Drop` dans la fermeture de la portée : `cancel_all` (drapeau global, drapeaux, `shutdown` des sockets) avant la jonction ; un enregistrement tardif voit le drapeau global (FR-038, SC-012, T026) ; test avec deux appels bloqués |
| R2-3 | Sommeil fixe de 5 s : une attente de 1 s dure jusqu'à 5 s | lu `tasks.md` de la version révisée (T003, T011 : `wait_changed(seen, 5 s)`) | oui | `min(5 s, délai restant)` ; pour une demande, aussi borné par `deadline_at` ; tranche paramétrable pour les tests (FR-007, SC-013, T003, T011, T016) |
| R2-4 | La réponse du guichet n'incrémente pas `WaitEpoch` | lu `store/service_events.rs:380` (`mark_answered_in_transaction` dans la transaction du `Store`, aucun accès au compteur) et le site d'appel `daemon.rs:16074` | oui | incrément après le commit, au site d'appel (FR-039, R10, T016) ; test `native151_envoi_guichet_reveil_sans_filet` avec un filet de 60 s ; T001(g) liste tous les écrivains de `tracked_requests` |

Retenues : 4. Rejetées : 0. Sans objet : 0.
