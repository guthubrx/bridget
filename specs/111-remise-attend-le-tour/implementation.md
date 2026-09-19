# Journal d'implémentation — 111 Une remise attend la fin du tour

## Métadonnées
- **Spec** : 111-remise-attend-le-tour — **Branche** : session-111-remise-attend-le-tour
- **Base** : main `c2e8cd86` — **Date** : 2026-09-19 — **Statut** : In Progress (livraison en cours)

## Diagnostic

Journal du pont : `remise mcp-47021-6aae971d-1 périmée avant démarrage`, deux minutes exactement
après l'envoi d'une libération de périmètre entre deux agents. Un relais envoyé ensuite a subi le
même sort à la même seconde d'écart. Neuf remises jetées dans la journée. Le destinataire n'était
pas en panne : il écrivait encore une minute avant le constat, mais ses tours durent plus de deux
minutes, donc il n'est jamais inactif au bon moment.

Cause : `drive_queue` écartait toute remise dont l'attente dépassait `turn_wait`, quelle que soit
l'échéance réelle du message. Ce délai servait de garde-fou mémoire, la file n'étant bornée par
rien d'autre.

## Correction

- `t3code.rs` : la condition `received.elapsed() >= turn_wait` est retirée des deux points de
  filtrage. Les autres motifs, tous fondés sur une échéance portée par le message, sont inchangés :
  échéance métier, délai de réponse, expiration de saga, demande suivie close.
- Nouvelle borne `QUEUE_BOUND = 64` appliquée à l'insertion. Un dépassement écarte la plus ancienne
  avec un avertissement distinct, pour ne pas confondre saturation et péremption.
- `turn_wait` reste employé par le cache des annulations, dont la sémantique est différente.

Allonger le délai aurait déplacé le seuil sans le supprimer, et rendu le symptôme plus rare donc
plus difficile à diagnostiquer. Un message sans échéance n'a aucune raison de périmer : le
protocole permet déjà à l'expéditeur d'en déclarer une.

## Vérifications

- `cargo fmt --all -- --check` : OK. `cargo clippy --workspace --all-targets -- -D warnings` : OK
  après simplification d'une condition imbriquée.
- Recette complète : **1519 réussis, 0 échec, 52 ignorés**, dont trois tests nouveaux : survie à une
  attente dix fois supérieure à l'ancien seuil, respect d'une échéance dépassée, borne de file.
