# Session 039 — Stabilisation du témoin de reçus idempotents

## Défaut

`wrapper::reconnect_tests::livraison_idempotente_interactive_injecte_une_fois_et_rejoue_l_accuse`
échoue parfois à sa propre réouverture du `ReceiptStore` :
`store de reçus déjà ouvert pour cette instance`.

## Mesure avant correction

- Base initiale `2561dce` : campagne complète de 544 tests, 1 échec ciblé sur 10 passages ;
  le sous-ensemble `wrapper::reconnect_tests` (51 tests) est vert 10/10.
- Tête `1d0f39a` : ni `wrapper.rs` ni `receipt_store.rs` n'ont changé ; campagne complète
  de 547 tests, 0 échec ciblé sur 10 passages.
- La trace montre le même fil qui acquiert puis tente de reprendre le même verrou unique
  (PID + UUID) ; aucun second accès à ce chemin n'a été observé.
- L'hypothèse d'un descripteur hérité dans une fenêtre fork/exec reste plausible, mais non
  attestée pendant un échec réel.

## Décision

Isoler l'oracle de reconnexion dans un sous-processus exécutant ce seul test, avec une attente
bornée. C'est une isolation du banc, pas une correction déclarée de la cause hypothétique.

## Acceptation

1. Le sous-processus vérifie toujours la première injection, l'accusé et le rejeu sans seconde
   injection.
2. La campagne complète est mesurée au moins dix fois après correction.
3. Le mutant supprimant `drop(tracker)` rend l'oracle rouge.
