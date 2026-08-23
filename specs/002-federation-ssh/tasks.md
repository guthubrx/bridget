# Tâches : fédération SSH

- [x] T001 Créer `scripts/federate-ssh.sh` avec les commandes `install`, `status` et `remove` basées sur un reverse Unix socket SSH.
- [x] T002 Créer `scripts/test-federate-ssh.sh` pour vérifier les paramètres SSH et les erreurs d’usage sans hôte distant.
- [x] T003 Documenter la procédure et les prérequis dans `README.md`.
- [x] T004 Exécuter le test local et une validation réelle avec un hôte SSH fourni par l’opérateur.
- [x] T005 Ajouter au wrapper une reconnexion avec réenregistrement sous le même nom après perte de socket.
- [x] T006 Couvrir la reconnexion par un test d'intégration et valider le scénario de coupure SSH sur l'hôte Linux.
- [x] T007 Remplacer le retry fixe du wrapper par un backoff exponentiel plafonné avec jitter et réinitialisation après stabilité.
- [x] T008 Transmettre le nom d'hôte du wrapper et afficher `bridget who` en colonnes alignées.
- [x] T009 Conserver temporairement une présence `unreachable` et la mettre à jour par heartbeat après une coupure de tunnel.
- [x] T010 Déployer les skills Bridget dans les répertoires utilisateurs Codex et Claude de l'hôte enrôlé.
- [x] T011 Refuser `send --reply` depuis un client éphémère et rattacher les relances au wrapper durable de l'émetteur.
