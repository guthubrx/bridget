# Audit de réutilisation de l'existant - Spec 307

**Statut** : `APPROVED`
**Date** : 2026-11-21

## Inventaire des éléments du plan vs existant

| Élément proposé | Recherche effectuée (`rg`) | Existant trouvé (`fichier:ligne`) | Décision | Justification |
|---|---|---|---|---|
| Filtre des processus externes `cwd_unavailable_enoent` | `rg -n "cwd_unavailable_enoent\|drop_session_helpers" scripts/build.py` | `scripts/build.py:640` (`drop_session_helpers`) | `REUTILISER` | Même pattern de filtrage de blocs `lsof -F` dans `busy_paths()`, sans casser `process_output_complete` |
| Garde `client_probe_block_generation` | `rg -n "client_probe_block_generation" scripts/update.py` | `scripts/update.py:545,641` | `REUTILISER` | Ajustement en place dans `Updater.step()` sans nouvelle variable ni fichier d'état |
| Purge des reçus `auto_admission_receipts` | `rg -n "purge_expired_auto_admission\|purge_auto_refusals_in" crates/bridget-daemon/src/delegation.rs` | `crates/bridget-daemon/src/delegation.rs:763,1950` | `REUTILISER` | Étendre `DelegationStore` et `maintain_retention` au même endroit, même contrat transactionnel |
| Chargement groupé des noms d'affichage | `rg -n "agent_display_name" crates/bridget-daemon/src/` | `crates/bridget-daemon/src/store/ledger_requests.rs:93` | `REUTILISER` | Ajouter `agent_display_names` sur `Store` à côté de `agent_display_name` |
| Filtrage CLI `bridget who --all` | `rg -n "fn parse_directory_args\|fn cmd_who" crates/bridget-daemon/src/cli.rs` | `crates/bridget-daemon/src/cli.rs:6838,7700` | `REUTILISER` | Étendre `DirectoryArgs` et `cmd_who` sans toucher au protocole |
| Offset initial `IncrementalJournalReader` | `rg -n "struct IncrementalJournalReader" crates/bridget-transport/src/journal.rs` | `crates/bridget-transport/src/journal.rs:732` | `REUTILISER` | Ajouter un constructeur `from_offset` sur le lecteur existant |
| Persistance `journal_offsets` dans T3 | `rg -n "struct ThreadState\|seen_activities" crates/bridget-daemon/src/t3code.rs` | `crates/bridget-daemon/src/t3code.rs:262,3910` | `REUTILISER` | Étendre `ThreadState` existant (`#[serde(default)]`) et `confirm_journal` |
| Multiplexage SSH `mux_slots` sur `to-master` | `rg -n "federation_stream_run\|ControlMaster" scripts/federate-ssh.sh` | `scripts/federate-ssh.sh:79,428` | `REUTILISER` | Étendre `federate-ssh.sh` existant en préservant `federation_connection` (`ControlMaster=no`) |
| Sous-module `daemon/maintenance.rs` | `ls crates/bridget-daemon/src/daemon/` | `daemon/metrics.rs`, `daemon/mobile_admin.rs` | `CREER` | Extraction demandée (US7) suivant le pattern exact des sous-modules `daemon/*.rs` déjà en place |

## Arbitrages

- Aucune nouvelle dépendance externe.
- Aucun doublon de service, table, endpoint ou script.
- Extraction `daemon/maintenance.rs` justifiée par la taille de `daemon.rs` (19 709 lignes) et alignée sur les sous-modules `daemon/*.rs` existants.

## Gate avant tasks

- [x] Chaque élément du plan a été confronté à l'existant par recherche `rg`
- [x] Aucun doublon évident non arbitré
- [x] Aucune nouvelle dépendance externe non justifiée
- [x] Compatibilité ascendante/descendante vérifiée pour SQLite, `ThreadState`, `federate-ssh.sh` et `update.py`
