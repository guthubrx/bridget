# Archives techniques de l’adaptateur T3

Document d’origine : [specs/165-cycle-vie-liens-t3/research.md](../../../../../../specs/165-cycle-vie-liens-t3/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Le contrat local SPEC-149 définit déjà les fils `subagent` comme projections de tâche, et non comme agents T3 permanents.
- [Google A2A décrit l’échange agent comme une tâche dotée d’un cycle de vie](https://developers.googleblog.com/en/a2a-a-new-era-of-agent-interoperability/).
- [Anthropic présente les sous-agents comme des unités spécialisées qui retournent leur résultat à un orchestrateur](https://www.anthropic.com/engineering/multi-agent-research-system).

<a id="extrait-2"></a>

## Extrait 2

- Garder tous les sous-agents et augmenter la limite : rejeté, car la croissance reste linéaire avec l’historique.
- Supprimer ou archiver les fils terminés : rejeté, car l’historique appartient à T3 et doit rester consultable.
- Multiplexer tous les agents sur une connexion : rejeté pour cette session, car le protocole actuel lie une connexion à un agent et le refactor serait disproportionné.

<a id="extrait-3"></a>

## Extrait 3

**Décision** : conserver `is_live()` pour la présence T3 et ajouter une décision d’exposition Bridget.

<a id="extrait-4"></a>

## Extrait 4

Le diagnostic est reproductible avec le snapshot authentifié T3 et `lsof` sur le PID du pont. Les mesures initiales du 14 octobre 2026 sont consignées dans le plan. La recette finale reprend les mêmes compteurs après activation afin de comparer avant et après dans les mêmes conditions.
