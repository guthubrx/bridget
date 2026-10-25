# Archives techniques de l’adaptateur T3

Document d’origine : [specs/151-attente-fin-de-tache/plan.md](../../../../../../specs/151-attente-fin-de-tache/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Fichier | Changement |
|---|---|
| `crates/bridget-transport/src/protocol.rs` | variantes `Wait` (et `Ack`), trames `RequestWait` / `RequestWaitResult`, champ `features` de `DaemonIdentityReport` |
| `crates/bridget-daemon/src/delegation.rs` | type `WaitEpoch` incrémenté par `save_projected` et `insert_projected` ; (US7) `Task.acknowledged_at` |
| `crates/bridget-daemon/src/delegation_lineage.rs` | appel du compteur dans `save_projected` / `insert_projected` ; (US7) `visible_changed` ignore `acknowledged_at` |
| `crates/bridget-daemon/src/daemon/native_delegation.rs` | dérivation `work_state`, résumé, `mailbox` dans `task_view` ; boucle d'attente hors verrou d'état ; clés du catalogue ; (US7) `Ack` |
| `crates/bridget-daemon/src/daemon.rs` | gestion de `RequestWait` et compteur de 64 attentes ; annonce `features` (`:17844`) ; incréments du compteur aux sites `answered` (`:7672`, `:17778`) et à l'annulation ; listes de trames (`:14384`, `:14615`) |
| `crates/bridget-daemon/src/idempotency.rs` | colonne `reply_summary` dans la création de `tracked_requests` et migration gardée |
| `crates/bridget-daemon/src/store.rs` | même colonne dans la création de `tracked_requests` |
| `crates/bridget-daemon/src/store/ledger_requests.rs` | `mark_answered_in_transaction` écrit `reply_summary` dans le même `UPDATE` |
| `crates/bridget-daemon/src/idempotency/send_delivery.rs` | calcul du résumé depuis le message de réponse et appel de `mark_answered_in_transaction` |
| `crates/bridget-daemon/src/delegation_mcp.rs` | outils `bridget_task_wait` (et `bridget_task_ack`), liste blanche, conversion en `Wait` / `Ack`, connexion avec délai propre, sonde |
| `crates/bridget-daemon/src/mcp.rs` | dispatch (`:475`), outil `bridget_request_wait`, `MAX_WAITING_TOOL_CALLS` et registre d'annulation et `cancel_all` dans `serve` (`:33`, `:120-215`, `:259`), catalogue (`:2597-2625`) |
| `crates/bridget-daemon/src/communication/client.rs` | `registered_connection_until` en `pub(crate)` ; fonction de sonde `daemon_supports_wait` |
| `crates/bridget-daemon/src/wrapper.rs` | `BRIDGET_SAFE_MCP_TOOLS` agrandie (`:5035`) ; `tool_timeout_sec=3700` au montage Codex de Bridget (`:5252-5270`) |
| `crates/bridget-daemon/src/t3code.rs` | un test seulement : famille de message de `native-result-*` |
| tests des mêmes modules, `crates/bridget-daemon/tests/native_delegation_e2e.rs` | tests `native151_*` ; mise à jour des tests qui figent le catalogue (`mcp.rs:2597-2625`, `wrapper.rs:8117`, `:8149`, `:8171`) et les constructions de `DaemonIdentityReport` |
| `docs/delegation-native.md`, `skills/bridget/SKILL.md`, `skills/bridget/references/commandes.md` | outils, états, limites, doublon, délai des hôtes |
| `docs/decisions/151-attente-et-boite-de-fin-de-tache.md` | ADR des décisions R1-R12 |

<a id="extrait-2"></a>

## Extrait 2

| Niveau | Contenu |
|---|---|
| Rouge d'abord | `native151_vue_*`, `native151_boite_*`, `native151_attente_*`, `native151_facade_*`, `native151_envoi_*` écrits et vus en échec avant le code |
| Unitaire | table `work_state` ; résumé (0, 2047, 2048, 2049 octets, caractère de 3 octets coupé) ; états de boîte (table de vérité R5) ; compteur (incrément, réveil, expiration) |
| Attente | retour immédiat si déjà terminé ; retour sur événement ; expiration à 1 s sans annulation ni écriture ; propriétaire ; refus de plage ; 65e attente ; client fermé ; arrêt du daemon ; résultat publié entre deux lectures (100 répétitions) ; attente permise pendant la pause |
| Lecture pure | avant/après une attente : `result_sent`, `failure_sent`, `delivery_acked`, `cleanup_done` identiques ; le message part une fois, attente ouverte, expirée ou fermée ; résultat publié, remise en vol, remise indéterminée |
| Façade | 4 attentes ouvertes, la 5e refusée, `bridget_task_cancel` aboutit ; annulation MCP : socket fermé et créneau libéré |
| Charge | 64 attentes, écritures de tâches étrangères continues, appels de contrôle sous 500 ms |
| Regroupement | trois tâches terminées ensemble : une réponse ; famille de message du pont |
| Envoi | `answered`, `timed_out`, `open`, `cancelled`, `reply_unavailable` (guichet, ligne ancienne), deux scopes même identifiant et même destinataire, remplacement du ledger sans effet, rappel après coupure, émetteur seul |
| Compatibilité | décodage d'un `DaemonIdentityReport` sans `features` ; client 151 contre le vrai daemon de la base 150 : `wait_unsupported` sous 5 s |
| Bout en bout | parent et enfant synthétiques, `crates/bridget-daemon/tests/native_delegation_e2e.rs` |
| Non-régression | filtres `native148`, `native149`, `native150`, `delegation149`, `spec_087`, tests du pont T3 |
| Suite complète | recette `--workspace --no-fail-fast` avant clôture |
