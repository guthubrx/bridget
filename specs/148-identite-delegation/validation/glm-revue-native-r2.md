# Revue R2 indépendante — Session 148, moteur natif + intégration adaptateur de bureau

Lecture seule respectée. Aucun test, Git, build, daemon ni modèle lancé. Numéros de ligne = état du worktree lu le 2026-10-10.

## Objections R1 — toutes closes

| Objection | Verdict | Preuve lue |
|---|---|---|
| D1 transfert de tâche (majeur) | **Close** | `can_own_task` (`native_delegation.rs:105-139`) : présence `connected`/`busy` **ou** connexion primaire non-auxiliaire vivante de l'instance d'origine bloque rejeu, `Status` et `Cancel`. `Status` ne mute plus. Le tick passe par `recover_owner`. Test `native148_live_original_owner_blocks_replay_status_and_cancel` : les trois chemins refusés, puis connexion primaire seule suffit, puis reprise après départ réel. `owner_instance` inchangé pendant les refus. |
| D2 révocation perdure (moyen) | **Close** | Tombstone `native_delegation_revocations(agent_id)` consulté en tête de `root_permission` (`native_delegation.rs:62`), avant grant d'instance, héritage managed et attestation projet. `grant_agent` (`delegation.rs:214-241`) ferme d'abord le droit stable, puis l'instance. Test : réouverture du store (durable), nouvelle instance + attestation projet + grant d'instance restent fermés ; seul `grant_agent` humain lève. Le puits Grant exige rôle Client + acteur contrôle humain + ControlStateV1. |
| D3 lecture inter-conversations | **Close** | Même garde ; `Status` devient lecture pure (`native_delegation.rs` commentaire explicite). |
| D4 zombie si capture échoue | **Close** | `capture_reply` retourne `Result`. `Err` → NACK `native_result_not_persisted` dans `SendIdempotent` (`daemon.rs:~8647`) et `Send` (`daemon.rs:~17415`), tous deux **avant** `prepare_dispatch` : aucune livraison directe. Test `native148_capture_database_failure_never_acks_or_delivers_directly` : trigger SQLite d'échec d'UPDATE, puis DROP TABLE ; aucun ACK, aucun octet émis sur le socket, tâche `working`, résultat absent. |
| Observation « completed sans reply → working à vie » | **Close** | `MISSION_REPLY_LIMIT_SECS = 3600`. Échéance figée à l'admission via `get_or_insert` (jamais renouvelée au rejeu, lignes ~698/827), recalculée durablement au tick si absente, expiration → `mission_reply_timeout` + cleanup. Visible au catalogue (`mission_reply_limit_secs`) et au reçu (`mission_deadline_at`). Test horloge dédié. |

## Intégrations demandées — vérifiées

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-revue-native-r2.md#extrait-1).

## Nouveaux défauts

Aucun bloquant ni majeur. Deux notes :

1. **INFO** — Un résultat enfant invalide (corps vide ou >256 Kio) produit le NACK `native_result_not_persisted` alors que la cause réelle est `native_result_invalid` (logguée seulement). Fail-closed, échéance bornée. Libellé trompeur pour l'enfant, sans effet de sécurité.
2. **NOTE** — Le worktree Bridget évoluait pendant ma lecture (`native_delegation.rs` passé de ~2100 à 2324 lignes ; introduction de `ParentPermission` et d'un test `attested_git_discovery`). J'ai relu les sections critiques dans l'état final : toutes les gardes validées y sont inchangées ou renforcées.

## Manques de preuve (distincts des défauts)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-revue-native-r2.md#extrait-2).

## Verdict : APPROVE

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-revue-native-r2.md#extrait-3).
