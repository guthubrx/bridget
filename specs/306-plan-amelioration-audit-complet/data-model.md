# Modèle de données : Programme complet d'amélioration post-audit V15 (306)

## 1. Cache de registre (`AgentRegistryCache`)
- **Clé de validation** : `(path: PathBuf, modified: SystemTime, len: u64, inode: u64)` (Unix).
- **Valeur** : `Arc<AgentRegistry>` (lecture partagée sans clone profond sur chemin chaud).
- **Invariant** : toute modification de fichier (écriture atomique par renommage changeant l'inode, ou modification de taille/date) invalide immédiatement l'entrée et recharge depuis le disque.

## 2. Enveloppe inter-agent sécurisée (`bridget-core/src/envelope.rs`)
- **Délimiteur unique** : `BRIDGET_MSG_<id_prefix>` généré déterministiquement depuis `msg.id` (UUID), encadrant `msg.body` afin qu'aucune balise statique contenue dans le corps ne puisse fermer prématurément l'enveloppe.
- **Politique de filtre de secrets** (`SecretFilterMode`) : `Off` (défaut rétrocompatible) | `Redact` | `Reject`, pilotée par variable d'environnement `BRIDGET_SECRET_FILTER`.

## 3. Gardes bornés `bridget-core`
- `EnvelopeGuard` : `seen: HashSet<String>`, `order: VecDeque<(Instant, String)>`, `max_entries: usize` (défaut 10 000). Éviction en tête en O(1) amorti.
- `Deduplicator` : clé `(content_key: String, target: String)`, `order: VecDeque<(Instant, (String, String))>`, `max_entries: usize` (défaut 10 000).
- `CircuitBreaker` : `history: HashMap<String, VecDeque<Instant>>`, suppression de la clé dès que son `VecDeque` devient vide après purge.

## 4. Plafond de connexions Unix (`UnixConnectionBudget`)
- `general_active: AtomicUsize` (plafond 256) + `reserved_active: AtomicUsize` (plafond 32 pour workers/admin).
- Garde RAII `ConnectionPermit` libérant systématiquement le compteur à la fin du thread, y compris en cas d'erreur ou de panic.

## 5. Archive de sauvegarde (`BackupManifest`)
```json
{
  "format_version": 1,
  "created_at_ms": 1763668800000,
  "bridget_version": "0.8.53",
  "entries": [
    { "relative_path": "bridget.db", "kind": "sqlite_hot_copy", "bytes": 123456, "sha256": "..." }
  ]
}
```
- Chaque base SQLite est copiée à chaud via `VACUUM INTO` sur un fichier temporaire privé `0600`, puis vérifiée par `PRAGMA integrity_check`.

## 6. Rapport de diagnostic (`DoctorReport`)
- Regroupe : statut daemon, `metrics` (12 compteurs), `native_tick_latency_ms` (`p50`, `p95`, `p99`, `samples`), état des sockets et connexions actives, schéma/mode journal de chaque base SQLite, santé workers distants, passerelle mobile et synthèse `ok` / `degraded` / `error`.

## 7. Plan de simulation (`DelegationDryRunPlan`)
- Retourne : candidat sélectionné (machine, fournisseur, modèle, score), candidats écartés avec motif structuré, vérification des quotas/permissions et `would_spawn: bool`, sans créer d'enregistrement en base ni réserver de slot.

## 8. Modèles Android typés (`MobileModels.kt`)
- `data class` Kotlin remplaçant les `org.json.JSONObject` bruts pour : `MobileSessionState`, `AgentSummary`, `ProjectSummary`, `ConversationEvent`, `PendingQuestion`, `SpecialistProfile`, `NotificationFilterPreferences` (`projectFilter`, `minUrgency`).
