# Matrice A14 — preuves prêtes au cochage ROOT

A14/B14. P3 + annexe CONVERGED, R2 traitée, audit final A99,95 partiel. Aucun cochage exécuté ici. Les empreintes et exécutions restent propres à chaque reçu ; aucun GREEN global.

| Tâche | Porte probatoire | Preuve |
|---|---|---|
| T001 | Référence close :903 PASS/6 ignored cumulés, deux FAIL initiaux et replays conservés. Latence simulée distincte. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/validation/baseline.md |
| T002 | Dix axes relus, source/autorité cadrées ; aucune preuve runtime déduite de la prélecture. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/validation/T002-read-manifest.json |
| T003 | Migration10 PASS, acked_at complément ; T011 réel ferme la restauration. Synthétiques liens non vides distincts. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/validation/T003-T004-progress159A.json |
| T004 | Dix rang/rattachement PASS ; adoption ordinaire corrigée et ordre E2E strict conservé. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/validation/T004-T006-progress159A.json |
| T005 | 15 hold/registry PASS ; QueueOnly corrigé et repris dans GREEN 54. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/validation/T013-R1-PERF-existing-groups-GREEN-summary.json |
| T006 | Signal/horloges ; R2 RED 1 puis GREEN 8 courant abf654 ; pas de false fabriqué. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/validation/T006-R2-turnstate-group-GREEN-summary.json |
| T007 | 20 release+ACK1 PASS ; réservation/pause/reprise/commit avant push, GREEN 54 distinct. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/validation/progress-T007-T008.json |
| T008 | 11 sweep PASS ; retain après COMMIT/erreur tardive lu, pas de framework. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/validation/T013-R1-PERF-existing-groups-GREEN-summary.json |
| T009 | Sept List PASS ; ownBareClient fermeture T028 ci-dessous. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/validation/T028-queue-list-existing-group-GREEN-summary.json |
| T010 | Onze cas de matrice partagés, tout prescrit ciblé ; SC004proxy seuil dépassé, défaut fermé. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/validation/T010-matrice.json |
| T011 | PASS réel6→7→6 debug, ancienrefuse, restore ; privé/summarysanscorps, pas rollbackprod. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/validation/restauration-v7.json |
| T012 | Référence/skill/commandes/ADR appliquésROOT ; sauvegardePROD datée acquise. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/docs/decisions/159-file-retenue-et-edition.md |
| T013 | 920P ciblés, FULLhistorique9 FAIL tousrejouésPASS ; relecture/P3/audit bornés et courant abf654 viaannexe. | /Users/moi/.cache/bridget-converge159A-pass3-20261012/converge159A-P3-R2-annexe.json |
| T028 | RED 1F15,79 s attendu2/reçu1 ; GREEN 7P11,64 s ; getter SELECT/droits156conservés. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/specs/159-edition-file/validation/T028-queue-list-existing-group-GREEN-summary.json |

La livraison de production reste à exécuter par ROOT. Les preuves T011 debug et les anciennes mesures ne sont pas réattribuées à un futur release.
