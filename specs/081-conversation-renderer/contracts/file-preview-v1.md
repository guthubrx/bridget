# Contrat - Aperçu relayé de fichier V1

## Requête

```text
GET /v1/content/file-preview?path=<chemin-absolu-encodé>&token=<jeton-relay>
```

La route est appelée seulement après un clic explicite sur une référence de
fichier et si `fileReferences` est activé localement.

## Succès 200

```json
{
  "version": 1,
  "path": "src/main.rs",
  "media_type": "text/rust",
  "encoding": "utf8",
  "content": "fn main() {}\n",
  "truncated": false,
  "sha256": "..."
}
```

Pour une image raster autorisée, `encoding` vaut `base64` et `content` est le
flux d'octets encodé. Le client le transforme en URL `blob:` locale après le
clic, jamais en URL distante ou en `data:` injectée par le message.

Le chemin retourné est relatif à une racine de projet autorisée. La réponse ne
contient ni chemin racine absolu, ni configuration, ni secret.

## Refus

| Statut | Code | Sens |
|---|---|---|
| 400 | `invalid_path` | Chemin absent ou non absolu. |
| 403 | `outside_project_roots` | Chemin canonique hors de la politique projet. |
| 404 | `not_found` | Fichier absent. |
| 413 | `too_large` | Fichier ou image au-delà de la limite. |
| 415 | `unsupported_media_type` | Binaire, SVG ou type non prévisualisable. |

## Garanties

- lecture seule, sans shell, sans écriture et sans suivi de lien ;
- canonicalisation avant comparaison aux racines autorisées ;
- lien symbolique accepté seulement si sa cible canonique reste sous une
  racine autorisée ;
- taille maximale et temps de lecture bornés ;
- aucune redirection, aucun proxy HTTP et aucune ouverture du Finder.
