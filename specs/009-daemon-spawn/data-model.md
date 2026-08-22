# Data Model : équipiers gérés par le daemon

## État désiré (`~/.config/bridget/fleet.json`, 0600)

Daemon **seul écrivain** ; édition manuelle daemon arrêté seulement ; chargé au
démarrage ; écriture durable = temp + fsync + `rename` + fsync du répertoire.

```json
{
  "schema": 1,
  "equipiers": {
    "codex-1": {
      "type": "codex",
      "cwd": "/chemin/absolu/capture",
      "command_id": "…",
      "generation": 4,
      "created": "2026-08-22T20:14:00Z"
    }
  }
}
```

Clé stable = nom d'équipier. `generation` croît à chaque spawn du même nom ;
`stop` retire l'entrée **durablement avant** de répondre (FR-010bis).

## Marqueur de groupe (`~/.cache/bridget/managed/<nom>.json`, 0700/0600)

```json
{ "pgid": 4211, "birth": 1766434440, "instance_id": "…", "command_id": "…", "generation": 4 }
```

Écrit + fsync par le daemon **entre** `BootstrapReady` et `RELEASE` (D-502).
Valide seulement si `birth` et `instance_id` concordent (anti-pid-recyclé).
Supprimé après disparition confirmée du groupe ou échec terminal consigné.

## Issues de commandes (SQLite, table bornée `spawn_commands`)

| Colonne | Rôle |
|---|---|
| `command_id` | clé d'idempotence |
| `name`, `generation` | rattachement |
| `state` | `Requested`/`Reserved`/`Starting`/`Connected`/`Failed`/`Cancelled` |
| `issue` | issue terminale rejouable (motif typé) |
| `ts` | rétention (purge alignée demandes ; expiration → `IdempotencyExpired`) |

### Table de vérité de récupération (D-503, unique issue reconstruite)

Précondition : présence et `state` de la ligne `spawn_commands` (« — » =
indifférent) :

| ligne `spawn_commands` | `fleet` | issue | marqueur | Récupération |
|---|---|---|---|---|
| terminale | — | présente | — | rejouer l'issue |
| en vol | présent | absente | présent | reprise en cours : rattacher le retry à la génération, attendre le vrai terminal |
| en vol | présent | absente | absent | crash avant bootstrap durable : la reprise relancera ; retry rattaché, attend le terminal |
| terminale | absent | présente | — | spawn éphémère ou stop postérieur : rejouer l'issue |
| — | absent | absente | présent | groupe orphelin d'une génération retirée : terminer le groupe, consigner |
| **en vol** (`Requested`/`Reserved`/`Starting`) | absent | absente | absent | **crash avant toute durabilité fleet/marqueur** : reconstruire une **unique** issue `Failed`/`Cancelled` motivée, la persister **avant** de répondre — jamais `IdempotencyExpired` (point de crash testé) |
| absente (ou purgée par rétention) | absent | absente | absent | `command_id` inconnu/expiré → `IdempotencyExpired` |

## Canal de statut (FD unique hérité, D-501/D-502)

| Événement | Émetteur | Sens |
|---|---|---|
| `BootstrapReady { pid, pgid, birth, instance_id, command_id, generation }` | bootstrap (avant `RELEASE`) | jamais un succès |
| `StartupFailed { kind, reason, instance_id, command_id, generation }` | wrapper (hook `managed-status`) | motif exact de FR-005 |
| (le succès) | `Register` réel sur la socket daemon | seule source de `Connected` |

La corrélation `instance_id`/`command_id`/`generation` est vérifiée à la
réception : un événement d'un lancement obsolète ne peut jamais terminer le
lancement courant (fixture croisée imposée).

## `StopOutcome` (réponse synchrone typée)

`Stopped` (chemin propre complet) \| `StoppedForced { survivors_killed }`
(escalade `killpg`) \| `NotManaged` \| `NotFound` \| `Timeout { state }`.

## Machine d'états du spawn (mémoire daemon, générations)

`Requested → Reserved (nom+slot atomiques) → Starting (bootstrap+RELEASE) →
Connected (Register) | Failed (motif typé) | Cancelled (stop/timeout)` —
transitions consignées dans `spawn_commands`, `stop` invalide la génération à
tout état (D-506).
