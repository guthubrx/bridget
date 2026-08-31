# Contrat relais - profils d'agents et attention v1

## Portée et autorisation

Toutes les routes sont servies par le relais UI existant et exigent le jeton
actuel. Elles n'ouvrent aucun port supplémentaire et ne créent aucun compte.
Le jeton est transporté par le paramètre de requête `token`, comme toutes les
routes du relais existant. Cette API locale ne reconnaît donc pas l'en-tête
`Authorization` dans cette version.

Le protocole utilise JSON UTF-8. Les erreurs utilisateur sont des codes stables
et des phrases sans `agent_id`, nom de routage, texte de consigne ou secret.

`profile_ref` est un UUID opaque, présent seulement dans l'état applicatif pour
adresser une route. Le client ne doit pas le rendre. Les exemples le montrent
pour définir le contrat, jamais comme texte UI.

## Projection de snapshot et watch

Chaque `UiAgentRowV1` contient la projection suivante après migration:

```json
{
  "profile": {
    "profile_ref": "opaque-uuid",
    "display_name": "Bibou",
    "labels": ["coordinateur", "recherche"],
    "avatar": { "shape": "round", "color": "blue" },
    "instruction_state": {
      "revision": 4,
      "status": "pending_restart",
      "updated_at": 1788165000
    }
  }
}
```

Les valeurs historiques de routage peuvent rester dans les données de transport
interne mais ne font pas partie de cette projection affichable. Pendant la
migration, `profile` peut être absent: le client applique un fallback de lecture
sans casser le fil, puis le prochain snapshot confirmé remplace ce fallback.

## Lire un profil ouvert

```text
GET /v1/agent-profiles/{profile_ref}?token=<relay-token>
```

Réponse `200`:

```json
{
  "profile": {
    "profile_ref": "opaque-uuid",
    "display_name": "Bibou",
    "labels": ["coordinateur", "recherche"],
    "avatar": { "shape": "round", "color": "blue" },
    "instructions": "Coordonne les recherches...",
    "revision": 12,
    "instruction_state": {
      "revision": 4,
      "status": "pending_restart",
      "updated_at": 1788165000
    },
    "updated_at": 1788165000
  }
}
```

La réponse est destinée exclusivement au panneau déjà ouvert par l'utilisateur.
Elle ne doit pas être mise dans les traces JavaScript, les titres, l'historique
ou les événements d'attention.

## Modifier un profil

```text
PATCH /v1/agent-profiles/{profile_ref}?token=<relay-token>
Content-Type: application/json
```

```json
{
  "expected_revision": 12,
  "display_name": "Bibou",
  "labels": ["coordinateur", "recherche"],
  "avatar": { "shape": "round", "color": "blue" },
  "instructions": "Coordonne les recherches..."
}
```

Règles de validation:

| Champ | Règle |
|---|---|
| expected_revision | obligatoire, entier positif |
| display_name | trim, 1 à 80 caractères, unique après normalisation serveur |
| labels | 0 à 12 libellés, 1 à 32 caractères, trim et unicité normalisée |
| avatar.shape | enum de la palette réellement rendue |
| avatar.color | enum de la palette réellement rendue |
| instructions | 0 à 8 000 caractères, texte privé, jamais loggé |

Réponse `200`: profil confirmé et état d'application courant. Une mutation
sauvegardée pendant une session active renvoie `pending_restart`; elle ne doit
pas créer de relance implicite ou d'interruption.

Erreurs:

| HTTP | code | Sens pour l'UI |
|---|---|---|
| 400 | invalid_profile | champ invalide, rendre l'erreur au champ concerné |
| 401 | unauthorized | session relais invalide, ne pas répéter la valeur saisie dans le diagnostic |
| 404 | profile_not_found | profil disparu, rafraîchir le snapshot |
| 409 | display_name_conflict | choisir un autre nom, sans désigner l'autre agent |
| 409 | profile_revision_conflict | recharger puis faire confirmer à l'utilisateur |
| 503 | profile_store_unavailable | garder le formulaire local non soumis et afficher une indisponibilité sobre |

La mutation est atomique: aucune étiquette, apparence ou instruction partielle
ne peut être visible après un échec.

## Client d'attention et préférences

Le client génère une fois un `client_id` UUID. Le navigateur le conserve dans
son stockage local. Bridget Desktop le conserve dans son répertoire de données
natif. Ce n'est ni un jeton ni une identité de routage.

### Lire l'activité

```text
GET /v1/attention?token=<relay-token>&client_id=<uuid>&after=<event_id>&limit=100
```

Réponse `200`:

```json
{
  "version": 1,
  "events": [
    {
      "event_id": "01J...",
      "profile_ref": "opaque-uuid",
      "display_name": "Bibou",
      "event_type": "human_input_needed",
      "summary": "Bibou attend votre réponse.",
      "created_at": 1788165060,
      "seen": false,
      "native_notified": false,
      "attention_enabled": true
    }
  ],
  "next_after": "01J..."
}
```

Le serveur applique les préférences de ce `client_id` dans cette projection.
Les événements non sélectionnés peuvent rester consultables seulement si le
produit les présente comme historique non alertant, mais ils ne contribuent ni
au badge ni à `native_notified`.

### Lire et modifier les préférences

```text
GET /v1/attention/preferences?token=<relay-token>&client_id=<uuid>
PUT /v1/attention/preferences?token=<relay-token>
```

```json
{
  "client_id": "uuid-local-au-client",
  "preferences": [
    {
      "profile_ref": "opaque-uuid",
      "human_input_needed": true,
      "task_completed": false,
      "terminal_failure": true
    }
  ]
}
```

`PUT` remplace atomiquement les préférences explicitement envoyées pour le
client et renvoie le verdict normalisé. Il n'altère aucune préférence d'un autre
client ni le profil serveur partagé.

### Marquer les événements lus et notifiés

```text
POST /v1/attention/state?token=<relay-token>
```

```json
{
  "client_id": "uuid-local-au-client",
  "event_ids": ["01J..."],
  "action": "mark_seen"
}
```

Tous les horodatages numériques de ce contrat sont des secondes Unix UTC. Le
client les convertit seulement au rendu. Cela évite de créer une deuxième
convention de date dans le relais.

Actions v1: `mark_seen` et `mark_native_notified`. Elles sont idempotentes. Un
client envoie `mark_native_notified` seulement après un envoi local réussi ou
une permission explicitement refusée. Un événement n'est jamais marqué lu par
l'émission seule d'une notification.

## Sémantique des événements

| event_type | Produit lorsque | Exclusions obligatoires |
|---|---|---|
| human_input_needed | l'exécution attend une décision ou un complément humain explicite | message routinier, attente réseau, pensée, outil |
| task_completed | une tâche ou délégation ciblée se termine avec succès | chunk final sans fin attestée |
| terminal_failure | une exécution se termine sans reprise possible | avertissement, retry, erreur outil récupérée |

Chaque occurrence a une clé de déduplication interne. Une reconnexion ou une
réémission du fournisseur ne crée donc pas une deuxième notification.

## Compatibilité et confidentialité

1. Tout nouveau champ est optionnel à la lecture pour préserver les clients
   antérieurs pendant le déploiement progressif.
2. Aucun endpoint ne modifie `name`, les adresses, les messages, les commandes
   de flotte ou les permissions.
3. Le serveur ne renvoie jamais le texte d'instructions dans snapshot, watch,
   activité, logs de contrat ou erreurs.
4. Tous les textes utilisateur affichables proviennent du display name validé
   ou de summaries construits côté serveur.
5. Les routes de profil et d'attention emploient l'autorisation du relais
   existant, avec la même politique d'origine et de jeton.
