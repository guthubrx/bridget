# Journal des changements

Chaque version publiée s'ajoute à l'historique du dépôt public, avec son tag `vX.Y.Z`. Les numéros
suivent le versionnage sémantique : le dernier chiffre pour les corrections et la documentation, le
deuxième pour les nouvelles capacités, le premier pour les ruptures de contrat. Les dates sont au
format AAAA-MM-JJ.

## [0.1.3] - 2026-10-02

### Corrigé

- `bridget spawn` attend la durée demandée par `--timeout` (plus une marge) avant de conclure ;
  un fournisseur lent à démarrer était lancé mais annoncé « outcome_unknown » au bout de 10 s.
- Pont T3 Code : les notifications d'observation qui se suivent partagent un seul tour au lieu de
  réveiller l'abonné une fois chacune ; elles ne sont jamais mêlées aux messages.
- Pont T3 Code : un message sans réponse attendue destiné à un fil Claude ou Cursor occupé est remis
  aussitôt au tour en cours, au lieu d'attendre sa fin (jusqu'à 8 h 45 observées). Codex, les
  demandes suivies et les notifications attendent toujours la fin du tour.

### Documentation

- Guide d'installation : déclarer un fournisseur compatible Claude (exemple GLM) ou un second
  compte Codex dans le registre, avec sa variante lecture seule automatique.

### Modifié

- Pont T3 Code : une demande avec réponse attendue invite l'agent à ouvrir sa réponse par
  « ↪ Réponse à <expéditeur> (relayée par Bridget) : ». Relayée à l'expéditeur, elle reste aussi
  affichée dans le fil de l'utilisateur, qui la prenait pour lui.
- La consigne des messages sans réponse attendue sépare l'accusé de réception de l'action :
  « Pas d'accusé de réception… Ce n'est pas une absence de tâche ». L'ancienne formule était lue
  comme « rien à faire ».
- `send` accepte un début d'UUID d'au moins 6 caractères (CLI et MCP) ou un nom d'affichage exact
  (CLI) quand il ne désigne qu'un seul agent.
- Un lancement d'équipier d'un type absent du registre est refusé « type d'agent inconnu », avec
  la liste des types connus, au lieu d'une capacité manquante.

## [0.1.2] - 2026-10-02

### Documentation

- Skill : lancer des équipiers en lecture seule est autorisé d'office pour une tâche de lecture,
  sans terminal ; seul le droit d'écrire (`--posture development`, Codex) exige un terminal
  humain ; un refus « posture_decouverte » signale un type non déclaré dans le registre.

### Corrigé

- Pont T3 Code : un tour aussitôt suivi d'un autre, entre deux lectures du pont, était signalé
  comme une lacune d'observation ; sa fin est désormais annoncée normalement quand l'origine du
  tour a été relevée pendant qu'il tournait.

## [0.1.1] - 2026-09-25

### Modifié

- `bridget --help` affiche la version réelle du paquet ; elle était figée à 0.1.0 dans le texte.

### Documentation

- README à jour des capacités récentes, et ce journal des changements.
- La publication est désormais versionnée : une nouvelle version s'ajoute à l'historique au lieu de
  le remplacer, ce qui permet de comparer deux versions.

## [0.1.0] - 2026-09-25

Première version publique versionnée. Les publications précédentes remplaçaient l'historique ;
celle-ci en est le point de départ. Elle contient notamment les changements suivants.

### Ajouté

- **Abonnements durables.** Les abonnements d'observation survivent à un redémarrage du daemon :
  ils reprennent seuls au retour de leur source, et leur propriétaire est averti que les faits de
  la coupure sont perdus, au lieu de devoir se réabonner.
- **Avis amortis.** Un changement d'état d'une source n'est annoncé qu'après 30 secondes de
  stabilité ; une source qui clignote donne au plus un avis « source instable » toutes les 5
  minutes. Chaque avis réveillant un agent, cela évite des tours dépensés pour rien.
- **Pont T3 Code : tours spontanés.** Un fil dont le dernier tour est un réveil d'arrière-plan (fin
  d'un sous-agent) reste observable ; sa fin de tour est signalée.
- **Pont T3 Code : réponses incertaines.** Quand une réponse ne peut pas être reliée avec certitude
  à sa demande, elle est transmise avec la mention « appariement incertain » au lieu d'être gardée.
- **Pont T3 Code : seconde application.** Une seconde application T3 ouverte sur les mêmes données
  déclenche une notification qui la nomme ; le fichier d'état qu'elle efface en se fermant est
  rétabli automatiquement.
- **Appariement par origine prouvée.** La réponse relayée est celle du tour réellement déclenché par
  la demande, prouvé par horodatage, et non plus déduite d'un décompte.
- **Entretien automatique.** Le daemon purge chaque heure les remises expirées et l'état d'identité
  des processus disparus, et fait tourner ses journaux ; la construction retire les worktrees
  fusionnés et inoccupés.
- **Remises regroupées.** Les messages sans réponse attendue destinés à un même fil partagent un tour.

### Corrigé

- Un grand fil (plus de 1 000 messages) n'est plus recopié en boucle dans le journal d'observation.
- Le pont n'annonce plus une source prête avant d'avoir vérifié son journal, ce qui produisait des
  notifications en alternance.
- Un processus disparu pendant l'inventaire des fichiers ouverts n'invalide plus l'identité de
  tous les autres fils.
- La CLI lancée depuis un fil T3 attesté ne signe plus ses envois « human ».
- L'identifiant de build n'est plus figé au premier passage du script de compilation.
- Les tests d'intégration ne dépendent plus de la charge de la machine.
