# Tâches 119
Statut: Implemented - 5/5.
- [x] T001 Mesure : 222 avis vers `sol_city_ai`, 197 tours démarrés, 37 réponses de simple commentaire ; T3 n'offre qu'une activité invisible pour l'agent (piste écartée).
- [x] T002 `Observations::due_source_notices` : état immédiat, avis après 30 s de stabilité, aller-retour bref silencieux, « source instable » au plus toutes les 5 min (`observation.rs`) ; émis par la boucle d'une seconde du daemon (`daemon.rs`).
- [x] T003 Tests `spec119_*` (2), test unitaire 101 et test d'intégration `spec101_real_daemon_journal_share_collision_and_restart` adaptés au nouveau contrat.
- [x] T004 Documentation : `docs/reference-communication.md`, `skills/bridget/references/commandes.md`.
- [x] T005 fmt, clippy ; recette complète 1545 réussis, 7 échecs à charge 62-65 : `spec101_real_daemon…` (contrat, corrigé, 1/1), `managed_parity_test` (7/7 seul), `core_089_content_test` (7/7 seul), `search_104_test` s22/s24/s25-26 instable à l'identique sur main (5/10 échecs pour chacun, essais alternés).
