# Archives techniques de l’adaptateur T3

Document d’origine : [specs/119-avis-de-source-apaises/implementation.md](../../../../../../specs/119-avis-de-source-apaises/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Mesures
- Nuit du 25/09 : 222 avis d'observation vers `sol_city_ai`, 197 tours démarrés (GPT-6 astra,
  réflexion élevée), 37 réponses de simple commentaire. Base T3 lue en lecture seule.
- T3 propose `thread.activity.append` sans tour, mais l'activité n'entre pas dans le contexte de
  l'agent : piste écartée, l'avis doit lui parvenir.
