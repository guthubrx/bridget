# Archives techniques de l’adaptateur T3

Document d’origine : [specs/118-memoire-des-messages/spec.md](../../../../../../specs/118-memoire-des-messages/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Exigences
- **FR-001** : un message encore montré par T3 n'est jamais oublié ; au-delà de la borne, seul ce
  que T3 ne montre plus est oublié.
- **FR-002** : chaque message d'un fil, quelle que soit sa taille, est recopié une seule fois.
- **FR-003** : un journal en retard produit une annonce d'indisponibilité, sans alternance ; une
  lacune reste signalée sur une source déclarée.
