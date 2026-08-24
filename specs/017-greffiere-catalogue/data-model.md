# Modèle de données — session 017

## Journal de catalogue v1

Le fichier déclaré est l'autorité unique. Il contient une suite de lignes JSON
fermées, chacune terminée par LF. Un en-tête éventuel n'est pas une seconde
source de vérité : `registre list` est la vue humaine du journal.

### `add`

| Champ | Règle |
|---|---|
| `v` | vaut exactement `1` |
| `kind` | vaut exactement `add` |
| `id` | identifiant stable unique |
| `date` | horodatage RFC 3339 avec fuseau explicite |
| `mission_source` | `{kind, id}` ; kind fermé `mission|incident|review|gate` |
| `severity` | enum fermé `blocker|major|minor|info` ; déclarée par l'humain **ou** dérivée du contrat FR-1711 pour un fait couvert |
| `recurrence_of` | absent ou identifiant de constat existant |
| `text` | texte verbatim, jamais classé ni réécrit |

### Contrat de transcription (FR-1711)

| Type de fait | Source | Sévérité |
|---|---|---|
| `gate_failed` | `gate` + `failed=true` | `blocker` |
| `review_amender` | `review` | `major` |

Hors table → `pending_qualification`. Identité : `id = "{type}:{source_id}"`.

### `transition`

| Champ | Règle |
|---|---|
| `v` | vaut exactement `1` |
| `kind` | vaut exactement `transition` |
| `constat_id` | identifiant de constat existant |
| `from` / `to` | uniquement `open` vers `delivered` |
| `objective_id` | identifiant exact porté par le lien de délégation |
| `observed_at` | horodatage du fait de clôture attesté |
| `trigger` | vaut exactement `objective_closed` |

## Lien d'arbitrage

Une délégation durable qui est créée depuis un constat porte `constat_id` dans
son entrée canonique. Lorsque l'objectif est créé ou explicitement choisi, le
couple `(constat_id, objective_id)` est journalisé dans la même décision. Ce
lien ne se déduit jamais d'un texte, d'un nom ou d'une recherche.

## Idempotence et append atomique

Le writer verrouille exclusivement le fichier régulier déclaré avant de lire
les identifiants existants. Pour `add`, mêmes `id` et octets canoniques donnent
un no-op ; mêmes `id` et octets différents donnent un refus sans mutation.
Après validation, il append la ligne complète via `O_APPEND`, appelle la
synchronisation de données, puis relâche le verrou. Il n'emploie ni
temporaire+rename ni réécriture du journal, pour ne jamais perdre un append
concurrent.

## Projection

La projection réduit les entrées dans l'ordre du journal. Un constat historique
incomplet reste `pending_qualification` hors de la liste ouverte, mais compte
dans `P`. Les constats ouverts sont triés selon les seuls champs déclarés :
sévérité, récurrence, source gate ratée, date puis identifiant.

Une transition peut être demandée par un événement de clôture reçu ou par une
réconciliation qui lit l'état durable attesté du même `objective_id`. Ces deux
voies réduisent à la même identité de transition et n'autorisent aucune
déduction depuis une horloge, un silence ou un texte.
