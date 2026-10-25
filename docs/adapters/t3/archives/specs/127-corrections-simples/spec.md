# Archives techniques de l’adaptateur T3

Document d’origine : [specs/127-corrections-simples/spec.md](../../../../../../specs/127-corrections-simples/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Exigences
- **FR-001** : une consigne commune `NO_REPLY_NOTICE` distingue « pas d'accusé de réception » et
  « action éventuelle à faire », et dit comment rendre un résultat ; reprise par tous les pilotes
  et par le pont T3 (message seul, lot, notification).
- **FR-002** : un type absent du registre est refusé `UnknownType` (types connus, registre).
- **FR-003** : `send` CLI et MCP résolvent un nom exact ou un début d'UUID (≥ 6) unique ; toute
  ambiguïté est refusée.
- **FR-004** : le fichier vide de l'ancien Bridget est retiré.
