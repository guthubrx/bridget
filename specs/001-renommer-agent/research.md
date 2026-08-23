# Recherche : renommage d’agent

## Décision : renommer depuis la connexion d’agent existante

**Raisonnement** : l’identité active est liée à une connexion inscrite dans le routeur. Le démon doit donc traiter la demande au même endroit que les autres messages émis par le wrapper, afin qu’un tiers ne puisse pas renommer un autre agent.

**Alternatives écartées** :

- Réenregistrer l’agent : crée une fenêtre où l’identité est absente et complique les collisions.
- Laisser un autre client fournir le nom à renommer : ne prouve pas l’autorité sur l’agent ciblé.
- Déconnecter puis relancer l’agent : ne satisfait pas l’exigence de renommage à chaud.

**Impact mainteneur** : la règle reste simple : le démon dérive l’agent source de sa connexion, jamais d’un nom fourni par la commande.

## Décision : échange atomique dans l’annuaire

**Raisonnement** : l’annuaire est la source de vérité des noms. Il valide d’abord le nouveau nom et son unicité, puis remplace l’ancienne clé et l’identité de l’agent dans la même opération logique.

**Alternatives écartées** : retirer puis ajouter l’agent dans deux opérations publiques. Elles exposeraient un état intermédiaire et fragiliseraient le routage concurrent.

**Impact mainteneur** : le routeur porte un unique invariant testable : un nom pointe vers au plus une connexion.

## Décision : conserver le nom après confirmation du démon

**Raisonnement** : le wrapper ne doit mettre à jour son nom persistant qu’après la confirmation de succès. Ainsi une collision ou une erreur ne modifie pas la reprise future.

**Alternatives écartées** : écrire le nom avant la réponse ou ne jamais l’écrire. La première désynchronise client et démon ; la seconde rend le renommage éphémère.

**Impact mainteneur** : le fichier persistant suit strictement l’état confirmé par le démon.

## Décision : fichier d’état local du wrapper

**Raisonnement** : le processus enfant reçoit son nom initial dans son environnement. Sans point de consultation partagé, ses commandes ultérieures continueraient à utiliser l’ancien nom. Le wrapper écrit donc le nom confirmé dans un fichier d’état local dont le chemin est transmis au processus enfant.

**Alternatives écartées** : ouvrir une seconde connexion CLI au démon avec le nom déclaré par l’enfant. Elle ne prouve pas que l’enfant possède l’identité visée et laisserait `send` et `reply` sur l’ancien nom.

**Impact mainteneur** : un seul état de nom actif côté client ; les sous-commandes le lisent sans ajouter de serveur local.
