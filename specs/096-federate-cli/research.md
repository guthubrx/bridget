# Recherche de couture096

Lecture locale : cli.rs n'a pas d'entrée federate ; script095 fournit déjà tout le cycle natif et les reçus privés. include_str permet d'emporter la même source dans le binaire ; le runner du service reste une copie autonome.

Réécrire launchd/systemd en Rust doublerait validations et tests. Chercher un script adjacent au binaire ou dans le dépôt créerait une dépendance d'installation et une ambiguïté d'autorité. Décision : source canonique embarquée, adaptateur étroit, matérialisation privée uniquement pour l'exécution qui doit pouvoir recopier son runner.

Cartae095 est enregistré avec IP37.59.185.67, port2222, labelcartae-core. L'URL DNS doit retrouver ce candidat sans réécrire sa cible ni choisir par simple préfixe de label. Le statut global ne nécessite aucune résolution DNS ni réseau.
