# Modèle de données - Session157

## Limite connue (mémoire du daemon, module `provider_limits.rs`)

Volatile. Pas de table SQL. Effacée au redémarrage du daemon.

```
ProviderLimits { by_agent_type: BTreeMap<String, BTreeMap<String /*fenêtre*/, Seen>> }
Seen { fact: RateLimitFact, source: RateLimitSource, observed_at: i64 }
```

- Écrite par `handle_rate_limit` (`daemon.rs`), en plus de `Presence.rate_limits`,
  avec la source du fait (le statut Codex est global, le statut Claude est par
  fenêtre).
- Clé : `Presence.agent_type`.
- Bornes : au plus 8 fenêtres par type d'agent. Une fenêtre dont l'heure de
  retour est passée est retirée à l'écriture.
- Lecture : `quota(agent_type, now) -> Option<Quota>`, qui ignore les fenêtres
  dont l'heure de retour est passée.

```
Quota { windows: Vec<{window, used_percent, resets_at, status, exhausted}>,
        exhausted: bool, limit_reached: bool, resets_at: Option<i64>, observed_at: i64 }
```

`exhausted(fenêtre, now)` : `resets_at > now` ET (source Claude `stream-json` avec
`status == "rejected"` OU `used_percent >= 100`). Un statut Codex non
`available` (global, recopié sur les deux fenêtres) ne rend aucune fenêtre
épuisée : il lève `limit_reached`. `Quota.exhausted` = une fenêtre au moins.
`Quota.resets_at` = le plus tardif des retours des fenêtres épuisées. Le
statut `unknown` du hook statusline ne compte jamais.

## Tâche native (`delegation.rs`, struct `Task`, payload JSON)

Champ ajouté, optionnel, pour garder les payloads 148 à 150 lisibles :

| Champ | Type | Défaut si absent | Rôle |
|---|---|---|---|
| `substitution` | `Option<Substitution>` | `None` | Trace durable du remplacement |

```
Substitution { requested: {agent_type, model, effort},
               selected:  {agent_type, model, effort},
               reason: String /*code de refus de la cible demandée*/,
               tried: Vec<{agent_type, model, code}> /*candidats écartés*/ }
```

- Quand `substitution` existe, `request` porte la cible effective
  (`agent_type`, `model`, `effort`) et un `fallback` vide.
  `canonical` (clé de rejeu) garde la demande d'origine avec `fallback`.
- Aucun nouvel état Lineage. La vue de tâche ajoute `substitution` (additif).
  Les tâches de cette session ne portent aucun champ d'attente de quota : la
  reprise sur quota est la session 160 (`spec.md`).

## Requête `Delegate` (`bridget-transport/src/protocol.rs`)

| Champ | Type | Sérialisation |
|---|---|---|
| `fallback` | `Vec<FallbackTarget>` | omis si vide |

`FallbackTarget { agent_type: String, model: String, effort: Option<String> }`.
Les octets canoniques d'un appel 150 sont inchangés.

## Verdict de cible (code, pas de stockage)

`resolve_target(...) -> Result<ResolvedTarget, String>` où `ResolvedTarget` =
`(registre épinglé, instantané de droits, posture effective)`. Le `Err` porte
un code. La liste fermée « indisponibilité » (FR-013) décide si un code autorise
un remplacement.

## Entrée du catalogue (additif)

Voir `contracts/delegation157.md`.
