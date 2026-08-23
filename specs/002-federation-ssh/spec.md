# Spécification : fédération Bridget par tunnel SSH

**Branche** : `session-02-federation-ssh`  
**Statut** : Clôturée — implémentée et validée ; le binaire release attend son prochain redémarrage contrôlé du daemon.

## Scénarios

### US1 — Connecter une machine distante (P1)

Un opérateur expose le daemon Bridget local à une machine distante via SSH ; un agent lancé sur celle-ci apparaît dans le même annuaire et échange avec les agents locaux.

**Acceptation** : un agent distant envoie et reçoit un message sans port réseau public.

### US2 — Maintenir le lien (P2)

Le tunnel est relancé automatiquement après une déconnexion et son état est observable.

### US3 — Réinscrire un agent vivant après une coupure (P1)

Lorsqu'un agent distant reste en cours d'exécution pendant une coupure SSH, son wrapper se reconnecte au daemon maître et reprend le même nom dès le retour du tunnel.

**Acceptation** : l'agent n'est plus visible pendant la coupure, puis réapparaît sans redémarrage du processus IA après rétablissement du socket.

## Exigences

- Le système DOIT utiliser un tunnel SSH authentifié et un socket Unix distant.
- Aucun port TCP public ne DOIT être ouvert.
- Le tunnel DOIT viser un daemon maître unique ; aucun second annuaire ne DOIT être créé.
- Le système DOIT fournir des commandes installer, statut et arrêt réversibles.
- Les erreurs SSH DOIVENT être visibles dans des journaux locaux.
- Le wrapper DOIT tenter de se reconnecter tant que son processus IA enfant est vivant.
- Après reconnexion, le wrapper DOIT se réenregistrer sous son nom existant.

## Succès

- Une machine distante peut joindre le même annuaire d’agents que la machine maître.
- Une coupure de tunnel est détectée puis rétablie sans intervention après retour du réseau.
- Une tentative sans clé SSH valide échoue sans exposer le socket local.
- Un agent distant vivant redevient joignable après une coupure et un retour de tunnel.
