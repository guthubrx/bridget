# Archives techniques de l’adaptateur T3

Document d’origine : [specs/169-priorite-bridget/data-model.md](../../../../../../specs/169-priorite-bridget/data-model.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Élément | Forme | Règle |
|---|---|---|
| `preferBridgetDelegation` | Booléen neuf dans `ServerSettings` existant, exclu de `ServerSettingsPatch`. | Défaut `false`. Clé absente dans un document valide : OFF. Valeur non booléenne : décodage invalide. Un patch d'un autre réglage conserve la valeur lue. |
| Fichier de réglages | Fichier résolu par le service T3 existant. | Pas de second fichier ni de réglage par projet. Les autres valeurs sont conservées. |

<a id="extrait-2"></a>

## Extrait 2

Le registre existant compare les objets par référence dans /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/mcpSession.ts:153. Remplacer cet objet pour actualiser le mode est donc interdit. Une véritable rotation continue à suivre SPEC-166.
