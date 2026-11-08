# Tâches 244

- [x] T001 Spécification et plan (d7ecaeda)
- [x] T002 L1/E3 : sortie d'erreur du bootstrap de qualification gardée,
      fiche d'échec privée `<sha>.failed.json`, refus nommé
      `program_check_unavailable` après 600 s d'échec ; test
      `spec244_echec_durable_de_qualification_devient_un_refus_nomme` (Mac PASS)
- [x] T003 L1 : tests Linux, build, activation quatre hôtes
- [ ] T009 PRIORITÉ — L6/E6.3 messages : sur les 2 000 dernières remises du
      maître, 251 `indeterminate` (12 %) et 193 `orphaned` (10 %). Pics les
      jours de livraison (31/10, 01/11). `orphaned` : remises abandonnées à la
      purge de présence (`orphan_dispatching_for_instance`) au lieu d'attendre
      le retour de la même identité. `indeterminate` : vu sans preuve
      d'injection, jamais réessayé (`mark_delivery_indeterminate`). Signalement
      revue145 du 8 novembre (mcp-22423-6af02814-1 indeterminate,
      mcp-22423-6af028f0-2 acked plus tard). Objectif : file conservée par
      identité jusqu'au retour, et preuve d'arrivée fiable avant de conclure.
- [x] T004 L2/E4 : test d'exhaustivité des guidages de refus, compléments
- [ ] T005 L3/E2 : réparation automatique des mappings sans racine additionnelle
- [ ] T006 L4/E1 : `bridget sentinel run` (délégations + messages, agents
      détachés `spawn` sur Cartae, sans T3), rapport `sentinel.json`
- [ ] T007 L5/E1-E5 : minuteur quotidien, déclencheurs, alerte unique
- [ ] T008 L6/E6 : réparation automatique des composants, statut de panne
      lisible, file de messages conservée, panne simulée réparée sans T3

Preuve préalable (8 novembre 2026) : `bridget spawn claude --no-persistent`
lancé sur Cartae crée un agent attesté sans T3 (c39bc322, claude_stream_json).

Reprise OpenAI : sous-tâches explicites sans réduction du périmètre.
- [x] T009a Conservation dispatching/held après purge et fermeture avant injection ; doublon vivant attend son ACK unique (testsMac).
- [x] T009b Réconciliation positive après disparition du tracker ; aucun renvoi d’une injection ambiguë.
- [x] T009c File durable sous saturation et réception différée sans boucle ni quota arbitraire.
- [x] T004a Guidages exhaustifs des23refus worker, texte source fermé (testMac).
- [x] T004b Compléter l’inventaire des refus d’admission et leurs guidages manquants.
- [x] T005a Copie stricte d’une racine additionnelle déjà approuvée, révision+1, sauvegarde/CAS et7testsPython.
- [ ] T005b Qualification Linux et publication réelle des copies source/maître/worker dans le parcours natif.

Point de livraison du8novembre :0.8.10/R5 active, deux incidents prioritaires corrigés et avis de reprise remis. Les implémentations CLI/backend/Android de la sentinelle sont livrées, mais la matrice complète, le minuteur et la panne simulée ne sont pas qualifiés : T006/T007/T008 restent ouverts. À la demande de François, aucun nouveau contrôle Claude ni prolongement de campagne ; quota fournisseur extérieur à Bridget. Les deux seuls parents de test ont été arrêtés par leur commande native, sans perdre leurs résultats ni changer leurs clés. T005b reste partiel : préparation locale corrigée, lien Cartae→Mac absent, aucun privilège ajouté.
