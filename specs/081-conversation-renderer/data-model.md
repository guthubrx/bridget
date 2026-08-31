# Modèle de données - SPEC-081

## `ConversationTurnV1`

Projection locale, non persistée, dérivée de la chronologie existante.

| Champ | Type | Règle |
|---|---|---|
| `id` | chaîne | `messageId` attesté du tour ; jamais généré depuis le texte. |
| `request` | message humain ou `null` | Unique après dédoublonnage journal/flux. |
| `segments` | liste ordonnée | Réponse texte, lot d'actes, système, erreur ; ordre source conservé. |
| `state` | `working`, `completed`, `failed`, `interrupted` | Déduit seulement des événements attestés. |
| `durationMs` | entier ou `null` | Visible uniquement si `turn_end` ou échec terminal l'atteste. |
| `activitySummary` | objet | Compteurs, dernier acte, état et détails repliables. |

Les rondes de vigilance et `peer_exchange` ne deviennent jamais des
`ConversationTurnV1` humains.

## `ContentSecurityPreferencesV1`

Préférences strictement locales de l'opérateur.

```json
{
  "version": 1,
  "externalLinks": false,
  "fileReferences": false,
  "remoteImages": false
}
```

| Champ | Validation | Défaut sûr |
|---|---|---|
| `version` | entier exact `1` | document refusé si différent |
| `externalLinks` | booléen strict | `false` |
| `fileReferences` | booléen strict | `false` |
| `remoteImages` | booléen strict | `false` |

Le document de préférences globales garde ses autres champs. Une migration
depuis le format existant est explicite et ne s'applique qu'à un document
valide déjà présent sur le Mac actuel. Un reset, une corruption, une version
inconnue ou un profil nouvellement créé remet les trois booléens à `false`.

## `ContentReferenceV1`

Objet transitoire issu d'un lien Markdown après analyse et avant rendu.

| Champ | Valeurs | Règle |
|---|---|---|
| `kind` | `external_link`, `project_file`, `remote_image`, `blocked` | Classé sans effectuer de requête. |
| `target` | URL ou chemin | Jamais injecté comme HTML. |
| `label` | texte | Échappé, borné et affiché à l'opérateur. |
| `reason` | code de refus optionnel | Requis pour un contenu bloqué. |

Les schémas `javascript:`, `data:`, `vbscript:`, `file:` et les URL relatives
ambiguës sont `blocked`. Les images acceptées sont HTTPS et non SVG. Les
chemins de projet doivent être absolus et seront revalidés par le relais.

## `FilePreviewV1`

Réponse relayée de consultation, jamais un handle de fichier.

| Champ | Type | Règle |
|---|---|---|
| `version` | entier | `1`. |
| `path` | chaîne | Chemin canonique relatif à la racine autorisée, pas de secret de configuration. |
| `media_type` | enum | `text/plain`, langages texte connus ou image raster sûre. |
| `encoding` | `utf8` ou `base64` | Texte UTF-8 ou image raster encodée, sans `data:`. |
| `content` | texte ou octets encodés | Seulement sous plafond. |
| `truncated` | booléen | Vrai si limite de taille atteinte, jamais silencieux. |
| `sha256` | chaîne | Empreinte de la portion servie, pour repérer un changement. |

## Invariants

- Une préférence ne porte ni jeton de tunnel, ni contenu de conversation, ni
  chemin de serveur.
- Un aperçu est une lecture unique explicitement demandée et ne rend aucun
  fichier exécutable.
- Une URL ou un chemin rejeté ne provoque aucune requête secondaire.
- Un code inconnu reste du texte. Une coloration ne modifie jamais le contenu
  copié.
