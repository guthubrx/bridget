# Contrat : état de contrôle v1

Constante : `CONTROL_STATE_CONTRACT_VERSION = 1`. Capacité client : `ClientCapability::ControlStateV1`.

## Trames

### `WrapperToDaemon::ControlStateRead { version: 1 }`
Rôle : `Client`, capacité `ControlStateV1` ou `Lookup`. Réponse : `DaemonToWrapper::ControlState`.

### `WrapperToDaemon::ControlStateSet`
```json
{ "version": 1, "command_id": "<uuid>", "expected_generation": 12,
  "paused": true | false | null, "auto_objectives_cap": 5 | null, "reason": "…" | null }
```
Rôle : `Client`, capacité `ControlStateV1`. Au moins un champ non nul. Réponse : `ControlState` (generation + 1) ou `ControlStateRejected { reason }`.
Quand `paused` devient vrai, Bridget mémorise chaque exécution active et émet une commande interne `Interrupt` nommée `control-pause-<generation>-<execution_id>`. Quand `paused` devient faux, les exécutions mémorisées sont reprises pour les agents connectés et libres.

Refus fermés `ControlStateRefusal` :
- `HumanPrincipalRequired` : l'émetteur n'est pas attribué au principal humain ;
- `GenerationMismatch { current }` ;
- `BudgetOutOfRange { min: 1, max: 100 }` ;
- `NothingToChange` ;
- `StoreUnavailable`.

### `DaemonToWrapper::ControlState`
```json
{ "version": 1, "generation": 13, "paused": true, "paused_since": 1788400000,
  "paused_by": "humain", "pause_reason": "…", "auto_objectives_cap": 5, "updated_at": 1788400000 }
```

## Garde de la ronde

`ProjectRoundRefusal::ControlPaused` est rendu par `ProjectRoundDispatch` avant `PolicyDisabled`, et `record_project_round_dispatch` l'enregistre en `Refused`.

## Rendu CLI

`bridget control status` imprime les mêmes champs. `bridget control pause [--reason]`, `bridget control resume`, `bridget control budget <n>` exigent un terminal interactif en entrée et en sortie, sinon refus « contrôle = terminal interactif uniquement ». Le pied de `bridget who` ajoute une ligne `Contrôle : …` après `Daemon build-id`.
