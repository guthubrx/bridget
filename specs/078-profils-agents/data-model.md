# Modèle de données - SPEC-078 Profils d'agents et notifications

## Portée de persistence

Le ledger SQLite du daemon devient l'autorité de profil et d'attention. Le
fichier `fleet.json` demeure réservé au cycle de vie des agents gérés. Les
noms techniques présents dans le ledger restent intacts.

Tous les horodatages sont UTC RFC 3339. Les UUID sont générés côté daemon. Ils
sont opaques pour l'UI: ils peuvent circuler dans le contrat mais ne sont jamais
rendus comme texte, attribut DOM visible, libellé ou message d'erreur.

## Entités partagées du serveur

### AgentIdentity

| Champ | Type | Règle |
|---|---|---|
| agent_id | UUID | primaire, stable, non rendu |
| current_routing_name | texte | unique, adresse active connue |
| created_at | timestamp | défini à l'import ou création |
| updated_at | timestamp | défini à toute mutation de binding |

Une identité existe aussi pour un agent historique, arrêté ou externe. Elle ne
porte ni état de flotte ni configuration fournisseur.

### AgentRoutingAlias

| Champ | Type | Règle |
|---|---|---|
| routing_name | texte | unique, normalisé comme le routage existant |
| agent_id | UUID | référence AgentIdentity |
| is_current | booléen | un seul alias actif par identité |
| observed_at | timestamp | dernière observation ou mutation |

L'alias convertit le nom trouvé dans le ledger ou l'annuaire en identité. Un
ancien alias est conservé. La migration ne tente pas de séparer rétroactivement
plusieurs agents ayant partagé exactement le même ancien nom.

### AgentProfile

| Champ | Type | Règle |
|---|---|---|
| agent_id | UUID | primaire et référence AgentIdentity |
| display_name | texte | obligatoire, normalisé, unique par daemon |
| avatar_shape | enum | forme parmi la palette UI approuvée |
| avatar_color | enum | couleur parmi la palette UI approuvée |
| instructions | texte privé du store daemon | jamais renvoyé dans les snapshots généraux ni les logs |
| instructions_revision | entier positif | incrémenté pour tout changement de texte |
| updated_at | timestamp | dernier verdict d'écriture |
| revision | entier positif | contrôle de concurrence optimiste du profil |

Le texte des instructions est retourné uniquement par la lecture du profil de
l'agent explicitement ouvert, à un client autorisé. Les projections de liste,
watch, messages et événements ne contiennent que son état et sa révision.

### AgentProfileLabel

| Champ | Type | Règle |
|---|---|---|
| agent_id | UUID | référence AgentProfile |
| position | entier | ordre stable de rendu |
| label | texte | 1 à 32 caractères après trim |
| normalized_label | texte | unicité insensible aux espaces et à la casse |

La mutation accepte une liste ou une saisie découpée par virgule et Entrée.
Elle borne le nombre de labels à 12. Les vides et doublons sont rejetés ou
éliminés avant l'écriture atomique selon le verdict explicite du contrat.

### AgentInstructionApplication

| Champ | Type | Règle |
|---|---|---|
| agent_id | UUID | référence AgentProfile |
| provider_spawn_id | texte opaque | corrélation du lancement, non rendu |
| instructions_revision | entier | révision concernée |
| status | enum | pending, applied, unsupported ou failed |
| applied_at | timestamp nullable | présent seulement si applied |
| diagnostic_code | enum nullable | code sûr, sans texte de consigne |

L'état visible est dérivé de la dernière application: `active`, `pending_restart`,
`unsupported` ou `failed`. Une écriture pendant une session active crée
`pending_restart`, sans interrompre le travail ni prétendre modifier le tour.

### AttentionEvent

| Champ | Type | Règle |
|---|---|---|
| event_id | UUID ou ULID | primaire, ordre total stable |
| occurrence_key | texte | unique, déduplication idempotente |
| agent_id | UUID | référence AgentIdentity |
| event_type | enum | human_input_needed, task_completed, terminal_failure |
| source_message_id | texte nullable | lien interne si disponible, non affiché |
| created_at | timestamp | date du fait sémantique |
| summary | texte court | construit côté daemon avec nom affiché, sans consigne ni ID |

Les états de transport, commandes, outils, chunks et journaux routiniers ne
créent jamais cette entité. La réémission d'un même fait conserve son
`occurrence_key` et ne crée pas une seconde entrée.

## État spécifique à un client

### ClientRegistration

| Champ | Type | Règle |
|---|---|---|
| client_id | UUID | créé une fois par navigateur ou installation Desktop |
| client_kind | enum | web ou desktop |
| created_at | timestamp | audit minimal |
| last_seen_at | timestamp | maintenance et diagnostic |

Le client_id n'est pas un compte, un secret ni une identité de routage. Il
permet seulement de séparer les choix d'attention de deux appareils reliés au
même relais. Desktop le conserve dans son `app_data_dir`; Web le conserve dans
son stockage d'origine.

### ClientNotificationPreference

| Champ | Type | Règle |
|---|---|---|
| client_id | UUID | référence ClientRegistration |
| agent_id | UUID | référence AgentIdentity |
| human_input_needed | booléen | défaut false |
| task_completed | booléen | défaut false |
| terminal_failure | booléen | défaut false |
| updated_at | timestamp | verdict confirmé |

Ces préférences sont privées au client. Elles ne changent jamais le profil de
l'agent, le travail, les autres clients ou la génération de l'événement.

### ClientAttentionState

| Champ | Type | Règle |
|---|---|---|
| client_id | UUID | référence ClientRegistration |
| event_id | UUID/ULID | référence AttentionEvent |
| seen_at | timestamp nullable | vide tant que l'élément reste non lu |
| native_notified_at | timestamp nullable | garantit une seule alerte locale |

Le centre d'activité est une projection de `AttentionEvent` et de cet état.
Le badge compte les événements non lus autorisés pour le client. Une permission
native refusée ne modifie pas `seen_at` et ne supprime pas l'événement.

## Contraintes transactionnelles

1. La création d'une identité, du profil initial et de l'alias initial est une
   transaction unique.
2. La mise à jour de profil compare `revision`, valide toutes les valeurs puis
   écrit profil et labels ensemble, ou rien.
3. L'unicité de `display_name` est garantie en SQLite après normalisation. Un
   conflit renvoie un code `display_name_conflict` sans révéler l'identité de
   l'autre agent.
4. L'écriture d'un événement utilise `occurrence_key` et est idempotente.
5. Les préférences et marqueurs de lecture sont séparés de l'événement: deux
   clients n'ont donc jamais le même état de lecture par accident.
6. Une erreur de profil, de migration ou de préférence dégrade la projection
   vers le nom historique sans empêcher la conversation ou le cycle de vie.

## Projection UI

```text
AgentRuntimeRow (nom de routage interne)
        │
        ├── AgentIdentity via alias
        │       └── AgentProfile + labels + application state
        └── messages et état runtime existants
                    │
                    ▼
UiAgentRowV1 { profile_ref, display_name, labels, avatar, instruction_state }
```

`profile_ref` est manipulé uniquement par le code applicatif pour les routes et
la mise en cache. Toute chaîne affichable vient de `display_name` ou d'un
libellé validé.
