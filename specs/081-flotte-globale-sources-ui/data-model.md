# Modèle de données - SPEC-081

## Structures privées lues par Desktop

| Structure | Champs consommés | Champs éliminés |
|---|---|---|
| `RemoteUiSnapshotV1` | agents et faits d'exécution publics | champs non requis par une carte |
| `RemoteProjectsV1` | `project_id`, `display_name`, `state` | `canonical_path` obligatoirement |
| Session connectée | `profile_id`, libellé, état, port local, endpoint | endpoint et jeton avant sérialisation Tauri |

## Contrat public

```text
DesktopFleetSnapshotV1
  version: 1
  sources: DesktopFleetSourceV1[]
  agents: DesktopFleetAgentV1[]

DesktopFleetSourceV1
  source_id: string
  label: string
  kind: ssh | local
  connection_state: connected | reconnecting | failed | disconnected
  projects: DesktopProjectV1[]
  error: unavailable | incompatible | null

DesktopProjectV1
  project_id: string
  display_name: string
  state: active | disabled | path_missing | pending_binding | binding_failed | unregistered

DesktopFleetAgentV1
  key: source_id + ":" + agent_name
  source_id: string
  source_label: string
  agent_name: string
  display_name: string
  project_id: string | null
  project_name: string | null
  state: string
  wait_state: string | null
  alerts: string[]
  last_message_at: integer | null
  unread: integer
  is_coordinator: boolean
```

La clé n'est pas une adresse réseau. L'ouverture reçoit toujours `source_id` et `agent_name` séparément.

## Préférences de présentation locales

```text
FleetPresentationPreferencesV1
  pinned_agent_keys: string[]
  unpinned_coordinator_keys: string[]
  sort_criteria: FleetSortCriterionV1[]
  collapsed_group_keys: string[]

FleetSortCriterionV1
  field: source | project | state | activity | name
  direction: ascending | descending
```

- Les filtres sont temporaires et ne modifient aucune source.
- Les épingles et tris sont locaux à Desktop.
- Un coordinateur explicite est en tête sauf exclusion locale explicite.
- Les clés composites empêchent de confondre des homonymes.

## Ordres

| Critère | Ascendant | Descendant |
|---|---|---|
| Source | A vers Z | Z vers A |
| Projet | A vers Z, sans projet dernier | inverse |
| État | à traiter : attente utilisateur, bloqué, alerte, actif, inactif | inverse |
| Activité | plus récente | plus ancienne |
| Nom | A vers Z | Z vers A |

« À traiter » dérive seulement de `wait_state` et `alerts`, pas d'une priorité supposée.
