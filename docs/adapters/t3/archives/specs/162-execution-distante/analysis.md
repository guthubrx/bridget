# Archives techniques de l’adaptateur T3

Document d’origine : [specs/162-execution-distante/analysis.md](../../../../../../specs/162-execution-distante/analysis.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**Constats actifs : aucun.** Les quatre constats de passe1 sont résolus. Aucun conflit constitutionnel actif : les demandes utilisateur de tests ciblés et de livraison complète prennent priorité sur une batterie générique répétée par tâche. T3 impose également des tests ciblés. Aucun nouveau test de confort ni outil de maintenance.

<a id="extrait-2"></a>

## Extrait 2

T3 peut démarrer T035 après T002 (schémas wire fermés), puis T036 après T033 et T035. T033 dépend seulement T002/T004. Il n’est pas nécessaire d’attendre le worker terminé. T037 attend T034 pour l’intégration maître ; tests et livraison finaux attendent le reste du lot.
