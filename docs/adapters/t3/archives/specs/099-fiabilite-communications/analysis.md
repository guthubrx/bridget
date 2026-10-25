# Archives techniques de l’adaptateur T3

Document d’origine : [specs/099-fiabilite-communications/analysis.md](../../../../../../specs/099-fiabilite-communications/analysis.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| ID | Catégorie | Gravité | Emplacement | Constat | Correction |
|---|---|---|---|---|---|
| A1 | Précision | LOW | tasks T010 | Chemin tests/t3code_098_test.rs abrégé | Chemin complet depuis dépôt |
| A2 | Vérification | MEDIUM | plan/US5 | enqueue n'est pas l'append disque | T012 couvre le failure sink existant |
| A3 | Couverture | MEDIUM | US2 | Trois voies d'envoi, pas seulement Register MCP | T004/T006 couvrent MCP, CLI, Client |
| A4 | Dégradation | INFO | contre-revue | Outil refuse identity_not_found | Fait documenté, aucune équivalence de revue prétendue |
