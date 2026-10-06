# Tâches 135 — Contrôle silencieux des missions

## Phase 1 — Préparation

- [x] T001 Créer le worktree, relire le heartbeat, observer la boucle Politique
  et consigner la cause dans le plan.
- [x] T002 Faire la contre-revue adverse du plan et intégrer les corrections
  vérifiables.

## Phase 2 — Contrat durable

- [x] T003 Ajouter les tests rouges des métadonnées de mission, de `progress`
  et de la migration v2.
- [x] T004 Implémenter les politiques, métadonnées, `progress` et `migrate-run`
  dans `/Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py`.

## Phase 3 — Heartbeat silencieux et escalade

- [x] T005 Ajouter les tests rouges de déduplication, seuils, groupement et
  séquence worker, coordinateur, escalade, y compris un destinataire injoignable
  et une pause limitée à une nature de travail.
- [x] T006 Remplacer le backoff par âge par le registre durable d'anomalies et
  le plan de notification groupé.

## Phase 4 — Décision et clôture

- [x] T007 Ajouter les tests rouges de décision terminale, intégrité du verdict,
  absence de faux achèvement et clôture explicite.
- [x] T008 Implémenter `disposition`, le contrôle du run ouvert et `close-run`.

## Phase 5 — Documentation et validation

- [x] T009 Mettre à jour `/Users/moi/.codex/skills/agent-loop/SKILL.md` et le
  quickstart, puis exécuter les tests complets, la compilation Python et la
  validation de skill.
- [x] T010 Faire la contre-revue adverse post-implémentation, converger les
  artefacts et exécuter l'audit final.

## Phase 6 — Mise en service Politique

- [x] T011 Sauvegarder le run Politique, rattacher ROOT, migrer le run, relancer
  le LaunchAgent et vérifier un passage réel sans perte ni rafale.

## Dépendances

## Reprise corrective systémique — 2026-10-06

- [x] T012 Recenser les producteurs actifs et les copies de compétences ; prouver
  les répétitions Psychologie, les fausses déconnexions busy et les inscriptions CLI.
- [x] T013 Extraire un moteur de notification partagé pour v2, legacy et
  Psychologie ; conserver validations métier, relances et preuves de progrès.
- [x] T014 Tester réapparition, nouvelle tentative, migration et crash ancien ;
  remplacer les copies Claude par des entrées canoniques et valider les suites.
- [x] T015 Remettre les deux minuteurs en service, observer deux passages planifiés
  et comparer tâches/résultats à la sauvegarde.
- [x] T016 Préserver un digest en vol si de nouveaux faits arrivent après un
  crash ; tester la reprise et refuser toute association historique ambiguë.
- [x] T017 Livrer les sources et documents, vérifier les fusions et le push,
  puis retirer uniquement les builds inutilisés et les worktrees fusionnés.

## Dépendances initiales

T001 précède T002. T002 précède T003. T003 précède T004. T005 précède T006.
T007 précède T008. T004, T006 et T008 précèdent T009. T009 précède T010. T010
précède T011.

## Estimation recalibrée

11 tâches. La modification centrale touche un script de 3 342 lignes et sa
suite de 1 303 lignes. Fin estimée entre 10:30 et 10:50 CEST selon les retours
de contre-revue et l'état réel du LaunchAgent.
