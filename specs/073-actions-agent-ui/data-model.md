# Modèle de données - SPEC-073

## Projection d'agent UI

Extension de l'entité existante, sans stockage supplémentaire.

| Champ | Type | Source | Règle |
|---|---|---|---|
| `name` | chaîne | `AgentInfo.name` | nom validé existant |
| `state` | état public | présence existante | `stopped` interdit l'action |
| `persistent` | booléen optionnel | `AgentInfo.persistent` | valeur présente = gestion attestée |
| `turn_state` | chaîne optionnelle | projection d'exécution | activité en cours = avertissement |
| identité runtime | faits existants | SPEC-071 | aucune inférence par nom |

### Éligibilité calculée

```text
eligible = persistent est présent ET state n'est pas stopped
busy_warning = turn_state représente un tour actif ou en attente fournisseur
```

Cet état reste dérivé dans l'interface et n'est pas stocké.

## Demande de décommissionnement

| Champ | Type | Validation |
|---|---|---|
| `version` | entier | exactement `1` |
| `name` | chaîne | nom d'agent valide, non vide |
| `command_id` | chaîne | identifiant de corrélation non vide, unique pour la confirmation |

La demande n'emporte ni PID, ni signal, ni chemin système.

## Verdict de décommissionnement

| Champ | Type | Valeurs |
|---|---|---|
| `version` | entier | `1` |
| `name` | chaîne | agent visé |
| `command_id` | chaîne | corrélation de la demande |
| `outcome` | enum | `stopped`, `stopped_forced` |
| `survivors_killed` | entier optionnel | seulement pour `stopped_forced` |

Les refus utilisent une enveloppe d'erreur versionnée avec un code stable :
`agent_not_managed`, `agent_not_found`, `agent_stopped`, `stop_timeout` ou
`daemon_unavailable`.

## État client temporaire

| État | Signification | Transition autorisée |
|---|---|---|
| `closed` | aucun panneau ouvert | vers `panel_open` |
| `panel_open` | identité visible | vers `confirming` ou `closed` |
| `confirming` | confirmation modale | vers `panel_open` ou `submitting` |
| `submitting` | ordre envoyé, verdict attendu | vers `succeeded` ou `failed` |
| `succeeded` | daemon a confirmé l'arrêt | fermeture puis snapshot réel |
| `failed` | refus ou panne affiché | retour à `panel_open` |

Un seul agent peut porter `submitting` dans la fiche globale. Un second clic sur
la même confirmation est ignoré tant que le verdict n'est pas terminal. Le
navigateur ne rejoue pas automatiquement une demande dont l'issue est inconnue.

## Transitions de présence

```text
connected/busy --StopOutcome::Stopped--------> stopped
connected/busy --StopOutcome::StoppedForced--> stopped
connected/busy --refus/timeout/panne----------> inchangé
stopped        --nouvelle demande-------------> refus avant envoi
non géré       --nouvelle demande-------------> refus avant envoi
```

Le navigateur ne produit jamais lui-même la transition vers `stopped` ; il
attend la prochaine projection attestée.
