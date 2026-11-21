# Recherche technique - Spec 307

## 1. Garde `scripts/build.py` (`cwd_unavailable_enoent`)
- **Constat mesuré** : sur `monordinateur`, PID `92290` (`/Users/moi/.cache/uv/...`) a un `cwd` supprimé, produisant `fcwd` / `ncwd|rtd info error: No such file or directory` dans `lsof -n -P -F pftDn`. `process_output_complete()` renvoie `False` et annule tout nettoyage (14 Gio dans `target`).
- **Contrainte** : `scripts/tests/test_build_cleanup.py:64-75` exige que `process_output_complete()` continue de refuser ce motif en entrée brute.
- **Décision** : conserver `process_output_complete()` intact et introduire un filtre amont dans `busy_paths()` uniquement lorsque `scopes` est non vide (`drop_foreign_dead_cwd_records`). Un enregistrement `cwd_unavailable_enoent` n'est ignoré que si la commande du PID est lisible, n'appartient pas aux outils de build/Bridget (`cargo`, `rustc`, `rustdoc`, `clippy-driver`, `bridget`, `cc`, `clang`, `gcc`, `ld`, `lld`, `ar`, `ld64`) et que le PID n'a aucun autre descripteur ouvert sous `scopes`.

## 2. Automatismes `bridget updates`
- **Admissions abandonnées** : sur Cartae, 5 lignes `state='chosen'` dans `auto_admission_receipts` datent du 6 au 17 novembre (`task_id IS NULL`) et maintiennent `activation: blocked`. `delegation.rs:595` impose un schéma DDL exact (ne pas modifier la table). Une purge bornée `purge_expired_auto_admission_abandoned(now, limit)` sur `state IN ('pending', 'decision_started', 'chosen') AND task_id IS NULL AND updated_at < now - 86400` sans tâche associée résout le blocage sans toucher au schéma.
- **Sonde `probe_stop_unconfirmed`** : `scripts/tests/test_updates.py:156-167` exige de ne pas répéter `client_check` automatiquement pour un candidat en attente sur la même `check_generation`. En revanche, dans `scripts/update.py:545`, le test de `client_probe_block_generation` s'exécute avant la vérification `version(pending) <= version(installed)`, maintenant `monordinateur` bloqué en `client_check_unavailable` même lorsqu'aucune mise à jour n'est à activer. Déplacer cette garde juste avant le téléchargement/activation d'un candidat plus récent résout le blocage tout en préservant l'invariant de `test209`.

## 3. Croissance `bridget.db`
- **Constat mesuré** : `auto_admission_receipts` occupe 113,86 Mio (3 471 lignes, 3 381 `admitted`, moyenne 23,6 Ko de `metadata`, plus ancienne à 38,1 jours). `freelist_count = 7 735` (~30 Mio).
- **Décision** : ajouter `purge_expired_auto_admission_admitted(now, limit)` (rétention 14 jours, `task_id` absent de `native_delegations`, lot <= 512 par passage) dans `delegation.rs`, appelée depuis `maintain_retention`, plus `PRAGMA wal_checkpoint(PASSIVE)`, `PRAGMA incremental_vacuum(256)` et `PRAGMA optimize` dans `maintain_stores`.

## 4. `bridget who` N+1 SQLite et lisibilité CLI
- **Constat mesuré** : `daemon.rs:5363-5588` appelle `self.store.agent_display_name(agent_id)` individuellement pour chacun des ~1 050 agents listés sous `Mutex<DaemonState>`. `cmd_who` affiche 992 sous-agents natifs UUID `stopped` mélangés aux 13 agents nommés `stopped`.
- **Décision** : ajouter `Store::agent_display_names(&self) -> Result<HashMap<String, String>, StoreError>` (1 seul `SELECT`) dans `store/ledger_requests.rs`, et filtrer par défaut dans `cmd_who` (`!parsed.all`) les entrées `state == "stopped"` dont `agent_id` est un UUID valide, en ajoutant `--all` à `DirectoryArgs` et un résumé du nombre masqué.

## 5. Reprise JSONL du pont `com.bridget.t3`
- **Constat mesuré** : `/Users/moi/Library/Logs/com.bridget.t3.log` montre des suspensions d'observation de 30 à 46 s après redémarrage car `IncrementalJournalReader::new` repart de l'octet 0 sur des fichiers de 14 à 23 Mo et `confirm_journal` (`t3code.rs:3937-3965`) fait 4 recherches linéaires `.iter().any(...)` sur des vecteurs allant jusqu'à 8 192 entrées.
- **Décision** : ajouter `IncrementalJournalReader::from_offset(path, offset)` (avec repli à 0 si `file_len < offset`), ajouter `#[serde(default)] pub journal_offsets: BTreeMap<String, u64>` dans `ThreadState`, et utiliser des `HashSet` locaux dans `confirm_journal`.

## 6. Relais SSH Mac vers Cartae (`scripts/federate-ssh.sh`)
- **Hypothèse testée par preuve jetable (21 nov. 06:01-06:03 CET)** :
  - Essai 1 (`ssh -N -L` streamlocal Unix) : **rejeté** par Cartae (`channel 1: open failed: connect failed: open failed`), confirmant que le `sshd` de Cartae ne sert pas `direct-streamlocal@openssh.com`.
  - Essai 2 (`ssh -o ControlMaster=auto -o ControlPersist=30` vers `37.59.185.67:2222`) : **validé**, latence mesurée **37 à 47 ms par appel** contre **~900 ms** sans mutualisation (`bridget who` local Cartae : 63 ms ; via relais actuel : 3 617 ms).
  - Contrainte `sshd` Cartae : `MaxSessions 10` pour `Match User moi`. Contrainte tests 089/095 : `federation_connection()` doit conserver `ControlMaster=no`.
- **Décision** : conserver `ControlMaster=no` dans `federation_connection()`, ajouter un paramètre optionnel `mux_slots` (1..16) pour `direction=to-master` dans `scripts/federate-ssh.sh`, utilisé uniquement par `federation_stream_run()` via un pool de sockets privées `$install_dir/m<slot>.sock` (`ControlMaster=auto`, `ControlPersist=120`, 8 slots = 80 canaux simultanés).

## 7. Découpage modulaire de `daemon.rs`
- **Constat mesuré** : `daemon.rs` compte 19 709 lignes. Le bloc entretien périodique (`sweep_held`, `maintain_stores`, `run_maintenance`, `spawn_maintenance_thread`, constantes associées) forme une unité cohérente d'environ 200 lignes sans couplage cyclique.
- **Décision** : extraire ce bloc dans `crates/bridget-daemon/src/daemon/maintenance.rs` en réutilisant le pattern déjà établi par `daemon/metrics.rs` et `daemon/mobile_admin.rs`.
