# Tâches 121
Statut: Implemented - 5/5.
- [x] T001 `spontaneous_turn`, `spontaneous_origin` : tour sans message déclencheur prouvé spontané si la page couvre sa demande ; gardé observable, fin journalisée avec `stop_reason` (`t3code.rs`).
- [x] T002 Refus d'écriture de message et de fin de tour signalés par `note_journal_block` (117) au lieu d'un avertissement à chaque lecture.
- [x] T003 `uncertain_answer` : demande sans appariement certain, fil au repos : texte suivant la demande jusqu'au message utilisateur suivant, préfixé d'un avertissement.
- [x] T004 Tests `spec121_*` (3) ; le test d'observabilité échoue sans la détection ; t3code 90/90.
- [x] T005 fmt, clippy ; recette 1555 réussis, 1 échec `service_capability_is_cleaned_after_a_broken_response_socket` à charge 111 (3/3 seul, hors périmètre).
