# Rapport d'implémentation et de livraison - Spec 307

**Date** : 2026-11-21
**Branche** : `main` (décision branche unique du 10 novembre 2026)
**Build-id livré** : `309-hygiene-reactivite-r1` (version `0.8.53`)
**Commit code + spec** : `7b05d6a5d8c961ed40cfe0e29cf78fc43f5e1430`

## Mesures avant / après

| Indicateur | Avant (`308-purge-fleet-r1`) | Après (`309-hygiene-reactivite-r1`) | Remarque |
|---|---|---|---|
| Nettoyage automatique `scripts/build.py` (`monordinateur`) | Bloqué par `cwd_unavailable_enoent` (processus externe `uv`) | **23,15 Gio** libérés automatiquement (`target/debug`), `0,00 Gio` bloqué ensuite | Garde fail-closed conservée pour Cargo/Rust/Bridget/compilateurs |
| `auto_admission_receipts` (`chosen`, abandonnés) sur Cartae | `5` | **`0`** | Purgés dès le 1er passage d'entretien (+120 s) avec pierres tombales |
| `auto_admission_receipts` (`admitted` > 14 j, orphelins) sur Cartae | `3 381` | **`2 869`** (-512 au 1er passage) | Drain borné à 512 lignes par passage horaire ; `1 000` tâches actives préservées |
| `bridget updates status` sur `monordinateur` | `client_check_unavailable` (`update_daemon_unavailable`, génération 19) | `waiting_idle` (génération 20, erreur effacée, candidat vidé) | Passe à `up_to_date` dès que `com.bridget.t3` (encore en `307`) sera redémarré |
| `bridget updates status` sur `cartae`, `sysle2`, `ksstore` | `up_to_date` | `up_to_date` (`rollback` pointé vers `308-purge-fleet-r1`) | 3 machines Linux alignées |
| `bridget-core.service` mémoire sur Cartae | `124 Mio` | **`138,0 Mio`** (64 threads) | Stable après redémarrage et premier passage d'entretien |
| Relais `cartae-master177` (`monordinateur` -> Cartae) | `ControlMaster=no` (nouvelle poignée de main SSH à chaque flux) | `mux_slots=16` (16 sockets privés `m0.sock`..`m15.sock`, 1 à 4 flux par slot) | `mux_slots=8` a saturé `MaxSessions=10` (53 flux persistants) ; basculé à `16` (capacité 160 flux) |
| Latence `bridget who` depuis `monordinateur` | `3,62 s` | `2,17 s` à `2,32 s` (projet) ; `3,06 s` (`--global`) | L'écart résiduel vient de Cartae même (`1,99 s` en local sur Cartae à cause de ~2 180 anciens sous-agents `unreachable` encore dans l'annuaire) |
| Sous-agents `stopped` UUID dans `bridget who` | `1 001` affichés par défaut | **Masqués par défaut** (compteur en pied de liste, `--all` pour tout afficher) | Protocole et sortie JSON inchangés |

## Vérifications exécutées

- `ruff check scripts` : PASS (0 erreur).
- `bash -n scripts/*.sh` : PASS.
- `scripts/tests/test_build_cleanup.py` : **85/85 OK**.
- `scripts/tests/test_updates.py` : **27/27 OK**.
- `scripts/tests/federation_089_test.sh` : **31/31 OK**.
- `scripts/tests/federation_095_test.sh` : **24/24 OK** (inclut `test307_mux_slots_persisted_for_to_master_and_rejected_for_from_master`).
- `make fmt` + `make lint` (`clippy --workspace --all-targets --locked --features bridget-daemon/test-support -- -D warnings` + `fmt --check`) : PASS (0 avertissement).
- `bridget-transport` (`journal::`) : **18/18 OK**.
- `bridget-daemon` (`spec307_`, `purge_fleet_retire`, `desired_state::tests::`, `spec163_t027`, `spec101_t3_journal_confirme`, `table225_parser_tests`) : **26/26 OK** (1 ignoré contrôlé par sous-processus).
- ABI Linux Debian 12 (`sysle2`) : `scripts/check-linux-abi.py` -> `compatible` (`required_glibc: 2.34 <= 2.36`), SHA-256 `80f5aba9a9e1c8a18e00901999898105c22d2688730966819b175ec93862089b`.
- Compatibilité formats bidirectionnelle `309-hygiene-reactivite-r1` <-> `308-purge-fleet-r1` : `{"valid":true}` dans les deux sens sur les 4 machines.

## Écarts et limites connus

- **Pont T3 (`com.bridget.t3`)** : toujours sur `307-memoire-daemon-r1` (PID `3440`) sur `monordinateur`, conformément à la règle permanente exigeant une confirmation explicite avant tout arrêt ou redémarrage de T3.
- **Clients MCP déjà ouverts sur `monordinateur`** : 3 processus `bridget mcp` restent en `288-preparation-guidage-r1` et 4 en `308-purge-fleet-r1` ; leurs binaires sont conservés tant qu'ils sont ouverts.
- **`macbook-m1`** : hors ligne (`192.168.1.51`), non déployé.
- **Latence `bridget who` sous 1 s (SC-004)** : partiellement atteinte. Le multiplexage `mux_slots=16` élimine la saturation `MaxSessions=10` et abaisse la latence projet de `3,62 s` à `2,17 s`, mais `list_agents()` sur Cartae retourne encore `3 271` entrées dont ~`2 180` sous-agents `unreachable` (`1,99 s` en local sur Cartae). Leur purge ou filtrage relève d'un lot dédié.
- **Contre-revue adverse inter-fournisseur** : non exécutée (aucun autre agent Bridget d'un fournisseur différent joignable sur ce projet et outil `bridget_send` absent de cette session) ; revue complète réalisée sur le diff et les invariants avant commit.
