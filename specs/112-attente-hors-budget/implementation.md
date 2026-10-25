# Journal 112 — L'attente en file ne consomme pas le budget de tour

- **Base** : main `dd195ccb` — **Date** : 2026-09-19 — **Statut** : Implemented, livré en production le 2026-09-19 17:01

## Diagnostic
Après la 111, le pont ne jetait plus une remise au bout de deux minutes, mais il appliquait avant
dispatch l'échéance de tour posée par le daemon (`deadline_at = poussée + 2 700 s` pour Codex).
Un destinataire occupé plus de 45 minutes perdait donc encore ses messages. `horizon-3D` était en
tour continu depuis 16h14 ; la libération de 16h07 était condamnée à 16h52.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/112-attente-hors-budget/implementation.md#extrait-1).

## Vérifications
fmt OK ; clippy OK ; recette complète **1521 réussis, 0 échec, 52 ignorés**, dont deux tests 112 :
budget de 45 min reçu il y a une heure toujours en file avec budget entier ; échéance déjà passée à
la réception écartée.

## Observation de terrain (avant livraison de la 112)
Grâce à la 111 seule, à 16h51 le tour de `horizon-3D` s'est terminé et le pont a injecté les six
remises en attente en 35 secondes, toutes accusées. La 112 couvre le cas où le tour aurait duré
au-delà de 45 minutes.

## T004 — Livraison
Binaire reconstruit, daemon et pont relancés à 17:01, quinze agents reconnectés. Plus aucune remise en attente vers `horizon-3D`.
