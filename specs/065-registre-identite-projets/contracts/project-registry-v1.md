# Contrat public v1: registre de projets

## Principes

- Contrat versionné et fermé aux champs inconnus.
- `project_id` opaque: aucun consommateur ne le parse pour déduire un chemin.
- Les chemins n'apparaissent que dans les commandes et réponses techniques
  Bridget autorisées.
- Les commandes de mutation sont locales et idempotentes.

## Direction, négociation et autorisation

`ProjectRegistryRequest` et `ProjectRegistryOutcome` sont des variantes
dédiées du protocole local, orientées Maicie vers Bridget puis Bridget vers
Maicie. Elles ne réutilisent pas `ServiceRequest`/`ServiceResponse`, dont la
direction existante est Bridget vers Maicie.

Avant toute mutation, la connexion doit annoncer le rôle `service`, puis
`ServiceHello.service="maicie"`, négocier la capability exacte
`project_registry_v1` et présenter un UID pair égal à l'UID du daemon sur
l'hôte Linux. Une capability ou version inconnue, un rôle différent, une
connexion sans preuve locale ou une route UI/MCP/distante est refusée avant
décodage métier. Chaque résultat reprend `command_id` et `contract_version`;
une seule réponse terminale est durable par enveloppe.

## ProjectBindRequest

```json
{
  "contract_version": 1,
  "command_id": "opaque-command-id",
  "issued_at": 1787997600,
  "deadline_at": 1787998200,
  "project_id": "opaque-project-id",
  "requested_root": "/srv/projects/example",
  "backend": "host"
}
```

## ProjectBindOutcome

Succès:

```json
{
  "contract_version": 1,
  "command_id": "opaque-command-id",
  "project_id": "opaque-project-id",
  "status": "active",
  "binding_generation": 1,
  "backend": "host",
  "observed_at": 1787997601
}
```

Refus:

```json
{
  "contract_version": 1,
  "command_id": "opaque-command-id",
  "project_id": "opaque-project-id",
  "status": "binding_failed",
  "reason": "root_outside_allowed_prefixes",
  "observed_at": 1787997601
}
```

Collision convergente après canonicalisation:

```json
{
  "contract_version": 1,
  "command_id": "opaque-command-id",
  "project_id": "proposed-project-id",
  "status": "registration_conflict",
  "reason": "root_already_bound",
  "existing_project_id": "opaque-existing-project-id",
  "existing_binding_generation": 4,
  "observed_at": 1787997601
}
```

`existing_project_id` est une référence opaque. Maicie peut rattacher la
commande à une identité qu'elle possède déjà ou reconstruire la convergence
d'une liaison gagnante après crash. Elle ne déduit jamais un chemin de cette
valeur. L'identité proposée perdante ne devient pas active.

## Ensemble fermé de raisons

- `invalid_contract`
- `invalid_project_id`
- `invalid_absolute_root`
- `root_missing`
- `root_not_directory`
- `root_outside_allowed_prefixes`
- `root_too_broad`
- `root_already_bound`
- `project_already_bound_elsewhere`
- `rebind_required`
- `project_disabled`
- `envelope_mismatch`
- `idempotency_expired`
- `store_unavailable`
- `registration_conflict`
- `project_registry_capability_missing`
- `project_registry_version_unsupported`
- `local_operator_required`
- `peer_uid_mismatch`
- `project_root_policy_unavailable`
- `project_root_policy_invalid`
- `project_root_policy_permissions_invalid`

## Exemples figés pour les tests

Les exemples suivants utilisent uniquement des identifiants et racines fixture.
Ils fixent la forme des requêtes et les raisons structurées attendues, sans
imposer de chemin de production.

| Cas | Entrée significative | Outcome attendu | Exigences vérifiées |
|---|---|---|---|
| Liaison initiale | racine absolue existante sous un préfixe autorisé, `backend=host` | `active`, génération 1 | FR-001, FR-004, FR-007, FR-023 |
| Rejeu strict | même `command_id` et mêmes octets | outcome durable initial, sans seconde liaison | FR-002, FR-006, NFR-001 |
| Chemin relatif | `requested_root="projet"` | `invalid_absolute_root` avant écriture | FR-007 |
| Racine absente | racine absolue inexistante | `root_missing` avant écriture | FR-007 |
| Préfixe refusé | racine hors politique | `root_outside_allowed_prefixes` | FR-007, FR-027 |
| Racine trop large | `/` ou `/home` après canonicalisation | `root_too_broad` | FR-007, FR-027 |
| Collision canonique | seconde identité pour la même racine canonique | `registration_conflict`, identité et génération gagnantes | FR-008, FR-025 |
| Politique absente | document de politique introuvable | `project_root_policy_unavailable` | FR-027 |
| Politique invalide | version, contenu ou racines invalides | `project_root_policy_invalid` | FR-027 |
| Permissions de politique | propriétaire, groupe ou mode non conformes | `project_root_policy_permissions_invalid` | FR-027, NFR-003 |

Les cas `rebind`, `disable` et rapprochement confirmé produisent exactement un
`ProjectAuditEvent` transactionnel. Leur rejeu strict renvoie l'outcome durable
sans créer de second audit event. Les refus sans mutation n'émettent pas de
faux événement de changement d'état.

## Administration locale

`ProjectAdminRequest` et `ProjectAdminOutcome` sont des variantes dédiées du
même rôle Service négocié que l'enregistrement. Les opérations fermées sont
`list`, `status`, `rebind`, `disable` et `review_project_reconcile`.

- `list` et `status` sont des lectures explicites, sans création d'identité;
- `rebind`, `disable` et `review_project_reconcile` vérifient la policy de
  racines et l'UID pair local avant toute mutation;
- `rebind` et `disable` mémorisent leur issue par `command_id` et n'écrivent
  un audit que lors de l'effet effectif;
- `review_project_reconcile` est déclenché seulement par
  `review-project reconcile --confirm`. Le `--dry-run` est une
  prévisualisation pure. Une confirmation réussie porte un audit durable, y
  compris lorsque la racine est déjà la bonne;
- toute projection publique exclut `canonical_root`, les contenus de dépôt et
  les secrets. L'absence d'observation Bridget est `unavailable`, jamais
  `active` déduit par Maicie.

## ProjectStatusProjection

La projection rend `project_id`, état Maicie, état Bridget, backend, génération,
fraîcheur, dernière raison et prochaine action. Si Bridget est indisponible,
l'état technique vaut `unavailable`; Maicie n'invente pas `active` ou
`binding_failed`.

## Audit durable des mutations

Chaque succès de `register`, `rebind`, `disable` ou réconciliation locale
produit exactement un `ProjectAuditEvent` dans la même transaction que la
mutation de `ProjectBinding`. L'événement porte au minimum `audit_event_id`,
`command_id`, `project_id`, l'opération, `binding_generation`, l'issue, la
référence de racine précédente si elle existe et `observed_at`.

`audit_event_id` est déterministe pour une commande donnée. Le rejeu exact
d'une commande retourne son résultat durable sans créer un second événement.
Un refus qui ne modifie aucun état ne produit pas d'événement mensonger de
changement d'état. Aucun chemin canonique complet, secret ou contenu fournisseur
n'est exposé dans l'audit.

## Compatibilité

- Les contrats existants acceptent un `project_id` optionnel.
- L'absence signifie `unregistered`, jamais projet courant implicite.
- `domain` reste un champ distinct et ne peut satisfaire une exigence de
  `project_id`.
