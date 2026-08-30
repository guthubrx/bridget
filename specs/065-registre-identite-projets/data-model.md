# Modèle de données: Registre et identité des projets

## ProjectIdentity - autorité Maicie

| Champ | Type logique | Règle |
|---|---|---|
| `project_id` | identifiant opaque | stable, non dérivé, immuable |
| `display_name` | texte borné | modifiable, non unique |
| `status` | `pending_binding`, `active`, `registration_conflict`, `disabled` | transition explicite |
| `created_at` | instant UTC | immuable |
| `updated_at` | instant UTC | monotone |
| `registration_command_id` | identifiant | clé du premier enregistrement |

Relations:

- 1 projet `active` vers N objectifs et délégations Maicie.
- 1 projet vers 0 ou 1 état de liaison Bridget projeté.
- Une identité n'incorpore ni chemin hôte, ni backend, ni secret.
- Une identité `pending_binding` ou `registration_conflict` n'est jamais
  référençable par un objectif, une délégation ou une exécution.

## ProjectBinding - autorité Bridget

| Champ | Type logique | Règle |
|---|---|---|
| `project_id` | référence opaque | clé primaire |
| `canonical_root` | chemin absolu canonique | unique parmi les liaisons actives |
| `backend` | enum | `host` uniquement en SPEC-065 |
| `state` | enum technique | voir transitions |
| `generation` | entier | incrémenté à chaque rebind |
| `bound_at` | instant UTC | preuve de liaison |
| `updated_at` | instant UTC | dernière transition |
| `last_reason` | raison structurée optionnelle | aucun texte de dépôt |

## ProjectRootPolicy - autorité de configuration Bridget

Document hôte v1 chargé depuis un chemin absolu explicitement fourni au
daemon. Il contient une liste non vide de racines canoniques admises, une
version et un digest canonique. Son fichier doit appartenir à l'UID du daemon
et ne pas être modifiable par le groupe ou les autres comptes. `/`, `/home`,
`/Users` et le home complet du daemon sont refusés comme racines trop larges.
L'absence ou l'invalidité de cette politique interdit uniquement les mutations
du registre et ne change pas le lancement host historique non enregistré.

## ProjectRegistrationCommand

| Champ | Règle |
|---|---|
| `command_id` | stable et unique dans Maicie |
| `canonical_payload` | octets conservés pour détecter les divergences |
| `proposed_project_id` | alloué une fois, non actif avant issue Bridget |
| `resolved_project_id` | identité gagnante après liaison ou collision |
| `requested_root` | absolu, pas encore autorité canonique |
| `state` | `prepared`, `binding`, `bound`, `failed`, `expired` |
| `retry_until` | borne absolue |

## ProjectBindingAttempt

Chaque tentative conserve `command_id`, `project_id`, racine demandée,
génération, instant, issue et raison. Un rejeu exact met à jour l'enveloppe de
tentative, jamais l'identité ni la racine déjà acceptée.

## ProjectReference

Référence optionnelle composée de `project_id` et de `binding_generation`.
Les deux valeurs sont portées sans redéduction par les délégations,
soumissions, `SpawnOrder`, `SpawnLease`, `DesiredEquipier`, générations
d'agent, snapshots d'exécution, références et projections Maicie, ordres
persistés, curseurs de reprise, `AgentLinkRecord`, `AgentLinkEvent`,
`DelegatedRuntimeEventRecord` et `DelegatedRuntimeEventFrame`. Elle ne
transporte jamais le chemin hôte vers Maicie. Les enregistrements historiques
utilisent `None`, jamais une valeur inférée depuis `cwd`, `domain`, une instance
ou un execution_id.

## ProjectAuditEvent - autorité Bridget

| Champ | Règle |
|---|---|
| `audit_event_id` | identifiant déterministe et unique |
| `command_id` | commande locale ayant produit l'effet |
| `project_id` | identité opaque concernée |
| `operation` | `register`, `rebind`, `disable`, `review_project_reconcile` |
| `binding_generation` | génération résultante ou observée |
| `outcome` | issue fermée, sans texte libre sensible |
| `previous_root_reference` | empreinte ou référence d'audit privée, jamais contenu du dépôt |
| `observed_at` | instant UTC |

L'événement est écrit dans la même transaction Bridget que la mutation
effective. Son identifiant est déterministe sur `command_id`, opération et
génération, de sorte qu'un rejeu exact ne crée pas un second événement. Un
refus sans mutation peut être observé dans les logs bornés mais ne produit pas
un faux événement de changement d'état.

## Transitions

### Identité Maicie

```text
absent -> pending_binding -> active -> disabled
                  |           ^         |
                  |           +---------+  réactivation explicite future
                  +-> registration_conflict
```

### Liaison Bridget

```text
absent -> pending_binding -> active -> disabled
                 |            |
                 |            +-> path_missing -> active par rebind
                 +-> binding_failed -> pending_binding par rejeu
```

Règles:

- `active -> active` avec même racine est un rejeu sans effet.
- `active -> active` avec autre racine exige un `rebind` explicite.
- Deux intentions concurrentes pour la même racine canonique produisent une
  seule identité active; la perdante converge vers `resolved_project_id` ou
  reste en `registration_conflict` durable.
- `disabled` n'arrête aucun agent existant.
- `path_missing` est une observation technique, pas une suppression.

## Index et complexité

- Index unique sur `project_id` des deux côtés.
- Index unique partiel sur `canonical_root` pour les liaisons non désactivées.
- Index sur `resolved_project_id` et interdiction applicative de référencer un
  projet qui n'est pas `active`.
- Index sur `command_id` et sur les outboxes non terminales.
- Les recherches par projet ou racine sont O(log n) au pire avec SQLite.
