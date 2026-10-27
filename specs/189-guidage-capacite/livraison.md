# Livraison189 — Guidage des refus

Sources candidates : 001a97aaf814ac7fe0c7fbd0f9604665604a0e7b.
23 contrôles ciblés PASS et guides FR/EN vérifiés. Aucune FULL. Contre-revue indépendante b45d760a favorable ; résultat lu/ACK et garde fermée.

## Incidents inspectés
- Tâche d605c969-e6aa-4f26-822f-225134e38e8c, clé opus166-preuves-lot-autonomes-cartae-20261027-01 : le propriétaire a ensuite lu son résultat et fait ACK par le parcours natif au tour201. La lecture transitoirement refusée au tour200 n’est pas attribuée à une cause non observée. Aucune nouvelle mission, reprise, annulation ou lecture sous son identité.
- Admission opus166-correction-saut-planificateur-cartae-20261027-01 : refus durable resource_saturated, sans tâche. Le reçu183 ne conserve pas la preuve de la mesure du contrôle final. Son snapshot initial CPU35/mémoire11/charge28, seuils90/10/100, n’est pas saturé. Il ne prouve ni erreur d’admission ni saturation finale réelle. Ce reçu historique reste inchangé.

## Changement
Les prochains refus de capacité qualifiés conservent les mesures et seuils effectivement évalués, dans la transaction de refus du même reçu. Aucun seuil ou critère d’admission changé. Le code durable reste resource_saturated pour les anciens lecteurs. La preuve ferme et bornée ne remplace ni le canon ni l’histoire initiale ; les rejeux ne la recalculent pas. Une preuve absente reste inconnue.
Le précontrôle MCP d’attestation rend son étape caller_scope, une vérification fermée ou unknown, et une action précise. Aucun contexte de tâche n’est exposé avant attestation. L’opération métier n’est pas envoyée après ce refus.

## Runtime
Maître183 conservé ; aucune activation, aucun service redémarré, aucun store réel modifié. Le chargement189 dépend du même parcours de maintenance encore bloqué par une annulation historique sans preuve d’arrêt. Le candidat189 ne sera pas présenté comme actif. Aucune nouvelle préparation de maintenance identique répétée.

## Candidat Linux vérifié
Chemin : /home/moi/.local/lib/bridget-communication/189-guidance-001a97aaf814/bin/bridget
SHA256 : 330d5e713febe617b94a23d309488698568340d657d4b97a799e3a3ec558f4a9.
Version0.6.3, build189-guidance-001a97aaf814 présent dans l’exécutable ; copie hors staging identique au build et commande installed passée. Manifeste eligible_for_activation=false. Cache partagé7,11Gio. Le correctif du précontrôle doit aussi être chargé dans le client MCP concerné ; ce candidat Linux ne prouve pas une mise à jour du client Mac.

## État de livraison
Sources et reçus à intégrer/pousser sur main, puis vérifier la propreté des checkouts Mac et Cartae. Travail de la mission Opus2D, reçus historiques, binaire183 actif et versions de retour conservés. Le nettoyage de cette session sera consigné après vérification des usages ; aucun travail d’autrui retiré.
