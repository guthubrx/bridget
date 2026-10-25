# Archives techniques de l’adaptateur T3

Document d’origine : [specs/140-bulles-compactes/quickstart.md](../../../../../../specs/140-bulles-compactes/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Artefacts et code Rust : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes.
Frontend : /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact.

<a id="extrait-2"></a>

## Extrait 2

1. Exécuter les tests de logique MessagesTimeline avec fixtures des cinq familles et faux positifs. Ajouter les cas titre absent, hostile et JSON invalide. Constater RED avant code, puis GREEN.
2. Exécuter les tests Rust existants étendus pour le champ facultatif et la remise de titre. Contrôler membre/non-membre, normalisation, vieux lecteur et canon/journal inchangés. Employer un BRIDGET_HOME et une socket isolés.
3. Exécuter format, lint et TypeScript sur les fichiers web touchés, puis build web ciblé. Consigner les commandes exactes et codes retour dans implementation.md lors de leur exécution.
4. Dans l'aperçu isolé, rendre le vrai MessagesTimeline avec données synthétiques. Ne pas relier cet aperçu à /Users/moi/.t3/userdata ou au daemon actif.
