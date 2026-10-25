# T039 - Scénarios dégradés 149 (daemon natif réel, hôte MCP adaptateur de bureau réel, enfants fermés)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent), ronde natif/réseau r1.
Aucun code de production modifié. Aucun commit. Aucune case cochée.

## Verdict

**CHANGES_REQUIRED sur un seul point (R9.2). T039 n'est pas cochable en l'état.**

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/recovery149-r1.md#extrait-1).

## Constat F1 (R9.2) - mission en vol et redémarrage du daemon

**Observé à chaque passage** (3 passages complets de `recovery149.ts`, 3 de `server149.ts`, 2 explorations manuelles) :

1. Une mission native est en cours (`SLOW_149:9000`, enfant vivant). Le daemon reçoit SIGTERM puis est relancé.
2. Moins d'une seconde après le début de l'arrêt (relance du daemon comprise, +0,44 s), le daemon **relance un fournisseur enfant** (nouveau PID) et lui envoie d'abord la carte de reprise (1342 caractères), puis **la mission d'origine une seconde fois** (772 caractères, +0,60 s).
3. Environ 2 s après la relance, la même tâche passe `failed` avec l'erreur `unreachable` (horodatage à la seconde). Le parent reçoit une seule notification d'échec.
4. Résultat : la mission a été présentée à **deux fournisseurs** (2 PID distincts, 2 tours portant `NONCE_i1`) alors que la tâche est déclarée échouée. Lancements d'enfants : 12 avant, 13 après.
5. Dans `server149.ts`, deux missions en vol (WAIT) donnent 4 -> 6 lancements après la relance, avec ces tâches en `failed: unreachable`.

Contraintes violées si le comportement est voulu comme tel :

- `spec.md` US6 « Reprise » : « sans dupliquer l'enfant ni relancer la mission ».
- `plan.md` : « Les reprises gardent la tâche, le modèle, la politique et la clé de remise ».

Deux mécanismes se contredisent. Le plan d'exécution marque l'exécution `unreachable` donc la tâche `failed` (`native_delegation.rs`, lecture du snapshot d'exécution). La flotte managée, elle, relance la commande figée avec une carte de reprise (comportement voulu par le test `daemon.rs` « reprise persistante »). Aucun des deux n'annule l'autre.

Risque : une mission qui écrit un fichier est exécutée deux fois alors que le parent lit un échec.
Limites de ce constat : le fournisseur est un faux serveur Codex ; un vrai Codex pourrait traiter la carte de reprise autrement. Le binaire est antérieur aux trois fichiers modifiés depuis.
Décision attendue du principal : (a) la tâche reste vivante et reprend, ou (b) la tâche échoue ET aucun enfant n'est relancé ni ne reçoit la mission.

## Les six scénarios US6 et les extras

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/recovery149-r1.md#extrait-2).

## Détail des contrôles

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/recovery149-r1.md#extrait-3).

## Comptage réel des exécutions, processus et remises

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/recovery149-r1.md#extrait-4).

Mesures de reprise en vol (relevé brut) :

```json
[{"dt": 0.44, "event": "started", "pid": 14845}, {"dt": 0.44, "event": "thread_start", "pid": 14845}, {"dt": 0.58, "event": "prompt", "pid": 14845, "nonce": "none", "turn": 1, "chars": 1342}, {"dt": 0.58, "event": "answered", "pid": 14845, "nonce": "none"}, {"dt": 0.6, "event": "prompt", "pid": 14845, "nonce": "i1", "turn": 2, "chars": 772}]
{"state": "failed", "error": "unreachable", "updated_at": 1791638224, "completed_at": 1791638224, "started_at": 1791638220, "created_at": 1791638220, "restartAtEpoch": 1791638222}
```

## Droits après perte de session (relevé de R8.3)

```json
{"tombstone": "t3_session_unavailable", "oldCredentialAfterRotation": "t3_session_unavailable", "newCredentialWithoutFact_replay": "ok:result_available", "newCredentialWithoutFact_status": "ok:result_available"}
```

Lecture : le credential tombstoné et l'ancien credential sont refusés. Un credential neuf SANS fait (v1) rend l'état mémorisé de la requête déjà admise (rejeu et status) sans rien relancer. Une nouvelle admission avec v1 reste refusée (interop149.md S4.3).

## Observations secondaires

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/recovery149-r1.md#extrait-5).

## Non couvert

Modèle réel, voie standalone Claude/GLM, observer PTY, `can_use_tool` corrélé, SIGKILL du daemon, coque desktop et UI. Voir `interop149.md`, section « Ce qui n'est PAS prouvé ici ».

Résultats bruts : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/recovery149.json` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/recovery149.stdout.txt`.
Commande : `node recovery149.ts` dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes` (voir `README.md`).
