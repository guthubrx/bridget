# Tâches 117
Statut: Implemented — 3/3, livré le 2026-09-24 à 21:49. Cause du blocage du 24/09 non reproduite : à lire au prochain blocage.
- [x] T001 `note_journal_block` / `clear_journal_block` et instrumentation des cinq sorties muettes (`t3code.rs`).
- [x] T002 Test `spec117_blocage_du_journal_signale_une_fois_apres_le_delai` ; `t3code` 83/83 ; clippy OK ; recette complète 1544 réussis, 3 échecs de charge (`search_104_test`, 23/23 seul deux fois).
- [x] T003 Livraison et vérification sur `opus-city-glmwrk1` : observation rétablie après la relance de 21:49, aucun blocage persistant signalé.

## Suite
La correction de la cause attend sa récidive : le journal du pont nommera alors le motif
(« observation du fil … suspendue depuis … »).
