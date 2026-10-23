# Livraison maintenance 174 — 23 octobre 2026

## Livré

Source de production : `34405ea280834fb06ac4d3367ae5fa1ab897eab0`.
Build optimisé `174-34405ea2` actif sur le maître macOS et le worker Cartae.
Le client CLI Cartae pointe également vers ce build. T3 et le pont sont restés sur leurs processus existants. Aucun travail métier n'a été créé ni rejoué.

- macOS signé : `9b6ebba9fd819aa4d2a91d8670a36a6d83ba13c5684e0487472ab69567f95e37`.
- Linux : `7899f1ff66050b59cda92754c76c3bf66ab5f321d561473207b1c6c644a0e4d9`.

## Deux corrections

1. LIFE-SERIAL : au plus une tâche distante sélectionnée par seconde et par passage, avec rotation. Les preuves de droits sont conservées. Trois tâches devant un worker muet : le passage mesuré passe de 15 007 ms à environ 5 003 ms. Les autres tâches gardent leur tour.
2. LIFE-COLLECT : les deux recherches croisées commandes/slots utilisent des index sur l'identité complète. Les protections de collecte sont inchangées. Sur la fixture de 4096 entrées, médiane de 1,637 à 1,584 s, soit environ 3 %. Les lectures disque restent dominantes ; ce résultat ne prouve pas la disparition des délais réels.

## Vérifications

40 tests ciblés distincts passent : cinq pour le maître, 35 pour le worker. Après la revue, le test du maître a été complété pour vérifier qu'un appel avant la seconde suivante n'émet aucune requête ; il passe sur le code combiné. Cet ajout ne change pas la production du build 34405ea2.

Revue indépendante du maître : accepté avec réserves. Le responsable a relu le diff complet du worker. Pas de deuxième revue ni de suite complète. Les deux compilations release ont réussi ; le statut officiel du maître confirme le build, et le processus worker correspond à l'empreinte installée.

## Limites conservées

LIFE-SERIAL est atténué, pas entièrement supprimé : un seul parcours réseau reste synchrone. La réconciliation peut attendre 5 s, puis la réservation 15 s. Ces deux délais ne constituent pas une borne globale, car un parent distant ou un autre contrôle peut ajouter une attente. Les ticks concurrents peuvent encore avoir chacun une opération en vol.

Une tâche pas encore actionnable peut consommer son tour. Le filtre existant n'a pas été élargi ou restreint sans preuve. Le débit de cette maintenance est limité à une tâche par seconde. Le curseur repart au début après redémarrage.

Neuf candidats de l'audit précédent restent non qualifiés : cinq sur les permissions, l'identité ou la reconnexion, et quatre dépendances. Le délai individuel de LIFE-SERIAL reste une limite supplémentaire à traiter. Le précédent rapport global reste inconclusif ; aucune nouvelle note globale ni certification n'est déduite de cette livraison.

Les mesures, revues et reçus opérationnels sont conservés hors Git. Aucune suppression de production ni modification de permissions. Correction technique sans parcours d'interface : pas de diaporama.
