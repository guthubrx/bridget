# Vérification rapide

1. Démarrer le démon puis deux agents Bridget.
2. Depuis le premier agent, exécuter `bridget rename analyste`.
3. Vérifier que `bridget who` affiche `analyste`.
4. Depuis le second agent, envoyer un message à `analyste` et vérifier sa réception.
5. Vérifier qu’un envoi vers l’ancien nom échoue.
6. Tenter de renommer le second agent en `analyste` et vérifier le refus.
7. Exécuter `cargo test` à la racine du dépôt.
