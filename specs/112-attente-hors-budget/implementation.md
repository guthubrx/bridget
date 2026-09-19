# Journal 112 — L'attente en file ne consomme pas le budget de tour

- **Base** : main `dd195ccb` — **Date** : 2026-09-19 — **Statut** : In Progress (livraison)

## Diagnostic
Après la 111, le pont ne jetait plus une remise au bout de deux minutes, mais il appliquait avant
dispatch l'échéance de tour posée par le daemon (`deadline_at = poussée + 2 700 s` pour Codex).
Un destinataire occupé plus de 45 minutes perdait donc encore ses messages. `horizon-3D` était en
tour continu depuis 16h14 ; la libération de 16h07 était condamnée à 16h52.

## Correction (`t3code.rs`)
- `deadline_after_wait(deadline, received)` crédite l'échéance du temps passé en file.
- Les deux contrôles avant dispatch n'écartent une remise pour échéance de tour que si elle était
  déjà passée à la réception (le report ne ressuscite rien).
- Au dispatch, `deadline_at` est reporté sur le message injecté : le budget d'exécution court à
  partir du dispatch effectif, pour l'enveloppe et les contrôles ultérieurs.
- `reply_timeout`, expiration de saga et demande suivie : inchangés. Aucun changement côté daemon.

## Vérifications
fmt OK ; clippy OK ; recette complète **1521 réussis, 0 échec, 52 ignorés**, dont deux tests 112 :
budget de 45 min reçu il y a une heure toujours en file avec budget entier ; échéance déjà passée à
la réception écartée.

## Observation de terrain (avant livraison de la 112)
Grâce à la 111 seule, à 16h51 le tour de `horizon-3D` s'est terminé et le pont a injecté les six
remises en attente en 35 secondes, toutes accusées. La 112 couvre le cas où le tour aurait duré
au-delà de 45 minutes.
