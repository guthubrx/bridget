# Implémentation — Maicie v3

- Dérogation T002 : `sc005_sc006_persistance_arrets_cooperatifs_et_reconciliation_sigkill` est rouge hors périmètre Maicie, à cause d'un temporaire `fleet.json.0.tmp` non unique entre processus 009 ; le correctif est assigné séparément.
- Observation T006 : une seconde passe workspace a reproduit le flake hors
  périmètre `managed_parity_test::matrice_fr008_compare_le_meme_corpus_et_les_frames_attach`
  (`Socket is not connected`, ligne 554). La première passe workspace et la
  relance isolée ont réussi ; aucun chemin T006 n'est présent dans ce test.
