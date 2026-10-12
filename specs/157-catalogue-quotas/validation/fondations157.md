# Fondations157 — T003 à T006
T003 :1PASS transport, octets150 identiques sansfallback.
T004/T005/T006 :24PASS daemon, dont six limites, cinq préflight, stockage150 et tous les tests lifecycle existants sélectionnés.
RED compilable exécuté :1PASS/10FAIL. Six échecs limites et trois préflight métier (billing/HOME/protocole).
Le dixième échec concernait la fixture de définition épinglée : submit_spawn_from_resolved exige un agent de reprise existant (InvalidOrder). Fixture corrigée par création initiale submit_spawn sur le registre épinglé, puis prepare_recovery ; oracles inchangés. Cet échec n'est pas compté comme défaut métier ni preuve RED.
GREEN reprend les cinq mêmes scénarios prévus. Aucun nouveau scénario ajouté.
Deuxjobs/deuxthreads/nice10, une commande à la fois ; logs conservés dans /Users/moi/.cache/bridget-plan157-20261012-r1.
