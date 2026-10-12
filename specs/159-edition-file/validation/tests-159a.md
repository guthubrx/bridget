# Tests 159A — état final de validation

A14/B14 ; gates techniques closes, cochage ROOT puis livraison attendus. Une seule FULL historique :2551 PASS/9 FAIL/75 ignored,RC101, 779,06 s. Neuf noms originaux ont chacun un dernier PASS ciblé, même package/cible/nom. Pas de seconde FULL, pas de GREEN global ni 2551 tests attribués au source courant.

| Chronologie | Résultat et limite |
|---|---|
| T001 référence |903 PASS/6 ignored cumulés ; deux FAIL initiaux restent tracés et replays distincts. |
| T003–T009 | Migration10 ; rang/adoption10 ; hold15/list7 ; signal9cumulés ; release20+ACK1/sweep11. Groupes propres, sans somme comme nouveaux tests uniques. |
| T013 ciblé |559daemon+361transport=920 PASS principaux,6 ignored ;903/903 PASS de référence inchangés ; deux résumés enfants transport exclus. |
| T010 | Onze cas partagés, preuves ciblées ; Seen16 FAIL de montage puis17exact1 PASS23,49 s. Cause syscallEPERM non instrumentée ; ni fauxSeen ni ACK/terminal artificiel. |
| R1/PERF | Trois oracles existants RED 0P3F18,26 s ; GREEN 54P0F14,56 s. QueueOnly, deuxhorloges silence/busy, retain après COMMIT/erreur tardive. |
| Replays FULL | Neuf derniers PASS exacts ; runs intermédiaires3P3F et2P1F gardentRC101. Support négatif cancel distinct ; Python098 hors222 mapsRust/Cargo, garde/provenance explicitée. WarningPID15797 : ROOT rapporte disparition, pas faux reçu ps indépendant. |
| T028 | OwnBareClient négocié :RED 0P1F15,79 s attendu2/reçu1, puis GREEN 7P0F11,64 s. Scope/clé/delivery_id inchangés ; getter SELECT pur et list_ne_modifie_rien, data_version seul limité. |
| R2 FR023 | Même oracleT006 :RED 0P1F11,14 s ; GREEN 8P0F7,08 s sur abf654,222 sourcesAV/APégales. True tardif conserve busy_period_start jusqu'au false. |

SC004 : p95avant27,763792 ms/après 1028,132375 ms ; delta 1000,368583 ms>250 ms. SamplerPASS54,72 s, mais défaut fermé. ProxyPython, pas horloge daemon_idle ni SLA ;20échantillons/rang19. Précondition busy après supplémentaire hors mesure, fixtures non byte-identiques. Métadonnées anciennes conservée historique ; valeur inchangée,54,72 s/logdb134 corrigés.

T011 : debug48c5… réel6→7→6 ; ancien158B refusev7RC1, restaurev6RC0 ; hash/tables/samples/rang égaux,quick_check=ok/FK0. Premier FAIL expliqué par maintenance historique 7jours, pas normalisation. Acked/indeterminate et zéro lien réels ; autresphases/liens synthétiques T003 distincts. ExactJSONprivé5853… jamaisGit ; résumé publice556… sanscorps. BackupPRODv6 réel ef4c… acquis à17:14:26UTC,native0quickOKFK0 ; préserver faits plus récents avant rollback, pas rollbackexecuté.

R2 brute terminale completed/result_available, hasPendingChildRuns=false ; lue/acquittée par ROOT. APPROVE_WITH_CHANGES ; FR023 corrigé par ROOT/GREEN 8. Reviewer sans tests ; queue_list, sweep, branche schedule complète et caller verrou non lus ; FULL/SC004/tâches non vérifiés par lui. Annexe P3 d121… CONVERGED ferme le delta sur abf654 sans P4 ni R3 massif.

Audit final A99,95,0C/0H/0M/1LOW DUP accepté ; validateurRC0/0E/0W. Partiel16/64 et4275/174030 lignes (2,4565%) ; guard64AV/APégales, seuldaemon changé vs18ad. Ancienne capture de 448 hunks historique, pas diffglobalcourant. Source actuel daemon :
abf654cd44116dc277c80b3f9ff9e188e6ba5dae47d5c03df75ddbcdbbdaaddc

Preuves exactes et SHA par fichier dans :
/Users/moi/.cache/bridget-docs159A-final-preparation-20261012/final-closed-copy-manifest159A.json
/Users/moi/.cache/bridget-docs159A-final-preparation-20261012/matrice-A14-gates.json
Les corps réels/blob_hex et DB/binaire rollback restent privés horsGit. Aucune modification WT, test, Cargo, Git, DB ou service par cet owner documentaire.
