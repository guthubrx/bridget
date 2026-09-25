# Journal des changements

Chaque version publiée s'ajoute à l'historique du dépôt public, avec son tag `vX.Y.Z`. Les numéros
suivent le versionnage sémantique : le dernier chiffre pour les corrections et la documentation, le
deuxième pour les nouvelles capacités, le premier pour les ruptures de contrat. Les dates sont au
format AAAA-MM-JJ.

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
