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
Quota { windows: Vec<{window, used_percent, resets_at, exhausted}>,
        exhausted: bool, limit_reached: bool, resets_at: Option<i64>, observed_at: i64 }
```

`exhausted(fenêtre, now)` : `resets_at > now` ET (source Claude `stream-json` avec
`status == "rejected"` OU `used_percent >= 100`). Un statut Codex non
`available` (global, recopié sur les deux fenêtres) ne rend aucune fenêtre
épuisée : il lève `limit_reached`. `Quota.exhausted` = une fenêtre au moins.
`Quota.resets_at` = le plus tardif des retours des fenêtres épuisées. Le
statut `unknown` du hook ne prouve rien seul ; used_percent >= 100 et
retour futur épuise, toute source, statusline comprise.

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

## Précisions retenues de la revue157-r1 — appliquées au GO157

GO157 le2026-10-12 après livraison et activation155/156. Base source156
cf498755dc871b51eb8c0add5f8a882d67651d2a, base Git avec clôture documentaire
8b5b313694cf05eabfe6a86a970b96f03b015f49. Les références150/1f2c661d
restent des preuves historiques ; les fonctions sont revalidées sur cette base.

Le verdict conserve l’ordre type/modèle/effort, préflight, politique, gardes
liées aux options de l’appel, quota. Workspace et, après livraison155,
child_requests sont non remplaçables pour la cible demandée ; un candidat
explicitement listé refusé est noté dans tried. Aucun gain de droits ni
contournement des gardes du parent. En worktree, sélection unique en Probe,
candidat et définition épinglés transmis à Reserve puis Commit. Commit
revalide ce même candidat dans le dossier enfant réel ; si une garde échoue,
refuser sans nouvelle sélection et conserver les diagnostics de chemin154.
Les options immuables et canonical original restent distincts de request effectif.

Quota : huit fenêtres ; purger les passées avant l’écriture. Une neuvième clé
évince le plus petit observed_at, égalité départagée par clé lexicale.
unknown du hook n’est pas une preuve seul ; pourcentage>=100 avec retour
futur épuise toute source. provider_limit_reached_window_unknown est l’unique
contrainte informative possible si can_run, hors famille verdict et jamais
rendue comme refus d’admission. Aucun nouveau champ public.

Refus enrichi : Err(String) sous forme code:JSON contenant message/detail.
Étendre le seul failure existant ; les comparaisons de listes portent sur
le code canonique. JSON non conforme : refus fermé, aucune valeur inventée.
Les anciens codes nus et préfixes restent compatibles. Aucun nouveau type,
helper, framework, table, outil, ID ou scénario autonome. Les tests prévus
T004/T016/T017/T018 couvrent ces précisions. Les26 tâches restent ouvertes.
