# Maintenance distante bornée — 23 octobre 2026

## Défaut LIFE-SERIAL confirmé

Trois tâches attendent leur remise sur une socket Unix de worker qui reste muette. Le vrai `tick` émet trois requêtes Reconcile successives et attend 15 007 ms avant de rendre la main. Le test échoue avant correction sur le nombre de requêtes, pas sur un seuil de durée fragile.

## Correction

Le passage de maintenance choisit au plus une tâche distante. Un curseur en mémoire parcourt les identifiants triés, revient au début et avance même si la tâche choisie échoue. Il ne sélectionne pas plus d'une tâche par seconde. Le contrôle des permissions reste avant la réservation et avant la remise, y compris lorsqu'un slot existe déjà. Les tâches différées n'obtiennent aucune preuve ni droit d'envoi.

La sélection reste O(n), pour n tâches en attente. Les appels réseau ne sont plus multipliés par n dans un passage. Le curseur n'est pas durable et repart au début après un redémarrage.

## Comparaison avec T3

Lecture de `apps/server/src/provider/Layers/ProviderSessionReaper.ts` : T3 sépare son balayage dans une activité de fond, espacée, et vérifie les tâches actives avant arrêt. T3 n'a pas de réservation de résultat sur un worker distant. Bridget réutilise ici son propre mécanisme existant de rotation des sondes ; déplacer tout le traitement dans une nouvelle activité asynchrone serait un changement plus large. Aucune preuve distante n'est supprimée pour gagner du temps.

## Validation et limites

Après correction, la même fixture produit une requête par passage : 5 003, 5 003, 5 003 puis 5 002 ms. Les trois tâches sont visitées dans l'ordre puis le curseur revient à la première. Aucune mission n'est remise sans preuve. Les slots conservés restent identiques et les demandes réseau en attente sont libérées.

Cinq tests ciblés passent : borne et rotation du tick, deux contrôles de réservation, reprise après pause, classification des preuves de continuation. Aucune suite complète ni mesure de charge de production.

La réservation reste synchrone : une tâche lente peut encore retarder un passage. Le plafond porte sur le nombre de tâches visitées, pas sur la durée totale de tous les contrôles. Le débit maximal de cette maintenance est d'une tâche par seconde ; une file importante attend son tour. L'admission initiale conserve son parcours direct. Le gain mesuré de 15 à 5 secondes concerne le worker muet de cette fixture, pas toutes les délégations réelles.
