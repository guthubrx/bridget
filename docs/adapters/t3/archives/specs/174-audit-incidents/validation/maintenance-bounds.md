# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/maintenance-bounds.md](../../../../../../../specs/174-audit-incidents/validation/maintenance-bounds.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Lecture de `apps/server/src/provider/Layers/ProviderSessionReaper.ts` : T3 sépare son balayage dans une activité de fond, espacée, et vérifie les tâches actives avant arrêt. T3 n'a pas de réservation de résultat sur un worker distant. Bridget réutilise ici son propre mécanisme existant de rotation des sondes ; déplacer tout le traitement dans une nouvelle activité asynchrone serait un changement plus large. Aucune preuve distante n'est supprimée pour gagner du temps.
