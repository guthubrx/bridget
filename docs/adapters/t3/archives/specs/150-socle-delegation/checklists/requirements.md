# Archives techniques de l’adaptateur T3

Document d’origine : [specs/150-socle-delegation/checklists/requirements.md](../../../../../../../specs/150-socle-delegation/checklists/requirements.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Les noms d'outils (`bridget_delegate`, `bridget_capabilities`) et de refus
  (`referent_paused`, `request_expired`, `retention_saturated`) sont le contrat
  public du produit, pas un détail d'implémentation. Ils restent dans la spec.
- Les chemins de code cités dans « Besoin et périmètre » sont des preuves des
  défauts constatés, pas des choix de conception.
- Cas limites couverts : purge bloquée par un résultat non remis, purge
  saturée, rejeu d'une clé purgée, pause pendant le lancement, interruption
  hors pause, délai d'enfant supérieur au parent, tâches actives avant mise à
  jour.
- Décisions laissées au plan : borne de conservation, forme de l'indicateur de
  suspension (compatibilité Lineage T3).
