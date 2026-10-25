# Archives techniques de l’adaptateur T3

Document d’origine : [specs/151-attente-fin-de-tache/reuse-audit.md](../../../../../../specs/151-attente-fin-de-tache/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Droits de l'attente | `can_own_task` + `owner`/`instance` de `Status` | `crates/bridget-daemon/src/daemon/native_delegation.rs:136`, `:506-516` | même refus `task_unavailable`, aucune fuite |
| Attente bloquante | un thread par connexion | `crates/bridget-daemon/src/daemon.rs:4965-4970`, `:4988` | le verrou d'état est relâché pendant l'attente |
| Relâcher le verrou avant le travail long | patron de `handle_with_fact` | `native_delegation.rs:280-320` | même ordre : lecture sous verrou, suite hors verrou |
| Remise du résultat au parent | `send_as`, identifiant `native-result-<task>` | `native_delegation.rs:716-770`, `tick` `:954-1295` | inchangée ; aucun lien avec les attentes |
| États `claimed`, `delivered` | phases de `send_deliveries` | `crates/bridget-daemon/src/idempotency/send_delivery.rs:311`, `:698-768` | lecture de phase livrée par 150-T005 |
| Preuve de remise | `delivery_acked` (150) | `specs/150-socle-delegation/data-model.md` | seul l'accusé de remise la pose ; la purge 150 la lit |
| Regroupement hors attente | famille `Message` du pont, bornes 8 / 32 Kio | `crates/bridget-daemon/src/t3code.rs:50-51`, `:2073-2092`, `:3142-3150` | un test, aucun code |
| Sonde de support avant trame nouvelle | `DaemonIdentityRequest` / `DaemonIdentityReport` | `crates/bridget-daemon/src/daemon.rs:14384`, `crates/bridget-transport/src/protocol.rs:4187` | champ `features` ajouté ; la négociation de capacité de client est écartée (un ancien daemon est muet, research R11) |
| Rejeu d'un envoi | `id` + `issued_at` | `mcp.rs:724-751` | `bridget_send` ne change pas ; l'attente est un outil séparé |
| Passage à `answered` | `mark_answered_in_transaction` | `crates/bridget-daemon/src/store/ledger_requests.rs:665-678` | le résumé de réponse s'écrit dans le même `UPDATE` ; le ledger n'est pas une clé fiable (`:687`, `INSERT OR REPLACE`) |

<a id="extrait-2"></a>

## Extrait 2

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Compteur d'événements | canal de changements de Lineage | `crates/bridget-daemon/src/delegation_lineage.rs:107`, `:161-175` | Non réutilisable : consommateur unique (`change_receiver`), et seulement les changements visibles. Compteur propre justifié (research R3). |
| Abonnement à un événement | `bridget_events` (`once`) | `mcp.rs:2001` | Non réutilisable : fins de tour et fichiers, pas de résultat de tâche. |
| Attente d'un fil T3 | `t3_thread_wait` côté T3 | `research-t3-bridget.md` B1 | Hors dépôt Bridget ; pas un outil Bridget. |
| Table outbox de livraison | `send_deliveries` | `crates/bridget-daemon/src/idempotency.rs:527` | Réutilisée telle quelle ; pas de table `mailbox`. |

<a id="extrait-3"></a>

## Extrait 3

| Source | Regle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` XVIII | complexité documentée | O(k), k au plus 16 ; 64 attentes ; pas de balayage de `send_deliveries` |
| `~/.speckit/constitution.md` XIX | minimalisme | pas de table, pas de module, pas de sucre `wait` sur `bridget_delegate` |
| `docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md` | garde unique par puits autonome | l'attente n'est pas un effet autonome ; aucune garde ajoutée |
| `docs/decisions/007-socle-idempotence.md` | rejeu et accusé explicites | `delivered` lit la phase `acked`, jamais `result_sent` seul |
| mémoire `bridget-tests-umask-077-env-prive` | recette de test | quickstart et gates |
| mémoire `bridget-regroupement-remises-t3` | lots de 8 messages ou 32 Kio, jamais de demande suivie | regroupement hors attente inchangé ; un test fige la famille |

<a id="extrait-4"></a>

## Extrait 4

| Requete | Portee | Resultat |
|---|---|---|
| `grep tool_timeout\|MCP_TOOL_TIMEOUT` | dépôt hors `target` et worktrees | aucune valeur ; seule la spec 149 note que `tool_timeout_sec` est conservé. Vérification en T001. |
| `grep Condvar\|wait_timeout` | `daemon.rs`, `wrapper.rs`, `t3code.rs`, `fleet.rs`, `daemon/` | patrons existants (`fleet.rs:797`, `:1466`) ; aucun compteur d'écritures de tâche |
| `grep mark_answered` | `crates/bridget-daemon/src` | 3 appelants : `send_delivery.rs:748`, `daemon.rs:17778`, `service_events.rs:380` |
| `grep BATCH_BOUND\|batch_family` | `t3code.rs` | regroupement existant, famille `Message` |
| `grep NativeDelegation` | `daemon.rs`, `cli.rs` | répartition `:16422-16427` ; la CLI n'a que `delegate-grant` et `lineage` (aucune commande d'agent à synchroniser) |
| `grep CREATE TABLE.*tracked_requests` | `crates/` | deux définitions : `idempotency.rs:580`, `store.rs:165` |
| lecture de la liste d'outils sûrs | `wrapper.rs:5035-5056` | tableau de 20 noms, à porter à 23 avec les trois outils 151 |
| `grep ALTER TABLE ... ADD COLUMN` | `crates/bridget-daemon/src/store/` | patron de migration gardée (`threads.rs:454`, `project_compat.rs:1935`) |

<a id="extrait-5"></a>

## Extrait 5

Le MCP donne 75 PASS et 1 ignoré, avec EOF mixte inférieur à 1 ms et notification
nonwait vérifiée pendant connexion, authentification et opération. Client :
19 PASS. JSONL : 3 PASS. La relecture RO après correctif n'avait confirmé aucun
défaut. Le full post-EOF révèle ensuite quatre FAIL nouveaux : contenu089,
isolation089, preuve d'identité T3 et message unique E2E151. Le test rouge
ordinaire confirme la perte de réponse sur EOF des trois clients historiques.
Le quatrième échec a une cause distincte dans la fixture E2E : son délai de
notification de 2 s expire avant la capture. Ces premiers GREEN ciblés restent
historiques et ne suffisent pas à prouver la compatibilité des clients finis.

<a id="extrait-6"></a>

## Extrait 6

Le GREEN compatible conserve id2 en 105 ms avec backend confirmé. Le mélange
Wait3600 et Status muet finit en 4504 ms et ferme les deux sockets. MCP donne
76 PASS et 1 ignoré, connecteur 19 PASS. Les trois bancs clients inchangés
donnent contenu 7 PASS, isolation 8 PASS et 1 ignoré, identité T3 1 PASS et
1 ignoré. Les trois fixtures MCP unitaires à entrée finie ont leur forme
originale restaurée. Ces tests prouvent les corrections isolées ; W002 reste
une suite historique en échec.
