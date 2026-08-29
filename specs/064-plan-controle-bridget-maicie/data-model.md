# Modèle de données : Plan de contrôle Bridget et Maicie

## Principes

- Chaque entité a une autorité unique.
- Les références croisées sont opaques et ne donnent aucun droit d'écriture sur
  la base de l'autre composant.
- Les transitions durables utilisent comparaison d'état ou génération pour
  empêcher une sortie tardive de modifier une nouvelle exécution.
- Les identifiants fournisseur ne sont jamais réutilisés comme identifiants
  Bridget.

## Entité WorkSubmission

Autorité : Bridget.

| Champ | Rôle |
|---|---|
| `submission_id` | Identité durable du travail logique |
| `origin` | Humain, agent, routine ou système |
| `intent` | QueueOnly, TriggerTurn, SteerCurrent, InterruptAndStart ou ControlOnly |
| `priority_class` | Classe déclarée, FIFO dans une même classe |
| `objective_id` | Référence Maicie optionnelle |
| `delegation_id` | Référence Maicie optionnelle |
| `requested_agent` | Destinataire logique |
| `pinned_instance_id` | Instance sélectionnée lorsque l'admission la fixe |
| `generation` | Génération attendue de l'instance |
| `payload_digest` | Empreinte du contenu canonique |
| `delivery_deadline_at` | Borne de remise |
| `execution_deadline_at` | Borne d'exécution optionnelle |
| `fallback_policy` | Repli autorisé ou refus requis |
| `created_at` | Date d'acceptation durable |
| `state` | État d'admission et de file |

### États

```text
prepared -> admitted -> queued -> bound_to_execution
prepared -> rejected
admitted -> cancelled
```

## Entité SendDelivery

Autorité : Bridget. Entité existante conservée.

Ajouts ou corrélations :

- `submission_id` optionnel pendant migration ;
- `execution_id` optionnel ;
- phase détaillée de réception et visibilité ;
- références fournisseur observées ;
- raison terminale structurée.

La livraison continue à épingler instance, génération, octets exacts et horizon
d'idempotence.

## Entité Execution

Autorité : Bridget.

| Champ | Rôle |
|---|---|
| `execution_id` | Identité durable du cycle runtime |
| `submission_id` | Travail propriétaire |
| `agent_instance_id` | Instance exécutante |
| `generation` | Génération exécutante |
| `state` | État runtime canonique |
| `reason_code` | Motif structuré de la dernière transition |
| `started_at` | Début de l'exécution |
| `last_progress_at` | Dernière preuve de progrès |
| `ended_at` | Fin terminale éventuelle |
| `provider_binding_id` | Mapping vers la session fournisseur |
| `revision` | Version de comparaison atomique |

### États et transitions principales

```text
queued -> starting -> running
running -> waiting_approval -> running
running -> waiting_user_input -> running
running -> interrupting -> interrupted
running -> completed
running -> failed
starting -> failed
queued | starting | running -> unreachable
```

Une transition exige l'état et la révision attendus. Une transition terminale
est monotone. Une réconciliation peut ajouter une preuve ou une classification,
mais ne réouvre pas silencieusement l'exécution.

## Entité ProviderBinding

Autorité : Bridget, alimentée par l'adaptateur.

| Champ | Rôle |
|---|---|
| `provider_binding_id` | Identité du mapping |
| `provider_kind` | Codex, Claude, Cursor, Gemini ou autre fournisseur |
| `execution_path` | `codex_app_server`, `claude_stream_json`, `acp`, `tmux` ou autre protocole attesté |
| `binary_path` | Chemin résolu du binaire |
| `binary_version` | Version observée |
| `binary_digest` | Empreinte observée ou issue de release |
| `contract_version` | Contrat de protocole validé |
| `provider_session_id` | Session fournisseur optionnelle |
| `provider_thread_id` | Thread ou conversation optionnel |
| `active_turn_id` | Tour actif optionnel |
| `capabilities_revision` | Révision des capacités |
| `observed_at` | Date de l'observation |

## Entité MessageCorrelation

Autorité : Bridget.

| Champ | Rôle |
|---|---|
| `submission_id` | Travail Bridget |
| `delivery_id` | Livraison Bridget |
| `client_message_id` | Identifiant propagé au fournisseur |
| `provider_item_id` | Identifiant interne de l'item fournisseur |
| `provider_thread_id` | Thread fournisseur |
| `provider_turn_id` | Tour fournisseur |
| `visibility_event` | Type d'événement qui prouve la visibilité |
| `visible_at` | Horodatage de la preuve |

La contrainte de preuve compare `client_message_id`, jamais
`provider_item_id`, sauf contrat fournisseur explicitement différent et
versionné.

## Entité AgentLink

Autorité : Bridget.

| Champ | Rôle |
|---|---|
| `link_id` | Identité du lien durable |
| `parent_instance_id` | Parent propriétaire |
| `child_instance_id` | Enfant |
| `parent_execution_id` | Exécution ayant demandé la création |
| `objective_id` | Objectif optionnel |
| `delegation_id` | Délégation optionnelle |
| `role` | Rôle déclaré |
| `agent_path` | Chemin stable dans la hiérarchie |
| `state` | Reserved, Open, Transferred, Closed ou Orphaned |
| `created_at` | Création |
| `closed_at` | Fin éventuelle |
| `revision` | Comparaison atomique |

### Invariants

- Un enfant possède au plus un lien propriétaire ouvert.
- La profondeur et les quotas sont vérifiés avant réservation.
- Une réservation abandonnée ne laisse pas de lien ouvert.
- La disparition du parent applique une politique durable, pas une suppression.

## Entité ProviderCapabilities

Autorité : adaptateur, persistée par Bridget.

Capacités initiales :

- démarrer un thread ;
- reprendre un thread ;
- bifurquer un thread ;
- démarrer un tour ;
- piloter le tour courant ;
- interrompre un tour ;
- corréler un client message ;
- exposer le cycle de vie des items ;
- exposer les demandes d'autorisation ;
- exposer l'usage ;
- conserver un historique durable.

Chaque capacité possède source, confiance, version et date d'observation.

## Entité ExecutionReference Maicie

Autorité : Maicie.

| Champ | Rôle |
|---|---|
| `delegation_id` | Délégation Maicie propriétaire |
| `submission_id` | Référence opaque Bridget |
| `execution_id` | Référence opaque Bridget |
| `agent_instance_id` | Référence opaque Bridget |
| `provider_kind` | Information d'affichage |
| `provider_session_id` | Référence opaque optionnelle |
| `provider_turn_id` | Référence opaque optionnelle |
| `bound_at` | Date de liaison |

Cette entité ne permet aucune mutation de l'exécution. Elle sert à demander et
corréler des faits Bridget.

## Entité ExecutionProjection Maicie

Autorité : Maicie pour sa copie observée, Bridget pour les faits sources.

| Champ | Rôle |
|---|---|
| `execution_id` | Référence Bridget |
| `runtime_state` | Dernier état observé |
| `waiting_reason` | Raison d'attente optionnelle |
| `last_progress_at` | Dernier progrès observé |
| `observation_cursor` | Curseur Bridget consommé |
| `freshness` | Fresh, Gap, Ended ou Unavailable |
| `observed_at` | Date locale d'observation |
| `source_generation` | Génération de la source |

Cette projection n'entraîne aucune transition d'objectif ou de délégation sans
règle métier et décision explicites.

## Entité MissionExecutionLimits

Autorité : Maicie.

Cette entité exprime les limites attachées à un objectif ou une délégation.
Elle est transmise comme contrainte demandée et ne produit aucun effet runtime
avant admission par Bridget.

| Champ | Rôle |
|---|---|
| `limits_id` | Identité Maicie |
| `objective_id` | Objectif propriétaire |
| `delegation_id` | Délégation optionnelle |
| `max_wall_time` | Limite temporelle demandée |
| `max_usage` | Limite d'usage demandée |
| `max_children` | Limite de descendants directs demandée |
| `max_depth` | Profondeur demandée |
| `continuation_mode` | Disabled, HumanOnly ou PolicyDriven |
| `revision` | Révision Maicie de la contrainte |

## Entité ExecutionPolicy

Autorité : Bridget.

Cette entité représente la politique technique effectivement admise. Elle
référence éventuellement les limites Maicie sources, mais Bridget reste seule
responsable de l'admission, de l'application et du refus technique.

| Champ | Rôle |
|---|---|
| `policy_id` | Identité |
| `source_limits_id` | Référence opaque Maicie optionnelle |
| `source_limits_revision` | Révision Maicie admise |
| `max_wall_time` | Limite temporelle optionnelle |
| `max_usage` | Limite d'usage optionnelle |
| `max_children` | Nombre maximal de descendants directs |
| `max_depth` | Profondeur maximale |
| `max_retries` | Reprises maximales |
| `approval_profile` | Profil d'autorisation technique |
| `continuation_mode` | Disabled, HumanOnly ou PolicyDriven |
| `fallback_policy` | Replis permis |

## Relations

```text
Objective 1 -> N Delegation
Delegation 1 -> N WorkSubmission
Delegation 1 -> N MissionExecutionLimits
WorkSubmission 1 -> N SendDelivery
WorkSubmission 1 -> N Execution, séquentielles sauf politique explicite
WorkSubmission N -> 1 ExecutionPolicy admise
Execution 1 -> 1 ProviderBinding actif
Execution 1 -> N MessageCorrelation
Execution 1 -> N AgentLink créés
AgentLink N -> 1 parent et 1 enfant
Delegation 1 -> N ExecutionReference
ExecutionReference 1 -> 1 ExecutionProjection courante
```

## Index et complexité attendue

- lookup livraison par identifiant et portée : O(log n) ou index équivalent ;
- prochain travail admissible par agent et priorité : index composite, pas de
  scan complet répété ;
- descendants directs : index parent ;
- ascendance : profondeur bornée, détection de cycle avant ouverture ;
- événements par exécution : index exécution et séquence ;
- projections Maicie : clé délégation et exécution.

Toute dégradation de classe dans un chemin chaud doit être documentée et
mesurée avant acceptation.
