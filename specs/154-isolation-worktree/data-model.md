# Modèle de données - Session154

Aucune table nouvelle. Aucune migration. L'état vit dans la charge JSON de la
tâche (`Task`, `delegation.rs`), comme `reply_timeout` en session150.

## Demande (`bridget-transport/src/protocol.rs`, variante `Delegate`)

| Champ | Type | Défaut si absent | Rôle |
|---|---|---|---|
| `workspace` | `Option<WorkspaceRequest>` | absent = `shared` implicite | Stratégie d'espace de travail |

`WorkspaceRequest` est une énumération étiquetée par `strategy`, avec
`deny_unknown_fields` :

| `strategy` | Champs | Notes |
|---|---|---|
| `shared` | aucun | comportement 150 |
| `worktree` | `base_ref: Option<String>`, `branch: Option<String>` | `base_ref` défaut `HEAD` ; `branch` défaut `bridget/<task8>` |

Le champ porte `skip_serializing_if = "Option::is_none"`. Il est placé après
`reply_timeout` (session150). Il entre dans l'enveloppe idempotente.

Validation syntaxique (avant tout Git) :

- `branch` : 1 à 100 caractères, `[A-Za-z0-9._/-]`, premier caractère
  alphanumérique, sans `..`, `//`, `@{`, fin `/` ou `.lock`. Puis
  `git check-ref-format --branch` confirme.
- `base_ref` : 1 à 200 caractères, sans espace ni caractère de contrôle, premier
  caractère différent de `-`.

## Réservation (mémoire, entre les deux passes d'admission)

| Champ | Rôle |
|---|---|
| `task_id` | décidé à la première passe, réutilisé à la seconde |
| `toplevel`, `path`, `offset` | racine du dépôt, racine du worktree, décalage du dossier du parent |
| `branch`, `base_commit` | branche et base figées |

Non stockée. Après un crash entre les deux passes, il reste au plus un
worktree et une branche inoffensifs, nommés dans le journal du daemon.

## Enregistrement durable (`Task.workspace`)

Champ ajouté avec `#[serde(default)]` : un payload 150 se relit avec `None`.

| Champ | Type | Rôle |
|---|---|---|
| `strategy` | `"worktree"` | seules les tâches en worktree ont un enregistrement |
| `state` | `"active"`, `"kept"` | `active` : l'enfant peut écrire ; `kept` : rapport final lu |
| `toplevel` | chemin canonique | racine du dépôt, pour `git -C` |
| `path` | chemin canonique | racine du worktree |
| `branch` | texte | branche créée ou choisie |
| `base_commit` | texte (40 hex) | base figée à l'admission |
| `head_commit` | `Option<texte>` | tête lue à la fin |
| `branch_commit` | `Option<texte>` | commit pointé par la branche à la fin |
| `head_on_branch` | `Option<bool>` | tête rattachée à la branche attendue |
| `commit_count` | `Option<u32>` | commits depuis la base jusqu'à la tête |
| `commits` | liste de `{sha, subject}` | 50 au plus, plus récents d'abord |
| `truncated` | booléen | vrai si `commit_count` > 50 |
| `dirty` | `Option<bool>` | modifications ou fichiers non suivis |

Règles :

- `Task.cwd` vaut le dossier de départ de l'enfant : `path` plus le décalage du
  dossier du parent dans le dépôt.
- `visible_changed` (Lineage) ignore `workspace`. Aucun signal, aucun nouvel
  état. La vue Lineage ne change pas.
- `state` passe `active` → `kept`, une seule fois, quand le rapport final est
  stocké. Jamais de retour vers `active`.
- `cleanup_done` devient vrai après `kept`.
- Aucun état « retiré » : Bridget ne supprime jamais.

## Cycle de vie

```
admission ──► active ──(enfant arrêté, mission finie, échouée ou annulée)──► kept
```

## Exécuteur Git (pas de stockage)

Entrée : dossier, sous-commande de la liste blanche, arguments, délai.
Sortie : code, texte borné à 1 Mio. Aucun état.

## Reçu `shared_writers`

Clé ajoutée au reçu de `bridget_delegate` seulement si la valeur est au moins 1
et la stratégie est `shared` en développement. Calculée à l'admission, non
stockée.
