# Archives techniques de l’adaptateur T3

Document d’origine : [docs/demande-interruption-de-tour.md](../../../../demande-interruption-de-tour.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le dépôt de référence `~/11.Repositories/t3code` pilote quatre fournisseurs et
implémente l'interruption pour chacun.

<a id="extrait-2"></a>

## Extrait 2

**Contrainte de langage** : le kit de référence est en TypeScript, Bridget est en
Rust. Deux voies — reproduire le protocole d'interruption en Rust en s'inspirant
de `t3code`, ou faire passer le pilotage par un programme intermédiaire. Le choix
appartient à celui qui prend le lot ; qu'il l'écrive.
