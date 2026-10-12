# Diagnostic099 — deadline reconstituée à la reprise

Les deux nouveaux échecsFULL sont réels. Ils révèlent un défaut historique de la projection vers le wrapper. Aucun correctif hors158B préparé.

## Cause

Parce que daemon.rs:11229 sérialise le message durable avant la projection, un message ordinaire sans deadline reste sans deadline dans message_bytes. À5854–5877, defer_idempotent_delivery relit ces bytes et ajoute une deadline avec stamp_turn_deadline_for_delivery. La reprise à5930–5973 appelle exactement le même puits. À9196–9223, stamp utilise SystemTime::now().as_secs()+timeout. Une frontièreseconde entre les deux poussées produit donc+1s ; la baseA peut passer quand les deux poussées tombent dans la même seconde.

Ce n’est pas une altération SQL158B. Le seul diffsend_delivery.rs est la visibilité de stored_send_delivery ; le SELECT etmessage_bytes sont identiques. Les fonctions complètes defer etstamp sont byteidentiques àA, hashes dansJSON. Les testscrash etfixture support sont euxaussi byteidentiques àA. Le nouveau filtre is_wake_delivery exclut seulement les remisesB internes avant l’ancien puits ; aucune mesure ne permet de lui attribuer un coût causal.

## Preuve fraîche

FULLcompiled-final.log:2312–2324 : recoveryacked-wrapper1791810535→536 ; redelivery1791810537→538. Les UUID,instances,générations etautreschamps sont stables. Extraits gelés séparément, aucune stabilité du logFULLencoreactif revendiquée.

## Relecture ROOT ciblée

Conserver les deux assertions d’égalité. Une seule reprise des deux filtres danscore_089_crash_test est autorisée parROOT, aprèsFULL. CommandeJSON exacte jointe ; jobs2/nice10/testthreads2. UnPASSciblé ne supprime pas les échecsFULL ni cette limite historique. Aucun nouveau test/helper/framework oupatch horsB.
