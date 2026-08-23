# Plan : fédération SSH

Le daemon maître reste le seul daemon. Un tunnel SSH inverse publie son socket Unix sur le chemin standard de la machine distante ; les wrappers distants rejoignent donc l’annuaire existant.

## Décision

Utiliser `ssh -N -R <socket-distant>:<socket-maître>` avec `ExitOnForwardFailure`, keepalives et un LaunchAgent local pour la persistance. Aucun transport TCP Bridget, chiffrement ou authentification applicative supplémentaire.

## Fichiers

- `scripts/federate-ssh.sh` : installer, statut, arrêt et validation du tunnel.
- `README.md` : procédure opérateur.
- `scripts/test-federate-ssh.sh` : validation locale des paramètres, sans hôte réel.
- `crates/bridget-daemon/src/wrapper.rs` : boucle de reconnexion du wrapper et réenregistrement stable après perte du socket.

## Risques

Le test de bout en bout exige un hôte SSH distant configuré. Une coupure peut laisser le processus IA vivant mais isolé : le wrapper doit donc dissocier la vie de ce processus de sa connexion socket et réessayer jusqu'au retour du tunnel.
