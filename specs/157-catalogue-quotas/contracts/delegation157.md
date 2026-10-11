# Contrat - Catalogue, quotas, remplacement (157)

Compatible avec les contrats 148, 149 et 150
(`specs/150-socle-delegation/contracts/delegation150.md`). Un appel sans nouveau
champ produit les mêmes octets canoniques. Tous les ajouts au catalogue sont
additifs.

## `bridget_capabilities` - entrée de fournisseur

Clés existantes inchangées : `agent_type`, `protocol`, `models`, `inherit`,
`inherit_refusal`, `discovery`, `development`, `development_refusal`.

Changement de sens : `discovery`, `inherit` et `development` valent `true`
seulement si le verdict partagé accepte la posture pour CET appelant
(préflight, droits, quota).

Clés ajoutées :

| Clé | Type | Sens |
|---|---|---|
| `can_run` | booléen | au moins une posture acceptée, binaire présent, quota non épuisé |
| `constraints` | liste de `{code, message}` | raisons lisibles ; vide si `can_run` |
| `quota` | objet ou `null` | voir ci-dessous ; `null` si aucun fait connu |

`quota` : `{windows: [{window, used_percent, resets_at, exhausted}], exhausted,
limit_reached, resets_at, observed_at}`. `used_percent` et `resets_at` sont `null`
s'ils sont inconnus. `exhausted` d'une fenêtre est vrai seulement si son
épuisement est attesté pour elle (statut `rejected` Claude, ou 100 % utilisé).
`limit_reached` est vrai quand une source Codex annonce une limite atteinte sans
dire quelle fenêtre : le catalogue ne bloque pas dans ce cas (contrainte
d'information `provider_limit_reached_window_unknown`).

Clés ajoutées à la racine :

| Clé | Valeur |
|---|---|
| `checks` | `{"binary": true, "quota": "observed", "auth": false}` |
| `features` | `{"fallback": true, "fallback_max": 3}` |
| `verdict_context` | `{"cwd": <cwd_root>, "fact_scope": "session"}` : le contexte pour lequel les booléens sont calculés |

Garantie de cohérence : à contexte égal (même appelant, même fait non limité à
une demande, `cwd` = `verdict_context.cwd`) et à état inchangé, un booléen
`true` du catalogue n'est jamais refusé par l'admission pour un code de la
famille verdict. Hors de ce contexte, l'admission reste l'autorité. Exemples
hors garantie : un fait limité à une demande (`request_id`), un sous-répertoire
du projet pour un parent Claude vers Claude (la politique exige un `cwd` égal à
celui du parent, `native_permissions.rs:230`), un binaire retiré ou un quota
atteint entre les deux appels.

## `bridget_delegate` - champs ajoutés

| Champ | Type | Obligatoire | Bornes | Défaut |
|---|---|---|---|---|
| `fallback` | liste d'objets `{agent_type, model, effort?}` | non | 1 à 3 entrées, sans doublon, différentes de la cible | absent |

Le champ fait partie de l'enveloppe idempotente.

## Réponse d'une tâche - champ ajouté

`substitution` : `null`, ou `{requested, selected, reason, tried}` (voir
`data-model.md`). Présent dans la réponse de `bridget_delegate`, dans
`bridget_task_status` et dans le rejeu. `agent_type`, `model` et `effort` de la
réponse sont ceux de la cible effective.

## Refus ajoutés (code renvoyé dans `native_delegation_refused`)

| Code | Quand | Détail | Écriture |
|---|---|---|---|
| `provider_quota_exhausted` | quota de la cible effective épuisé | `{resets_at}` | aucune |
| `fallback_exhausted` | `fallback` fourni, aucun candidat valide | `{tried: [{agent_type, model, code}]}` | aucune |

Refus qui changent de moment (sans changer de code) : `command_missing`,
`billing_guard`, `env_unfit`, `unsupported_capability` sont rendus à l'admission,
avant la création de la tâche. Avant 157, ils arrivaient dans l'échec de la
tâche.

Le pont MCP rend le message `code` suivi du JSON de `detail` quand un détail
existe.

## Famille « verdict » (liste fermée)

Refus qui entrent dans la cohérence catalogue et admission :
`command_missing`, `billing_guard`, `env_unfit`, `unsupported_capability`,
`negotiation_failed`, `permission_source_unavailable`, `permission_mapping_unavailable`,
`provider_confinement_unavailable`, `permission_not_inherited`,
`settings_revision_changed`, `negotiation_failed`, `provider_quota_exhausted`.

Liste « indisponibilité » (autorise un remplacement) : la famille verdict
SAUF `permission_not_inherited` et `settings_revision_changed`.

Ne déclenchent jamais de remplacement : `model_unavailable`,
`effort_unavailable`, `delegation_grant_required`, `cwd_*`,
`task_limit`, `retention_saturated`, `referent_paused`, `invalid_request`.

## Ordre des refus (insertion dans l'ordre 150)

`identity_unavailable`, `invalid_request` (dont validation de `fallback`),
`reply_timeout_out_of_range`, rejeu (`envelope_mismatch`, `request_expired`),
`referent_paused`, gardes de droits du parent 149, **sélection de cible**
(pour chaque candidat dans l'ordre : type connu, modèle, effort, PUIS
préflight, politique de l'enfant, quota), `task_limit`, `retention_saturated`, lignée parente,
`reply_timeout_exceeds_parent`.

Modèle et effort sont validés avant tout motif remplaçable : un modèle absent
ou un effort invalide avec un binaire absent ou un quota épuisé rend
`model_unavailable` ou `effort_unavailable`, sans substitution.

Sans `fallback` : le refus de la cible demandée est rendu tel quel. Avec
`fallback` : refus non remplaçable de la cible demandée rendu tel quel ; sinon
`fallback_exhausted` si tous échouent.

## Lineage T3

Aucun changement de forme. `agent_type` et `model` de la projection sont ceux de
la cible effective. Aucun état nouveau.
