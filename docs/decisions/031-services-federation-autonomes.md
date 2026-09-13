# ADR 031 — Services de fédération autonomes

Date : 2026-09-07. Statut : accepté, session095 validée par l'utilisateur.

## Contexte

L'extraction089 a conservé le transfert Unix SSH mais retiré install/status/remove et sa supervision persistante. L'utilisateur demande de retrouver cette capacité depuis la nouvelle distribution sur macOS et Linux, sans dépendre d'un ancien script installé.

## Décision

Étendre le script distribué existant, pas le daemon Rust. Installer une copie privée autonome du runner et une configuration inerte validée. launchd sur macOS et systemd utilisateur sur Linux supervisent le même processus SSH au premier plan, avec reprise cadencée. Une seule autorité Bridget, pas de service client vide ni daemon distant concurrent. La récupération d'une socket périmée est réservée au runner installé et exige les contrôles de propriété et refus de connexion ; run explicite garde le refus089.

## Conséquences

Le service continue après fermeture de l'installateur ou retrait du worktree ; les deux gestionnaires ont une petite projection dédiée plutôt qu'un framework commun. Le script reste l'interface d'administration humaine. Les bases et clés sont hors du retrait de service. La machine maître reste un point unique de disponibilité et les sessions utilisateur/linger conditionnent le démarrage natif. L'ancien `rm -f` avant bind n'est pas réutilisé.
