# Archives techniques de l’adaptateur T3

Document d’origine : [specs/089-communication-core/plan.md](../../../../../../specs/089-communication-core/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

1. **bridget-core** : identifiants, enveloppes et invariants de communication purs.
2. **bridget-transport** : protocole public, codecs et contrat ManagedSession ; adaptateurs ACP/Codex/Claude ; journal et provenance.
3. **bridget-daemon** : transactions, annuaire, requests, delivery, ledger, lifecycle de processus et reprise. CLI/MCP/attach sont des façades de ces mêmes services.
4. **Consommateurs extérieurs** : Maicie, future UI/T3 et Routr n'importent pas le store. Ils utilisent les interfaces publiques autorisées.

<a id="extrait-2"></a>

## Extrait 2

Refus hostiles, concurrence, compétences déclarées, métriques avant/après, corpus complet, documentation d'installation isolée/retour arrière et revue. Installation/bascule de la flotte sur décision utilisateur séparée. T3 reste une suite éventuelle, pas une dépendance à la clôture de 089.
