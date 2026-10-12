# Comparaison T013159A — deux runs ciblés clos

**903/903 références PASS identiques**, aux noms/package/cibles exacts. Les6ignored restent identiques. Aucun nom absent ni statut changé.

| Ensemble | Run principal | Références exactes |
|---|---|---|
| Daemon | 559PASS/0FAIL/4ignored, RC0,173,20s | 543/543PASS +4ignored |
| Transport | 361PASS/0FAIL/2ignored, RC0,49,24s | 360/360PASS +2ignored |
| Cumul principal distinct | 920PASS/0FAIL/6ignored | 903PASS référence +17PASS supplémentaires |

**922PASS est le cumul brut**, incluant deux résumés enfants transport fsutil (1PASS chacun). Le nom `fsutil::tests::atomic_temporary_names_are_unique_across_processes` apparaît troisfois. Le résumé parent lib327PASS/1ignored compte déjà ce test. Les deux occurrences enfant sont exclues du total principal, tout en restant consignées dans le JSON.

La FULL unique et les pannesE2E restent àclôturer par ROOT. Aucun GREEN global. Aucun cochage.

## Reçus et gardes sources

/Users/moi/.cache/bridget-tests159A-20261012/T013-T001-daemon-regression.json
SHA reçu : `88e63ac3f7f8faba9c92f55277bb7eff8bc0c00121a5e4b21c02bd58e1f35d22`
/Users/moi/.cache/bridget-tests159A-20261012/T013-T001-daemon-regression.log
SHA journal : `36f7a9e3524e20e28dc76dbab993fa81700423b2da1b1211fd231f6935b3712b`
222sources identiques avant/après dans ce run.

/Users/moi/.cache/bridget-tests159A-20261012/T013-T001-transport-regression.json
SHA reçu : `9d000b9a7ae02d9f54689af0c01e57157d77c7963d2c48d2e2b8cd7fa3659382`
/Users/moi/.cache/bridget-tests159A-20261012/T013-T001-transport-regression.log
SHA journal : `3c4449b4f23fa71bbe6be1403721f5c9f25b81c55b4b4654f9381f5e56f050d5`
222sources identiques avant/après dans ce run.

## Réglage fermé

Les deux oracles fermés exacts ont passé dans le précédent T00522PASS. Ils ne sont pas sélectionnés dans les filtres T013 actuels. Leur preuve reste attachée à son propre SHA ; les fichiers daemon/registry ont changé depuis. Aucune réattribution au dernier source. La FULL unique prescrite reste prévue.

- `daemon::presence_tests::held159_hold_ferme_identique` : ok historique
- `registry::held159_hold_ferme_identique` : ok historique

/Users/moi/.cache/bridget-tests159A-20261012/T005-admission-T009-list-GREEN-capacity-type.json
/Users/moi/.cache/bridget-tests159A-20261012/T005-admission-T009-list-GREEN-capacity-type.log

## PASS supplémentaires

- `idempotency::send_delivery::held159_acked_at_premier_ack_et_rejeu` (bridget-daemon/lib)
- `idempotency::send_delivery::held159_rank_monotone_sous_concurrence` (bridget-daemon/lib)
- `idempotency::send_delivery::held159_rank_orphelinage_couvre_les_retenus` (bridget-daemon/lib)
- `idempotency::send_delivery::held159_rank_reprise_dans_l_ordre` (bridget-daemon/lib)
- `idempotency::send_delivery::held159_rank_reprise_ne_rend_pas_les_retenus` (bridget-daemon/lib)
- `idempotency::send_delivery::held159_rank_sans_collision_entre_instances` (bridget-daemon/lib)
- `idempotency::send_delivery::held159_rank_survit_a_la_reaffectation` (bridget-daemon/lib)
- `idempotency::tests::held159_migration_fantomes_ecartes_avec_warn` (bridget-daemon/lib)
- `idempotency::tests::held159_migration_liens_execution_conserves` (bridget-daemon/lib)
- `idempotency::tests::held159_migration_ordre_preserve` (bridget-daemon/lib)
- `idempotency::tests::held159_migration_phases_held_et_cancelled_admises` (bridget-daemon/lib)
- `idempotency::tests::held159_migration_recipient_agent_rempli` (bridget-daemon/lib)
- `idempotency::tests::held159_migration_rejeu_sans_effet` (bridget-daemon/lib)
- `idempotency::tests::held159_migration_rollback_si_echec` (bridget-daemon/lib)
- `idempotency::tests::held159_migration_turn_gate_defaut_zero` (bridget-daemon/lib)
- `idempotency::tests::held159_migration_sans_perte` (bridget-daemon/lib)
- `protocol::tests::held159_turnstate_register_sans_turn_attest_octets_inchanges` (bridget-transport/lib)

Les903comparaisons exactes et les preuves de sérialisation/phase fermées restent séparées des douzeFAIL158B historiques. Le seulnom ne justifie jamais de masquer un futurFAIL.

JSON détaillé :
/Users/moi/.cache/bridget-validation159A-T013-preparation-20261012/comparison-T013.json
Préparation historique/commandes/tableau12FAIL :
/Users/moi/.cache/bridget-validation159A-T013-preparation-20261012/preparation-T013.json

Aucun test, build, service, écritureWT ou Git exécuté par cet agent.
