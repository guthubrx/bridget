# Archives techniques de l’adaptateur T3

Document d’origine : [specs/133-relais-sous-agents/data-model.md](../../../../../../specs/133-relais-sous-agents/data-model.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Champ | Règle |
|---|---|
| `pid` | PID du fournisseur interne, supérieur à 1 |
| `birth` | naissance OS exacte du PID |
| `parent_instance_id` | instance Bridget principale existante |
| `parent_name_file` | fichier privé qui livre le nom courant du parent |
| `provider` | valeur fermée et bornée issue de l'inventaire T3 |
| `child_ref` | empreinte opaque stable et bornée de la session interne |

<a id="extrait-2"></a>

## Extrait 2

Le fichier vit dans `delegated-pids/<pid>`. Il est privé, atomique, non
symbolique et possédé par une instance du pont T3. Il n'est pas une identité
Bridget et ne contient aucune preuve secrète du parent.
