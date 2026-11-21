# Modèle de données et états persistés - Spec 307

Aucune table SQL ni migration de schéma n'est ajoutée. Toutes les évolutions sont rétrocompatibles avec les fichiers et bases existants.

1. **`ThreadState` (`crates/bridget-daemon/src/t3code.rs`)** :
   - Ajout du champ `#[serde(default, skip_serializing_if = "BTreeMap::is_empty")] pub journal_offsets: BTreeMap<String, u64>`.
   - Clé : chemin canonique texte du fichier JSONL ; valeur : dernier offset d'octet confirmé.
   - Si le fichier sur disque a une taille `< offset` (rotation/troncature), le lecteur repart à `0`.
   - Un ancien binaire ignore ce champ ou repart de 0 ; un nouveau binaire lit un ancien état avec `BTreeMap::new()`.

2. **Configuration de fédération `to-master` (`scripts/federate-ssh.sh`)** :
   - Champ optionnel `mux_slots=<N>` (`1 <= N <= 16`) dans le fichier `config` (mode 0600) uniquement lorsque `direction=to-master`.
   - Sockets de contrôle privées créées par SSH dans `$install_dir/m<0..N-1>.sock` (dans un répertoire 0700 possédé par l'utilisateur).
   - Le reçu (`receipt`) conserve les 9 champs stricts (dont `config_sha256` qui couvre `mux_slots`).

3. **SQLite `auto_admission_receipts` (`bridget.db`)** :
   - DDL inchangé (compatibilité stricte `delegation.rs:595`).
   - Purge bornée ajoutée :
     - `state IN ('pending', 'decision_started', 'chosen') AND task_id IS NULL AND updated_at < now - 86400` sans ligne correspondante dans `native_delegations` ;
     - `state = 'admitted' AND updated_at < now - 14 * 86400` sans ligne correspondante dans `native_delegations` (ni par `task_id`, ni par `(owner, request_id)`).
