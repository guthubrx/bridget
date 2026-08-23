# Modèle de données — Maicie v3

## Frontières de persistance

| Propriétaire | Données | Accès de l'autre couche |
|---|---|---|
| Bridget | présence, livraison, ledger, demandes suivies, timeout | Maicie lit par contrat local public |
| Maicie | objectifs, délégations, décisions, profils, observations normalisées | Bridget n'y accède jamais |

La base Maicie est distincte de `bridget.db`. Les données de transport ne sont
pas copiées : les références Bridget sont des corrélations externes.

## Entités

### ObjectifCoordonné

| Champ | Type logique | Règle |
|---|---|---|
| `id` | UUID | immuable |
| `but` | texte | créé seulement par instruction explicite à Maicie |
| `mode` | `collaboratif` \| `délégué` | décrit la posture, pas un verrou de conversation |
| `état` | voir transition | jugement Maicie, jamais état Bridget |
| `créé_at`, `mis_à_jour_at` | timestamp | auditables |
| `synthèse` | texte optionnel | agrégation factuelle des réponses, modifiable/validable explicitement |
| `décision_en_attente_id` | UUID optionnel | une attente de décision explicite à la fois dans le MVP |

États : `ouvert → en_coordination → à_évaluer → synthétisé → clos`.
`à_évaluer` signifie qu'une réponse ou une erreur existe mais ne vaut pas
clôture automatique. `clos` demande une action Maicie ou humaine explicite.

### Délégation

| Champ | Type logique | Règle |
|---|---|---|
| `id` | UUID | immuable |
| `objectif_id` | UUID | référence vers ObjectifCoordonné |
| `participant` | nom d'agent Bridget ou profil | source explicitée |
| `instruction` | texte | contexte minimal utile |
| `durée` | `courte` \| `normale` \| `longue` | traduit vers un timeout Bridget configuré |
| `coordination_state` | voir transition | aucun état de livraison |
| `raison` | texte | ajout ou retrait traçable |

États : `créée → à_évaluer → terminée` ; `annulée` est une décision explicite.
Une réponse ou issue Bridget fait passer à `à_évaluer`, jamais directement à
`terminée`. Les états et tentatives de livraison appartiennent à l'outbox.

### OutboxDélégation

| Champ | Type logique | Règle |
|---|---|---|
| `message_id` | UUID | clé unique associée à la Délégation, générée avant I/O |
| `delegation_id` | UUID | obligatoire |
| `target` | nom Bridget | enveloppe immuable complète |
| `body_bytes` | bytes/texte exact | jamais reconstruit depuis Objectif/Délégation |
| `reply` | booléen | enveloppe immuable complète |
| `timeout_secs`, `deadline_contractuelle` | valeurs envoyées | enveloppe immuable complète |
| `body_hash` | hash | détecte toute corruption de l'enveloppe |
| `state` | `prepared` \| `outcome_unknown` \| `accepted` \| `rejected` | transitions transactionnelles ; `accepted` et `rejected` sont terminaux |
| `attempted_at` | timestamp optionnel | audit de reprise |
| `retry_until`, `dedup_retained_until` | timestamps | horizon de retry ≤ tombstone Bridget |

Les issues `Rejected`, `EnvelopeMismatch`, `IdempotencyExpired` et
`InvalidIssuedAt` convergent toutes vers `rejected`. En particulier,
`InvalidIssuedAt` ne doit jamais être réécrit en `outcome_unknown` ni rejoué.

Tout état non terminal, y compris `prepared` après crash brutal, commence par
une recherche Bridget par `message_id`. La reprise réémet ensuite exactement
l'enveloppe persistée, jamais une reconstruction. Si le tombstone de
déduplication est expiré, l'issue est `idempotency_expired` et aucun envoi
aveugle n'est permis.

### SnapshotTransport

| Champ | Type logique | Règle |
|---|---|---|
| `message_id` | UUID | référence de délégation |
| `request_state` | état Bridget optionnel | dérivé, non autoritaire Maicie |
| `observed_at` | timestamp | fraîcheur obligatoire |
| `source` | `bridget` \| `acp_subscription` | obligatoire |
| `subscription_id`, `seq` | références optionnelles | exigées pour source ACP |
| `stream_state` | `fresh` \| `gap` \| `ended` \| `unavailable` | interdit toute conclusion métier automatique |

### ObservationRuntime

| Champ | Type logique | Règle |
|---|---|---|
| `id` | UUID | immuable |
| `agent` | nom Bridget | connu au moment de l'observation |
| `source` | `bridget` \| `acp_subscription` | obligatoire |
| `nature` | disponibilité, tour, outil, idle, permission_auto_décidée | vocabulaire fermé MVP |
| `observé_at` | timestamp | fraîcheur explicite |
| `preuve_ref` | référence opaque | `subscription_id` + `seq` ou demande Bridget |
| `stream_state` | `fresh` \| `gap` \| `ended` \| `unavailable` | explicite la complétude |
| `détails` | données minimisées | jamais une interprétation de terminal |

### ProfilÉquipe

| Champ | Type logique | Règle |
|---|---|---|
| `id` | slug stable | indépendant des instances |
| `nom_affiché` | texte | ex. Prospective |
| `capacités` | liste de tags | égalité de tags déclarés, jamais inférées |
| `personnalité` | référence de configuration | explicable à l'utilisateur |
| `outils` | liste déclarative | ne confère pas d'autorisation automatique |
| `spawn_order_ref` | référence déclarative | seul le contrat public session 009 peut l'exécuter |
| `état_activation` | inactif, proposition, approuvé, lancé, connecté | transition auditée |

### DécisionCoordination

| Champ | Type logique | Règle |
|---|---|---|
| `id` | UUID | immuable |
| `objectif_id` | UUID | obligatoire |
| `type` | ajouter_participant, retirer, relancer, réveiller_profil, clôturer | fermé MVP |
| `proposée_par` | humain ou Maicie | visible |
| `statut` | proposée, approuvée, refusée, appliquée | approbation requise pour réveil |
| `motif` | texte | requis si action significative |

### ApprobationActivation

| Champ | Type logique | Règle |
|---|---|---|
| `id` | UUID | mono-usage |
| `objective_id`, `profile_id` | UUID/slug | liés à l'action exacte |
| `profile_hash`, `context_hash` | hash | revalidation TOCTOU avant `SpawnOrder` |
| `context_scope`, `parameters` | données minimales | exactement ce qui a été approuvé |
| `actor` | `local_human` | commande locale structurée du modèle mono-utilisateur, sans preuve cryptographique d'humain physique |
| `expires_at`, `consumed_at` | timestamp | expiration et consommation atomique |

Une approbation expirée, consommée, ou dont le hash de profil/contexte diffère
est refusée. Maicie ne possède aucune API `Child`, `Command::spawn` ou cycle de
vie de processus.

### ActivationOutbox

| Champ | Type logique | Règle |
|---|---|---|
| `command_id` | UUID 009 | généré avant I/O, clé idempotente SpawnOrder |
| `approval_id` | UUID | lié à une seule approbation |
| `spawn_order_bytes` | bytes/texte exact | enveloppe immuable approuvée |
| `state` | `dispatching` \| `outcome_unknown` \| `applied` | autorité de livraison SpawnOrder |
| `retry_until`, `dedup_retained_until` | timestamps | même règle de reprise/tombstone |

La transaction qui fait passer une approbation vers `dispatching` écrit
simultanément `ActivationOutbox`. Tout état non terminal fait lookup/replay du
même `command_id`. L'approbation n'est `consumed` qu'après issue durable. Le
contexte après connexion est une Délégation normale, donc livré par
`OutboxDélégation` avec les bytes dont le hash a été approuvé.

## Invariants

1. Une corrélation Bridget appartient à une seule délégation Maicie.
2. Un message direct Bridget sans intention confirmée vers Maicie ne crée aucune écriture dans
   `ObjectifCoordonné` ou `Délégation`.
3. Un snapshot de transport contient fraîcheur et source ; un `Gap`, `End` ou
   une absence de source présente `inconnu`/`flux incomplet`, jamais un état métier.
4. Aucune activation de profil n'atteint `lancé` sans décision `approuvée`.
5. `OutboxDélégation` est l'unique autorité de livraison ; Délégation n'en copie
   aucun état. Toute ligne non terminale est lookup puis replay exact.
6. `ActivationOutbox` est l'unique autorité de livraison SpawnOrder ; une
   approbation n'est consommée qu'après issue durable du même `command_id`.
7. La suppression d'un objectif conserve son journal d'audit selon la politique
   de rétention locale ; elle n'efface aucun enregistrement Bridget.

## Décision d'arbitrage (2026-08-23, T008)

`IdempotencyExpired` est **terminal** (`Rejected`, motif `idempotency_expired`)
— jamais rejoué par la réconciliation, même si `retry_until` n'est pas
atteint : sans distinction protocolaire entre identifiant jamais vu et
tombstone purgé, rejouer après expiration serait deviner, et le contrat
échoue toujours vers la non-duplication. La progression remonte au domaine :
créer une nouvelle tentative de délégation (nouveau `message_id`) est une
décision de coordination explicite et journalisée, pas un rejeu de transport.
Le « replay exact » de la tâche T008 s'applique sous l'horizon uniquement
(`Prepared`/`OutcomeUnknown` → lookup → replay).
