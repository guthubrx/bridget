# Plan - Session151 attente et boîte de fin de tâche

**Spec** : `specs/151-attente-fin-de-tache/spec.md`
**Branche** : `session-151-attente-fin-de-tache` (worktree dédié, socle 150 copié sans commit)
**Worktree** : `.worktrees/151-attente-fin-de-tache`
**Date** : 2026-10-11 (révisé après contre-revue Codex, verdict BLOCKED corrigé)

## Résumé

Six changements dans le moteur de délégation native et le client MCP. L'attente
est une lecture pure : elle n'écrit rien de ce qui touche la livraison.

1. `bridget_task_wait` : attente côté daemon, pilotée par un compteur d'événements
   relu avant chaque lecture (R1-R4) ;
2. résultat structuré : `work_state`, résumé stable, taille, référence au détail (R7) ;
3. boîte de fin de tâche calculée sur des faits existants, sans champ ni table (R5) ;
4. regroupement : une réponse d'attente pour plusieurs tâches, test de la famille de
   message du pont ;
5. `bridget_request_wait` : attente de la réponse d'un `bridget_send`, résumé figé
   dans `tracked_requests.reply_summary` (R10) ;
6. façade MCP : 4 attentes au plus, annulation qui ferme le socket (R8) ; sonde de
   support par `DaemonIdentityReport.features` (R11) ;
7. (P2, dernier) `bridget_task_ack` : accusé de lecture explicite, informatif (R12).

Aucun nouveau module, aucune nouvelle dépendance, aucune table nouvelle. `tick`,
l'envoi du message de résultat et la purge 150 ne changent pas.

## Contexte technique

- Langage : Rust 2024, workspace à 3 crates. Travail dans `crates/bridget-daemon`
  et `crates/bridget-transport`.
- Stockage : SQLite, `bridget.db`. Une colonne ajoutée sur `tracked_requests`. Un
  champ optionnel dans le JSON de la tâche (US7 seulement).
- Contrats touchés : `NativeDelegationRequest` (variantes `Wait`, `Ack`),
  `WrapperToDaemon::RequestWait`, `DaemonToWrapper::RequestWaitResult`,
  `DaemonIdentityReport.features`, catalogue MCP, `task_view`.
- Tests : recette `umask 077` (`quickstart.md`).

## Fichiers touchés

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/151-attente-fin-de-tache/plan.md#extrait-1).

Aucun changement dans `tick` (`native_delegation.rs:954-1295`) ni dans l'envoi du
message de résultat.

## Contrôle constitution

| Article | Statut | Justification |
|---|---|---|
| III SpecKit | OK | spec, plan, tâches avant code |
| VII ADR | OK | une ADR prévue (T020) |
| VIII Forensic | OK | faits à vérifier en T001 avant de coder (délais d'hôtes, inactivité des connexions, tests qui figent les clés, constructions de `DaemonIdentityReport`) |
| XV Test-before-next | Adapté | le pipeline interdit le commit automatique ; chaque tâche est testée avant d'être cochée |
| XVI Worktree | OK | `.worktrees/151-attente-fin-de-tache` |
| XVIII Complexité | OK | voir ci-dessous |
| XIX Minimalisme | OK | voir ci-dessous |
| XX Vertus LLM | OK | voir ci-dessous |

### Complexité (Article XVIII)

- Attente : réveil O(k), k au plus 16 lectures indexées ; au plus 64 attentes ;
  une lecture de la base au plus toutes les 5 s sans événement.
- `task_view` : O(1) pour les états, O(2048) pour le résumé, une lecture de phase
  indexée par `(issuer_scope, idempotency_key)`.
- Réponse d'attente : O(k) tâches, k au plus 16.
- Résumé de réponse : calculé une fois à l'écriture de `answered`, lu en O(1).
- Aucun balayage de `send_deliveries`, aucune boucle imbriquée sur la même collection.

### Minimalisme (Article XIX)

- Pas de table, pas de module, pas de champ de livraison. Une colonne nullable, un
  type de 25 lignes dans un module existant. Le champ `acknowledged_at` n'existe que
  pour US7, dernière et supprimable.
- Piste de simplification de la contre-revue appliquée : l'attente est une lecture
  pure. Supprimés : `result_read_at`, `task_waits`, branche de `tick`,
  `include_result`, négociation de capacité de client, fusion de l'attente dans
  `bridget_send`.
- Réutilisés : `can_own_task`, phases de `send_deliveries`, famille de message du pont,
  sonde d'identité du daemon, patron de migration gardée, connexion à délai propre de la
  session 132.
- Rejetés : mode `wait` dans `bridget_delegate`, table outbox, politique de réveil,
  suppression du message de résultat, suppression du doublon d'injection d'une réponse,
  liste des boîtes (session158).
- Potentiel minimalisme : aucune ligne existante supprimable ; ajout net estimé à 300
  lignes de code et 450 lignes de tests.

### Responsabilité future (Article XX)

- Chaque décision a sa preuve `fichier:ligne` dans `research.md`.
- Les faits non prouvés sont des tâches avant le code (T001), pas des affirmations.
- L'auteur humain peut expliquer l'invariant central : une attente ne change jamais
  l'état de livraison ; seul l'accusé de la remise le fait.
- Charge future : réduit les relectures de statut par les agents ; ajoute un compteur
  global dont chaque écriture de tâche dépend ; le doublon possible est documenté.

## Stratégie de test

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/151-attente-fin-de-tache/plan.md#extrait-2).

## Risques

| Risque | Parade |
|---|---|
| L'hôte coupe l'appel avant `wait_timeout` (Codex : 60 s par défaut) | l'attente est une lecture, on la rappelle ; `tool_timeout_sec=3700` dans le montage Bridget ; documentation ; question 1 |
| Doublon : le parent voit le résultat deux fois | assumé et documenté (FR-021) ; jamais zéro fois |
| Attente d'une heure laissée par une annulation MCP ou par l'EOF de l'hôte | registre `{annulé, socket}` sous un verrou, socket enregistré avant l'authentification, `cancel_all` avant la jonction des fils ; tests avant/pendant/après et EOF (T026) |
| Huit attentes bloquent les appels de contrôle de la façade | 4 attentes au plus par processus ; test |
| Chaque écriture de tâche réveille 64 attentes | lectures indexées bornées à 16 ; ordre des verrous écrit ; test de charge SC-008 |
| Ancien daemon muet face à une trame inconnue | sonde par `DaemonIdentityRequest` avant toute trame nouvelle ; test avec le vrai binaire ancien |
| Mauvaise réponse retournée par la clé du ledger | résumé figé dans la transaction `answered` ; test de deux scopes |
| Un test fige les clés exactes de `task_view`, du catalogue ou de `DaemonIdentityReport` | T001 les liste ; ils sont mis à jour dans la tâche qui change la clé |
| Dépendance au nom de la lecture de phase de 150 (T005) | T001 relève le nom exact dans le worktree 150 livré |
| Colonne ajoutée à deux définitions de `tracked_requests` | migration gardée par `pragma table_info` ; test d'une ancienne base |
| 153 s'appuyait sur une écriture de lecture de l'attente | 153 doit retirer ce couplage (voir contre-revue, « Impact sur 153 ») |

## Livraison

Code sur disque dans le worktree, sans commit automatique ni relance de service.
Diff cumulé enregistré dans `validation/diff-151.patch`. Les outils nouveaux
deviennent disponibles au chargement des nouveaux processus MCP.

## Corrections ciblées après contre-revue Claude (ce tour)

- Le gestionnaire de connexion transmet le socket au chemin d'attente, avec
  `recv(MSG_PEEK | MSG_DONTWAIT)` : ni consommation ni changement de `O_NONBLOCK`.
  La compatibilité des appels internes de test est conservée sans nouvelle table.
- `task_view` reçoit la phase issue de la lecture indexée ; phase orpheline ou
  absente après envoi = `claimed`, jamais `delivered`.
- Le snapshot d'une demande rend état, échéance et résumé dans un seul SELECT
  filtré par l'émetteur. Une ligne open échue est rendue timed_out sans écriture.
- La fonction commune de réponse et son wrapper `Store::mark_answered` sont
  étendus ensemble. Le premier auteur de la clôture fixe le résumé : une réponse
  tardive après le guichet conserve `reply_unavailable`.
- Le socket de sonde est enregistré puis remplacé par celui de l'attente dans
  le même registre. Le contexte du fil est effacé par garde Drop.
- Le refus de capacité de façade précède son exécution ; l'ordre des refus
  documenté côté daemon reste distinct. RequestWait : Client autorisé, Service refusé.
