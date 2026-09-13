# Recette 095

1. Tests shell 089 et 095 ; plist syntaxe valide, unit systemd vérifiée.
2. Install depuis la distribution puis vérifier processus lancé depuis copie privée, fermer l'installateur et appeler status.
   Production : installer sur le Mac maître, direction -R endpoint Cartae vers socket Mac. Aucun service de tunnel ne doit tourner sur Cartae client.
3. Sur Cartae client-only, interroger le maître par socket SSH et comparer les UUID à l'annuaire Mac.
4. Deux clients privés attestés échangent un marqueur par sens et accusent réception.
5. Interrompre seulement le processus tunnel de recette identifié, attendre relance native, vérifier reconnexion et annuaire sans daemon concurrent.
6. Recette séparée systemd utilisateur sur Linux ; retirer uniquement cette installation de test.
7. Consigner les commandes exactes, hashes, sauvegardes et limites dans implementation.md. Les injections protocole ne prouvent pas un appel de modèle.
