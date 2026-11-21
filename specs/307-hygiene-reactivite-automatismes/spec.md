# Spécification 307 : Hygiène, réactivité et déblocage des automatismes Bridget

**Statut** : En cours
**Branche** : `main` (décision branche unique du 10 novembre 2026)
**Date** : 2026-11-21
**Dépendances** : specs 089/095 (fédération SSH), 116 (entretien périodique), 150 (rétention délégation native), 209/221 (mises à jour vérifiées), 306 (audit complet).

## Demande initiale et cadrage (Phase 0)

**Demande initiale :**
> `/my-specify-all fais toute ces corrections/evolutions` (suite au bilan des 7 frictions mesurées sur Cartae et `monordinateur` le 21 novembre 2026).

**Demande reformulée :**
Corriger, tester, livrer et activer sur `main` et les 4 machines joignables (`monordinateur`, `cartae`, `sysle2`, `ksstore`) un lot de 7 améliorations couvrant l'hygiène disque/base, le déblocage des automatismes, la réactivité de l'annuaire et du pont SSH, la reprise du pont T3 et le découpage progressif de `daemon.rs`.

**Exclusions explicites :**
- Aucune modification de la configuration système `/etc/ssh/sshd_config` sur Cartae.
- Aucune suppression de donnée métier ou d'historique de messages (`ledger`).
- Aucun arrêt non autorisé de l'application T3 Code ; `macbook-m1` reste hors périmètre tant qu'il est hors ligne.

## Exigences fonctionnelles et non fonctionnelles

### US1 (P1) - Débloquer le nettoyage automatique des caches de build (`scripts/build.py`)
- **FR-001** : Lors d'une observation bornée à des dossiers cibles (`scopes` non vide), un processus externe sans lien avec Cargo, Rust, un compilateur/éditeur de liens ou Bridget dont seul le descripteur `cwd` est introuvable (`cwd_unavailable_enoent`) ne doit plus annuler le nettoyage des caches de build.
- **FR-002** : La garde reste fail-closed pour tout processus lié à Cargo/Rust/Bridget/compilateur, pour tout processus dont le nom de commande est illisible, et pour tout chemin ouvert dans `scopes`.
- **FR-003** : La fonction `process_output_complete()` conserve sa sémantique stricte et ses diagnostics fermés existants (sans exposer de chemin sensible).

### US2 (P1) - Débloquer les automatismes de mise à jour (`bridget updates`)
- **FR-004** : L'entretien périodique du démon purge de façon bornée les admissions automatiques abandonnées (`state IN ('pending', 'decision_started', 'chosen')`, `task_id IS NULL`) inactives depuis plus de 24 heures et sans tâche correspondante dans `native_delegations`.
- **FR-005** : Dans `scripts/update.py`, lorsque la version installée est déjà à jour (`candidate` absent ou `<= installed`) et que les services sont sains, l'outil de mise à jour rapporte `up_to_date` sans rester figé sur un ancien `client_probe_block_generation` ; en revanche, lorsqu'un candidat plus récent existe et qu'une sonde `probe_stop_unconfirmed` a bloqué la génération courante, le blocage automatique est conservé jusqu'à une nouvelle génération (`bridget updates check`).

### US3 (P1) - Maîtriser la croissance de `bridget.db`
- **FR-006** : L'entretien périodique purge par lots bornés (jusqu'à 512 lignes par passage) les anciens reçus `auto_admission_receipts` en état `admitted` vieux de plus de 14 jours dont la tâche n'existe plus dans `native_delegations`.
- **FR-007** : Le schéma SQL de `auto_admission_receipts` reste strictement inchangé (compatibilité bidirectionnelle des formats conservée).
- **FR-008** : L'entretien périodique exécute un entretien SQLite non bloquant (`PRAGMA wal_checkpoint(PASSIVE)`, `PRAGMA incremental_vacuum(256)` et `PRAGMA optimize`).

### US4 (P1) - Rendre `bridget who` rapide et lisible
- **FR-009** : `list_agents()` charge les noms d'affichage (`agent_profiles`) en une seule requête SQL groupée au lieu d'une requête par agent en boucle (suppression du N+1 sous le verrou global du démon).
- **FR-010** : La commande CLI `bridget who` masque par défaut les sous-agents natifs arrêtés (état `stopped` et identifiant UUID) en conservant tous les agents actifs, injoignables et nommés arrêtés, et indique en pied de liste le nombre de sous-agents masqués lorsqu'il est non nul.
- **FR-011** : L'option `--all` sur `bridget who` affiche tous les agents sans filtrage, et le contrat protocole/JSON de l'annuaire reste inchangé.

### US5 (P2) - Accélérer la reprise du pont `com.bridget.t3`
- **FR-012** : `IncrementalJournalReader` accepte un offset initial (`from_offset`) et repart proprement à 0 si le fichier a été tronqué ou tourné (`len < offset`).
- **FR-013** : `ThreadState` persiste la position de lecture (`journal_offsets`, champ rétrocompatible `#[serde(default)]`) de chaque journal JSONL observé.
- **FR-014** : `confirm_journal` supprime les scans linéaires O(N²) sur `seen`, `seen_activities` et `ended_turns` grâce à des ensembles de recherche O(1).

### US6 (P2) - Réduire la latence et le bruit SSH Mac vers Cartae (`scripts/federate-ssh.sh`)
- **FR-015** : `federation_connection` conserve `ControlMaster=no` par défaut (contrats 089/095 inchangés).
- **FR-016** : Le mode `to-master` accepte une configuration optionnelle `mux_slots` (1 à 16, option CLI `--mux-slots`) permettant à `federation_stream_run` de réutiliser un pool privé de sockets `ControlMaster=auto` (`ControlPersist=120`) dans le dossier privé 0700 de l'installation.
- **FR-017** : L'installation locale `cartae-master177` sur `monordinateur` est basculée proprement avec reçu réattesté (`mux_slots=8`), ramenant la latence de `bridget who` sous 1 seconde (contre 3,6 s mesurés avant).

### US7 (P3) - Amorcer le découpage modulaire de `daemon.rs`
- **FR-018** : Le bloc cohérent d'entretien périodique (`sweep_held`, `maintain_stores`, `run_maintenance`, `spawn_maintenance_thread` et ses constantes) est extrait de `crates/bridget-daemon/src/daemon.rs` vers `crates/bridget-daemon/src/daemon/maintenance.rs` sans changement de comportement.

## Critères de réussite mesurables

- **SC-001** : `python3 scripts/build.py clean --dry-run` s'exécute sans refus `cwd_unavailable_enoent` sur `monordinateur` en présence d'un processus externe au `cwd` disparu, et tous les tests `scripts/tests/test_build_cleanup.py` passent.
- **SC-002** : Tous les tests `scripts/tests/test_updates.py` passent, et `bridget updates status` sur Cartae ne montre plus les 5 vieilles admissions abandonnées après entretien.
- **SC-003** : Sur Cartae, la table `auto_admission_receipts` diminue après le premier passage d'entretien sans perte de tâche active.
- **SC-004** : `bridget who` depuis `monordinateur` répond en moins de 1,0 s (contre 3,62 s avant) et n'affiche plus par défaut les ~992 sous-agents natifs `stopped` (`--all` les réaffiche).
- **SC-005** : Les tests unitaires de `IncrementalJournalReader::from_offset` et de persistance `journal_offsets` dans `t3code` passent.
- **SC-006** : `scripts/tests/federation_089_test.sh`, `scripts/tests/federation_095_test.sh` et le nouveau test `mux_slots` passent.
- **SC-007** : `make lint` (`clippy -D warnings` + `fmt`), tests Rust ciblés, tests Python, contrôle ABI Debian 12 (`glibc <= 2.36`) et compatibilité stockage bidirectionnelle sont verts, committés, poussés sur `main` et activés sur les 4 machines joignables.
