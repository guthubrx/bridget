# Audit158B — scoring final ciblé

## Résumé exécutif

GradeA,numérique99.9500.0C/0H/0M/1L. Couverturepartielle7.9% :32/405hunks entièrementlus,17/39fichiersavecplagesadmissibles. Pas couverture de tests instrumentée.

## Tendance

C1historique0C0H5M1L ;cinqMrelus résolus ;MIN001LOW096f382ea2c7 conservéROOT,pascleanupconfort. Aucungrade157réutilisé. Sourcefinaledaemon2124 ;39files18Rust405hunks.

## Findings et corrections

PERF001 annotationclientO(B),PERF003 borneStoreCowner,PERF002guardemptyNone,TEST001CLI-filattesté12/13/14dansoraclesexistants,REL001FR106alignement. Preuvesexactes/hashs dansagrégats etfix-report. ClockcandidateC1nonfinding :suppl2124deuxcallers+puitsG5/G8,couverturepartielle seule. P3CONVERGED/R2APPROVE_WITH_CHANGEStriageROOT. Deferredavantdépart admis ;clôtureaupassageultérieur prouvéeparlecture,pasruntime secondtick inventé.

## Minimalisme & Frugalité

Potentiel minimalisme : ~2 lignes suppressibles à comportement constant. MIN001gardé ;aucunretrait demandé. Réemploi099/102 ettroissources,aucunframework/service/harnessneuf supposé.

## Vertus LLM & Responsabilité Future

[Note technique de l’adaptateur](../../../../docs/adapters/t3/archives/specs/158-reveils-controle/validation/audit158B-final/scoring.md#extrait-1).

## Tests et limites

FULLuniquehistorique2456PASS/12FAIL/74ignored,1356,44sRC101 ;replaysaffectés cumulés sans inventer unsecondFULL niGLOBALGREEN. DernierROOT4PASS0FAIL22,51s221sourcesstables,receiptT109-T117c3feb4ada748ebd22470b9cbdf5192031e322f0b3591b28cfe1ca0fe0ae77e4b. AucunCargoici.

## Couverture et mémoire négative

FichierentierSHAexact requis. Anciendaemon42exclu,seul supplément2124partiel crédité. Date réelle13:01:31 conservée,ROOTautoriseégalitécodeavantcapturemetadata. Contextes/selfreview nonindépendants exclus. Hunkspartiels/effacementsseuls ne gonflentpascomplétude. Détailsread_manifest-aggregation.json ;T122 ne prouvepaslecturemétierautomatique.

## Findings supprimés par la baseline

Aucun :baselineabsente vérifiée,non écrite.

## Points positifs

- {"file": "crates/bridget-daemon/src/store/observation_missed.rs", "line": 177, "text": "Lotvide sans snapshot court-circuité avantTX ; snapshotSome conservé."}
- {"file": "crates/bridget-daemon/src/daemon.rs", "line": 2115, "text": "Deadlinewriter=min(clock+1s,Monoactuel+1s), cap physique même si horloge de fixture future."}
- {"file": "crates/bridget-daemon/src/daemon.rs", "line": 2466, "text": "Horloge fraîche recalculée après réserveSQLite et avant begin_send_delivery ; aucun nouveau départ àexpiration."}
- daemon2106–2186 : reçu102/canon commun recherché avantTTL et gardesmutables ; ErrStore distinct deNone.
- daemon2230–2271/2392–2483 : cinq lecturesWall comptent ancrage initial + quatre relectures, jamais heure unique cachée avant attenteSQL/writer.
- daemon2935–3000 : once/provenance avantworker borné, absence route incrementeperte et solde FailedBeforeWrite, Full gardepending+5s.
- Storemissed177–184 : garde batchvide avantTX ; busy_timeout restauré aux deux résultats271–288.
- CLI10267–10359/10639–10713 : mêmes fixturesRegistered/credential149 réutilisées, daemon privé seulementenfant, DBROapresexit ; aucun nouveauframework.
- FR106spec230/research43/data-model55–59 : protectionowners+globalgap alignée, aucune cascade de faits protégés.
- Storemissed:179 — no-op lorsque notices vide ET snapshotNone ;snapshotonce présent continue la transaction. Aucun cache/table/helper supplémentaire.
