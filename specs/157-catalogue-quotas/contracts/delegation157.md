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
| `constraints` | liste de `{code, message}` | raisons lisibles ; vide si `can_run`, sauf l’information non bloquante unique `provider_limit_reached_window_unknown` |
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
| `verdict_context` | `{"cwd": <cwd_root ou null>, "fact_scope": "session"}` : le contexte pour lequel les booléens sont calculés ; `cwd:null` indique une racine absente, sans créer de droit ni autoriser une admission sans dossier |

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
`settings_revision_changed`, `provider_quota_exhausted`.

Liste « indisponibilité » : exactement FR-013. Tous les autres codes de la
cible demandée sont rendus tels quels. negotiation_failed n’est listé qu’une
fois ci-dessus ; la liste fermée évite de rendre un refus de droits remplaçable.

Ne déclenchent jamais de remplacement : `model_unavailable`,
`effort_unavailable`, `agent_type_unknown`,
`permission_attestation_unavailable`, `delegation_grant_required`,
`workspace_requires_development`, `workspace_inheritance_unavailable`,
`child_requests_unsupported`, `child_requests_reviewer_unsupported`, `cwd_*`,
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

## Précisions retenues de la revue157-r1 — appliquées au GO157

GO157 le2026-10-12 après livraison et activation155/156. Base source156
cf498755dc871b51eb8c0add5f8a882d67651d2a, base Git avec clôture documentaire
8b5b313694cf05eabfe6a86a970b96f03b015f49. Les références150/1f2c661d
restent des preuves historiques ; les fonctions sont revalidées sur cette base.

Le verdict conserve l’ordre type/modèle/effort, préflight, politique, gardes
liées aux options de l’appel, quota. Workspace et, après livraison155,
child_requests sont non remplaçables pour la cible demandée ; un candidat
explicitement listé refusé est noté dans tried. Aucun gain de droits ni
contournement des gardes du parent. En worktree, sélection unique en Probe,
candidat et définition épinglés transmis à Reserve puis Commit. Commit
revalide ce même candidat dans le dossier enfant réel ; si une garde échoue,
refuser sans nouvelle sélection et conserver les diagnostics de chemin154.
Les options immuables et canonical original restent distincts de request effectif.

Quota : huit fenêtres ; purger les passées avant l’écriture. Une neuvième clé
évince le plus petit observed_at, égalité départagée par clé lexicale.
unknown du hook n’est pas une preuve seul ; pourcentage>=100 avec retour
futur épuise toute source. provider_limit_reached_window_unknown est l’unique
contrainte informative possible si can_run, hors famille verdict et jamais
rendue comme refus d’admission. Aucun nouveau champ public.

Refus enrichi : Err(String) sous forme code:JSON contenant message/detail.
Étendre le seul failure existant ; les comparaisons de listes portent sur
le code canonique. JSON non conforme : refus fermé, aucune valeur inventée.
Les anciens codes nus et préfixes restent compatibles. Aucun nouveau type,
helper, framework, table, outil, ID ou scénario autonome. Les tests prévus
T004/T016/T017/T018 couvrent ces précisions. Les26 tâches restent ouvertes.
