# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/167-coordination-cli-autonome.md](../../../../../decisions/167-coordination-cli-autonome.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Un utilisateur qui travaille par le parcours natif Bridget doit pouvoir demander à son agent coordinateur de créer et piloter des enfants. Bridget doit rester le moteur de communication et de coordination interagent. Il ne doit pas reproduire l’interface de T3.

<a id="extrait-2"></a>

## Extrait 2

- T3 n’est pas requis pour déléguer ou suivre une tâche.
- Les identifiants et états ont le même sens en CLI et par MCP.
- Une clé générée est ajoutée au reçu affiché. Un script critique peut fournir `--request-id`.
- Les commandes historiques restent disponibles.
- Aucun protocole, service, stockage ou fournisseur n’est ajouté.

<a id="extrait-3"></a>

## Extrait 3

- `spawn` puis `send` : deux effets sans reçu de tâche commun.
- principal humain synthétique : filiation et remise des résultats incorrectes.
- moteur CLI séparé : duplication des règles natives.
- copie du coordinateur T3 : hors du métier de Bridget.
