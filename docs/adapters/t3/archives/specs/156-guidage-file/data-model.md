# Archives techniques de l’adaptateur T3

Document d’origine : [specs/156-guidage-file/data-model.md](../../../../../../specs/156-guidage-file/data-model.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

```text
GuidanceFacts {
  action: Steer | Interrupt | MidTurnDelivery | QueueList,   // MidTurnDelivery : guidage implicite T3
  path: SendIdempotent | Send | Control | Cancel,
  actor: AttachedHuman | BareClient | Agent{ id } | InternalPause,
  identity_attested: bool,
  trusted_origin: Human | System | None, // MessageOrigin existant ; preuve/source interne, distincte du rôle, jamais payload seul
  child_proof: bool,            // preuve enfant ADR 044
  actor_is_target: bool,
  actor_is_parent: bool,        // agent_link ouvert ou tâche native propriétaire
  actor_is_child: bool,         // seulement pour MidTurnDelivery
  cross_project: bool,
  paused: bool,
  target_has_active_native_task: bool,
  now: i64,                    // même horloge injectée que les lectures
  recent_interrupts_pair: u32,  // COUNT inclusif sur 30 s
  last_interrupt_pair_at: Option<i64>, // LAST admis du couple acteur/cible
  recent_interrupts_target: u32, // COUNT inclusif sur 10 min
  third_latest_interrupt_target_at: Option<i64> // troisième événement cible le plus récent
}
GuidanceRefusal =
  IntentNotPermitted{ reason: not_parent | cross_project | self_target | identity_unattested }
  | ReferentPaused | MissionActive | InterruptRateLimited{ retry_after_secs }
```
