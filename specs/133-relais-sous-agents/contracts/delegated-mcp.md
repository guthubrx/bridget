# Contrat 133 — MCP délégué

## Admission

Un appel est délégué seulement si le plus proche marqueur de la filiation est un
marqueur enfant valide et si son parent Bridget reste attesté. Un marqueur enfant
présent mais invalide arrête la résolution avec `identity_not_found`. Bridget ne
continue jamais vers un marqueur principal situé plus haut.

## Outils

| Outil | Résultat |
|---|---|
| `bridget_who` | autorisé, mêmes bornes de lecture que le parent |
| `bridget_send` | autorisé, message routé depuis le parent avec provenance enfant |
| tout autre outil | `delegated_tool_forbidden`, aucun effet |

La liste d'outils MCP peut rester compatible avec les clients existants. Le
contrôle est appliqué à chaque appel, après résolution et avant exécution.

## CLI

Une commande qui résout d'abord un marqueur enfant échoue avec
`delegated_mcp_only`. Elle n'hérite pas de l'identité principale plus haute dans
l'arbre de processus.

## Message

```json
{
  "from": "identite-parent",
  "to": "destinataire",
  "body": "contenu",
  "delegated_origin": {
    "provider": "codex",
    "child_ref": "empreinte-opaque"
  }
}
```

Le rendu visible nomme le parent et ajoute « via sous-agent <provider>
<child_ref> ». Une réponse corrélée vise `from`, donc le parent.

La provenance n'entre pas dans l'empreinte de rejeu. Un parent peut relire le
sort d'un dépôt enfant avec les mêmes `id`, `issued_at`, destinataire et corps.
Cette lecture ne remplace pas le message ni sa provenance initiale.

La façade MCP est le seul client pris en charge qui pose `delegated_origin`.
Un client de protocole modifié sous le même compte peut forger ce champ. Cette
usurpation de présentation ne donne aucun droit supplémentaire et reste hors du
modèle de cloisonnement local déclaré.

## Refus

- `identity_not_found` : aucune preuve principale ou enfant valide ;
- `identity_not_found` : preuve enfant présente mais invalide, sans repli principal ;
- `delegated_mcp_only` : tentative CLI depuis un enfant attesté ;
- `delegated_tool_forbidden` : outil MCP hors liste ;
- aucun parent unique : aucune preuve enfant publiée.
