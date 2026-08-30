# Contrat UI agent identity v1 - SPEC-071

## Extension de `/v1/snapshot`

L'objet `agents[]` conserve tous ses champs actuels et ajoute :

```json
{
  "name": "cartae0-flux",
  "type": "codex",
  "host": "cartae.app",
  "transport": "codex_app_server",
  "mode": "cli",
  "model": "gpt-5.6-terra",
  "effort": "high"
}
```

Compatibilité :

- `transport` est une chaîne issue de la présence actuelle ; un lecteur doit
  tolérer une chaîne vide ou inconnue ;
- `mode`, `model` et `effort` sont optionnels ;
- un ancien payload sans ces champs reste valide ;
- aucune valeur n'est reconstruite côté serveur.

## Routes d'assets

| Méthode | Route | Type |
|---|---|---|
| GET | `/providers/openai.svg` | `image/svg+xml` |
| GET | `/providers/claude.svg` | `image/svg+xml` |
| GET | `/providers/cursor.svg` | `image/svg+xml` |
| GET | `/providers/gemini.svg` | `image/svg+xml` |

Ces routes utilisent le contrat statique existant : ETag fort,
`Cache-Control: no-cache`, réponse 304 sur ETag identique et aucune exigence de
jeton supplémentaire pour un asset public déjà référencé par la page.

## Contrat de présentation

- Le logo est décoratif : `alt=""` et nom visible adjacent.
- La fiche possède `role="tooltip"` et un identifiant stable.
- La ligne ouverte référence cet identifiant via `aria-describedby`.
- Le texte visible fournit le produit, l'éditeur et le mode sans dépendre du
  logo ni de la couleur.
- `MODE INCONNU` est affiché quand la table fermée ne prouve ni TMUX ni FLUX.
