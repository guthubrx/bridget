# Tâches 244

- [x] T001 Spécification et plan (d7ecaeda)
- [x] T002 L1/E3 : sortie d'erreur du bootstrap de qualification gardée,
      fiche d'échec privée `<sha>.failed.json`, refus nommé
      `program_check_unavailable` après 600 s d'échec ; test
      `spec244_echec_durable_de_qualification_devient_un_refus_nomme` (Mac PASS)
- [ ] T003 L1 : tests Linux, build, activation quatre hôtes
- [ ] T004 L2/E4 : test d'exhaustivité des guidages de refus, compléments
- [ ] T005 L3/E2 : réparation automatique des mappings sans racine additionnelle
- [ ] T006 L4/E1 : `bridget sentinel run` (délégations + messages, agents
      détachés `spawn` sur Cartae, sans T3), rapport `sentinel.json`
- [ ] T007 L5/E1-E5 : minuteur quotidien, déclencheurs, alerte unique
- [ ] T008 L6/E6 : réparation automatique des composants, statut de panne
      lisible, file de messages conservée, panne simulée réparée sans T3

Preuve préalable (8 novembre 2026) : `bridget spawn claude --no-persistent`
lancé sur Cartae crée un agent attesté sans T3 (c39bc322, claude_stream_json).
