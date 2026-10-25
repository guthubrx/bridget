# Archives techniques de l’adaptateur T3

Document d’origine : [specs/155-demandes-enfant/validation/audit155-final-scoring.md](../../../../../../../specs/155-demandes-enfant/validation/audit155-final-scoring.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Agrégateur : aucun test, build, modèle, service ou validate_session exécuté.
- T040 validated_four_cases_with_explicit_user_consent,42/42 vérifiés ; trois captures runtime antérieures restent historiques, aucun GREEN global final déduit.
- child_request_followups reste O(n+B) ; LIMIT512 borne la projection, pas le travail SQL. Aucun EXPLAIN exécuté.
- q² reste borné par les données ; le commentaire exact corrige PERF-041, sans convertir cette lecture en recommandation d’optimisation.
- Actualisation ciblée du delta cwd sur capture03:34 ; un seul cycle thématique1. La passe précédente est historique et archivée, jamais redatée comme fraîche.
- read_manifest :10 entrées fraîchement lues après capture et12 héritées du C1 canonique uniquement si SHA inchangé, dates/ranges initiales intactes et provenance absolue vérifiée.
- Les fichiers modifiés depuis C1 ne gardent aucun crédit d’ancienne lecture ; leurs seules plages fraîches du présent delta sont comptées. La couverture peut donc différer de la passe pré-cwd.
- Aucun test, build, Cargo, lint, scanner, modèle, service, Git en écriture, script de préparation ou agrégation exécuté par ce lecteur.
- Aucun score global, couverture exhaustive, complexité cyclomatique ou duplication globale calculés.
- Reçu post-cwd native155-cwd-r1 RC0,71PASS,214sources stables. Les quatre régressions canal/STDIO et recette réelle T0404/4 conservent leurs provenances antérieures à cwd. Aucun total75 sur dernier snapshot.
- T040 validée et42/42 ; suite complète unique antérieure2259PASS/15FAIL/72ignored, baseline180PASS, sept nouveaux échecs corrigés ciblés et huit historiques conservés. Aucun nouveauFULL ni GREEN global.
- DUP-001 LOW c0dcaaf9b24b persiste ; preuve exacte13lignes/2branches relue sur snapshot courant. Aucun JSCPD exécuté.
- PERF-082 reste LOW/THEORETICAL : k budgets individuels peuvent s’accumuler ; aucun benchmark ni optimisation supplémentaire demandés.
- Le canal humain qui ignore SIGTERM peut survivre. L’arrêt forcé et le groupe de processus ne sont pas utilisés.
- 71PASS post-cwd sont attribués au principal ;4régressions additionnelles et recette4/4 restentpré-cwd. AucunGREENglobal.
- Revalidation post-cwd ciblée seulement, sans nouveau cycle thématique. Les lectures pré-cwd ne sont pas réétiquetées fraîches ; elles restent archivées séparément.
- Aucun test, Cargo, build, scanner, service, modèle ou Git exécuté par ce lecteur. ROOT seul a exécuté native155-cwd-r1 :71PASS/0FAIL/RC0/64,81s,214sources stables.
- Les lectures héritées viennent exclusivement des rapports C1 canoniques archivés, avec SHAinitial/final identiques, ranges et dates originales exacts. Leurs dates read_at_utc absentes restent absentes ; vérification SHA datée séparément.
- 61 fichiers/244hunks inventoriés ne sont pas une lecture.284sources globales seulement du contexte. La couverture baisse car les fichiers changés exigent les seules plages relues, pas une reconduction de leur ancienne lecture entière.
- Full historique avant correctifs2259PASS/15FAIL/72ignored,180baseline PASS ; sept nouveaux échecs repris en PASS ciblé, huit historiques conservés. Aucun FULL sur le dernier hash ni GREEN global.
- Les quatre régressions et la recette T040 réelle antérieures ne sont pas déclarées sur le dernier code cwd. T040 consentement explicite/quatre cas reste une preuve historique identifiée avec ses réutilisations, sans nouvelle recette par ce lecteur.
- Résolutions SEC-001/REL-001 et R2-cwd établies par relecture statique fraîche et reçusROOT cités ; aucun exploit ou banc supplémentaire exécuté par le lecteur.
- Le STDIO est borné à3s ; le SocketWriter interactif conserve un flush distinct pouvant ajouter3s après contention, limite explicite de la source.
- M2 : source post-cwd relue sans permis avant write=>sent réessayable ; erreur avec permis=>unknown. Son nom exact PASS appartient au reçuROOTcwd, sans nouvelle exécution.
- H3 : appel du canal hors state relu aprèscwd. La borne du programme externe3s+250msTERM et sa survie possible en ignorantTERM restent une preuve pré-cwd àSHAinchangé, pas une nouvelle lecture sémantique de human_inbox.rs dans cette revalidation.
- Couverture basse 27.46% : hunks entièrement couverts par des ranges explicites ; inventaire exclu.
- Aucun CRITICAL/HIGH actif : aucun contre-audit supplémentaire requis.
- Inventaire uniquement : couverture sémantique zéro, aucun cycle, finding ou score.
- Script154 réutilisé pour la forme ; aucun résultat d’audit154 transposé.
- Aucun test, build, service, changement Git ou activation produit par ce script.
- Diff limité155 ; T3/UI et travaux étrangers exclus.
- Workspace complet antérieur aux correctifs : 2259PASS/15FAIL/72ignored ; aucun GREEN global final.
- Sept nouveaux échecs corrigés et rejoués RC0 ; huit échecs historiques conservés.
- Baseline180 PASS acquise dans le passage complet antérieur ; aucun rejeu180 indépendant final.
- Native155-postfix71PASS et HTTP099-final-r2 1PASS restent des preuves ciblées distinctes.
- T040 validated_four_cases_with_explicit_user_consent,42/42 : quatre cas et consentement explicite vérifiés par le reçu canonique ; trois captures antérieures restent identifiées.
- Converge attesté par --after-converge à l’appel ; aucun Converge exécuté par cette préparation.
- Capture finale séparée : les métadonnées et rapports cycle1 restent byte-identiques ; aucun ancien read_manifest transformé en lecture fraîche.

<a id="extrait-2"></a>

## Extrait 2

Preuves contextuelles du manifeste, avec leurs limites originales :
```json
[
  {
    "path": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/workspace155-final-r1-comparison.json",
    "sha256": "06ec9f61963880ec12baec7fa223afd0342a716a1e8180669c29d0b6ab99c594",
    "kind": "validation_workspace_archivée_R1_par_principal",
    "workspace_totals": {
      "passed": 2259,
      "failed": 15,
      "ignored": 72
    },
    "failure_comparison": {
      "actual_count": 15,
      "legacy_present": 8,
      "new_count": 7,
      "same_failure_targets_and_names": false,
      "new": [
        {
          "target": "Running tests/core_089_skill_test.rs",
          "name": "exemples_skill_envoient_repondent_et_cloturent_sans_service_compagnon"
        },
        {
          "target": "Running unittests src/lib.rs",
          "name": "daemon::native_delegation::native_workspace154_tests::native154_partage_discovery_termine_et_statut_compatibles150"
        },
        {
          "target": "Running unittests src/lib.rs",
          "name": "daemon::native_delegation::tests::native152_rouge_annulation_multi_ticks"
        },
        {
          "target": "Running unittests src/lib.rs",
          "name": "daemon::native_delegation::tests::native152_rouge_annulation_partielle"
        },
        {
          "target": "Running unittests src/lib.rs",
          "name": "mcp::tests::matrice_fr009_couvre_les_quinze_cas"
        },
        {
          "target": "Running unittests src/lib.rs",
          "name": "mcp::tests::native151_ack_facade_schema_ferme_bornes_et_annotations"
        },
        {
          "target": "Running unittests src/lib.rs",
          "name": "t3code::tests::spec099_post_accepte_issue_illisible_conserve_la_correlation"
        }
      ],
      "absent_baseline_failures": [],
      "utc154_fixture_is_never_legacy": true,
      "utc154_observed": [
        {
          "target": "Running tests/core_089_attach_test.rs",
          "status": "ok",
          "annotation": null
        }
      ]
    },
    "baseline180": {
      "index": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/baseline155-index.json",
      "index_sha256": "d563d09a587da6279d9c0b279430548fff082d1c5e0cf1422783c552a18f20ab",
      "matched": 180,
      "missing": 0,
      "regressions": 0,
      "duplicates": 0,
      "wrong_target": 0,
      "scope": "executed in the one full run before corrective patches"
    },
    "scope_limit": "Passage complet avant correctifs ; aucune validation globale du gel final."
  },
  {
    "path": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/final155-cumulative-gate.json",
    "sha256": "61d9fbc874c0db43e712e3a35ba84c11e32c57515e74ad57d129baf0af43106c",
    "kind": "preuves_cumulatives_ciblées_T039_par_principal",
    "resolved_count": 7,
    "new_failures_resolved": [
      {
        "target": "Running tests/core_089_skill_test.rs",
        "name": "exemples_skill_envoient_repondent_et_cloturent_sans_service_compagnon",
        "before_full_status": "FAILED",
        "after_targeted_status": "ok",
        "successful_replays": [
          "skill089-postfix-r1"
        ],
        "authoritative_replay": "skill089-postfix-r1",
        "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/skill089-postfix-r1.json",
        "receipt_sha256": "a6a5d7c4646a407070f79e30ed2d6b88b74d4324e3013ba624c253d7c7fd1077",
        "log": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/skill089-postfix-r1.log",
        "log_sha256": "5fe14050bd619cef0e7bb9bba5272d150ea0080245fbfd10ad2201003b2c0b88",
        "full_after_patch_proof": false
      },
      {
        "target": "Running unittests src/lib.rs",
        "name": "daemon::native_delegation::native_workspace154_tests::native154_partage_discovery_termine_et_statut_compatibles150",
        "before_full_status": "FAILED",
        "after_targeted_status": "ok",
        "successful_replays": [
          "workspace154-schema-postfix-r1"
        ],
        "authoritative_replay": "workspace154-schema-postfix-r1",
        "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/workspace154-schema-postfix-r1.json",
        "receipt_sha256": "9681c33c5516b7bcf75fd3d8b4cc87ace18b524cc4d9a18937f3675a17a7ae21",
        "log": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/workspace154-schema-postfix-r1.log",
        "log_sha256": "e4d3496a06d155915e9f37d362e2f4381b3c09a5239480b022d86174a5a8605b",
        "full_after_patch_proof": false
      },
      {
        "target": "Running unittests src/lib.rs",
        "name": "daemon::native_delegation::tests::native152_rouge_annulation_multi_ticks",
        "before_full_status": "FAILED",
        "after_targeted_status": "ok",
        "successful_replays": [
          "annulation152-postfix-r1"
        ],
        "authoritative_replay": "annulation152-postfix-r1",
        "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/annulation152-postfix-r1.json",
        "receipt_sha256": "0299fe453425d5caea94ac27e4da4b7afce05d3ac07d32bffd15fdd39838a7e4",
        "log": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/annulation152-postfix-r1.log",
        "log_sha256": "ff7cf7a01970f9eebe72010870609cd723d06d342ec08f3618f908dfa678c797",
        "full_after_patch_proof": false
      },
      {
        "target": "Running unittests src/lib.rs",
        "name": "daemon::native_delegation::tests::native152_rouge_annulation_partielle",
        "before_full_status": "FAILED",
        "after_targeted_status": "ok",
        "successful_replays": [
          "annulation152-postfix-r1"
        ],
        "authoritative_replay": "annulation152-postfix-r1",
        "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/annulation152-postfix-r1.json",
        "receipt_sha256": "0299fe453425d5caea94ac27e4da4b7afce05d3ac07d32bffd15fdd39838a7e4",
        "log": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/annulation152-postfix-r1.log",
        "log_sha256": "ff7cf7a01970f9eebe72010870609cd723d06d342ec08f3618f908dfa678c797",
        "full_after_patch_proof": false
      },
      {
        "target": "Running unittests src/lib.rs",
        "name": "mcp::tests::matrice_fr009_couvre_les_quinze_cas",
        "before_full_status": "FAILED",
        "after_targeted_status": "ok",
        "successful_replays": [
          "mcp155-matrix-postfix-r1"
        ],
        "authoritative_replay": "mcp155-matrix-postfix-r1",
        "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/mcp155-matrix-postfix-r1.json",
        "receipt_sha256": "25aeaec3f8983465a0481cf107e7f80f8d3da5ddf83c63b9e7636fa84dec4a30",
        "log": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/mcp155-matrix-postfix-r1.log",
        "log_sha256": "7773f131a339c9488872b988f1cfeb14538292c729a117f957dc9dbc1a2dd610",
        "full_after_patch_proof": false
      },
      {
        "target": "Running unittests src/lib.rs",
        "name": "mcp::tests::native151_ack_facade_schema_ferme_bornes_et_annotations",
        "before_full_status": "FAILED",
        "after_targeted_status": "ok",
        "successful_replays": [
          "mcp151-schema-postfix-r1"
        ],
        "authoritative_replay": "mcp151-schema-postfix-r1",
        "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/mcp151-schema-postfix-r1.json",
        "receipt_sha256": "620a26e0d01da6bb1e803a07d12523255b682b432f55b83105e20ae676e6a0b4",
        "log": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/mcp151-schema-postfix-r1.log",
        "log_sha256": "4d659031417a57bbdc631d05a79f2a76b62c6dee4949cb7cbc2eadc3812b85f6",
        "full_after_patch_proof": false
      },
      {
        "target": "Running unittests src/lib.rs",
        "name": "t3code::tests::spec099_post_accepte_issue_illisible_conserve_la_correlation",
        "before_full_status": "FAILED",
        "after_targeted_status": "ok",
        "successful_replays": [
          "regressions155-replay-r1",
          "http099-final-r2"
        ],
        "authoritative_replay": "http099-final-r2",
        "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/http099-final-r2.json",
        "receipt_sha256": "839c64342edd2cd2683f1786df17b8fc1feb1f6b7ed49f784f016282d2294ea5",
        "log": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/http099-final-r2.log",
        "log_sha256": "921491c64f310cc59536276260e1fb16667bb9d684dd5c2744c5df2965de708d",
        "full_after_patch_proof": false
      }
    ],
    "native155_named_pass_count": 71,
    "native155_breakdown": {
      "daemon_library": 63,
      "native_e2e": 1,
      "transport_library": 7
    },
    "targeted_replays": [
      {
        "label": "native155-postfix-r1",
        "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/native155-postfix-r1.json",
        "receipt_sha256": "20f23d4ccfee42a487e8581c8639975a58b0d4d72e293cbd5c292e999bc61cc8",
        "rc": 0,
        "sources_stable": true,
        "totals": {
          "passed": 71,
          "failed": 0,
          "ignored": 0
        }
      },
      {
        "label": "annulation152-postfix-r1",
        "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/annulation152-postfix-r1.json",
        "receipt_sha256": "0299fe453425d5caea94ac27e4da4b7afce05d3ac07d32bffd15fdd39838a7e4",
        "rc": 0,
        "sources_stable": true,
        "totals": {
          "passed": 2,
          "failed": 0,
          "ignored": 0
        }
      },
      {
        "label": "annulation148-postfix-r1",
        "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/annulation148-postfix-r1.json",
        "receipt_sha256": "39d9a92cf36d31fe4987d0e3010eb7305dcb4e627db36e9d15b939358a014940",
        "rc": 0,
        "sources_stable": true,
        "totals": {
          "passed": 1,
          "failed": 0,
          "ignored": 0
        }
      },
      {
        "label": "mcp155-matrix-postfix-r1",
        "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/mcp155-matrix-postfix-r1.json",
        "receipt_sha256": "25aeaec3f8983465a0481cf107e7f80f8d3da5ddf83c63b9e7636fa84dec4a30",
        "rc": 0,
        "sources_stable": true,
        "totals": {
          "passed": 1,
          "failed": 0,
          "ignored": 0
        }
      },
      {
        "label": "mcp151-schema-postfix-r1",
        "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/mcp151-schema-postfix-r1.json",
        "receipt_sha256": "620a26e0d01da6bb1e803a07d12523255b682b432f55b83105e20ae676e6a0b4",
        "rc": 0,
        "sources_stable": true,
        "totals": {
          "passed": 1,
          "failed": 0,
          "ignored": 0
        }
      },
      {
        "label": "workspace154-schema-postfix-r1",
        "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/workspace154-schema-postfix-r1.json",
        "receipt_sha256": "9681c33c5516b7bcf75fd3d8b4cc87ace18b524cc4d9a18937f3675a17a7ae21",
        "rc": 0,
        "sources_stable": true,
        "totals": {
          "passed": 1,
          "failed": 0,
          "ignored": 0
        }
      },
      {
        "label": "skill089-postfix-r1",
        "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/skill089-postfix-r1.json",
        "receipt_sha256": "a6a5d7c4646a407070f79e30ed2d6b88b74d4324e3013ba624c253d7c7fd1077",
        "rc": 0,
        "sources_stable": true,
        "totals": {
          "passed": 1,
          "failed": 0,
          "ignored": 0
        }
      },
      {
        "label": "regressions155-replay-r1",
        "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/regressions155-replay-r1.json",
        "receipt_sha256": "6581d49315832ab51b2360bc7896451cd87b60902b81b6c656a1bc09b4b136c3",
        "rc": 0,
        "sources_stable": true,
        "totals": {
          "passed": 1,
          "failed": 0,
          "ignored": 0
        }
      },
      {
        "label": "http099-final-r2",
        "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/http099-final-r2.json",
        "receipt_sha256": "839c64342edd2cd2683f1786df17b8fc1feb1f6b7ed49f784f016282d2294ea5",
        "rc": 0,
        "sources_stable": true,
        "totals": {
          "passed": 1,
          "failed": 0,
          "ignored": 0
        }
      }
    ],
    "final_http099": {
      "label": "http099-final-r2",
      "receipt": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/http099-final-r2.json",
      "receipt_sha256": "839c64342edd2cd2683f1786df17b8fc1feb1f6b7ed49f784f016282d2294ea5",
      "passed": 1,
      "failed": 0,
      "rc": 0
    },
    "workspace_global_green": false,
    "T040": "validated_four_cases_with_explicit_user_consent",
    "scope_limit": "Replays ciblés distincts ; aucun test exécuté ni nouveau total workspace calculé ici."
  },
  {
    "kind": "T040_validation_humaine_vérifiée",
    "status": "validated_four_cases_with_explicit_user_consent",
    "receipt": "/Users/moi/.cache/bridget-delivery155.um4xccv4/recette155-postfix-final.json",
    "receipt_sha256": "153b70cbbbaa94315b87f192a8fa67fc217f607eb1cc70c5e9ea5b047b6cc9f3",
    "user_validation": {
      "at_utc": "2026-10-12T03:02:02.400752+00:00",
      "answer": "Oui, accepter et valider si les quatre cas réussissent",
      "scope": "printf RECETTE155_HUMAN isolated daemon; T040 conditional on four planned cases success",
      "executed": true
    },
    "user_validation_before_accept": {
      "at_utc": "2026-10-12T03:02:02.400752+00:00",
      "answer": "Oui, accepter et valider si les quatre cas réussissent",
      "scope": "printf RECETTE155_HUMAN isolated daemon; T040 conditional on four planned cases success",
      "executed": false
    },
    "cases": [
      {
        "name": "question_parent",
        "pass": true,
        "evidence": "question155-runtime.json",
        "reused_prior_runtime": true
      },
      {
        "name": "human_approval",
        "pass": true,
        "reused_prior_runtime": false
      },
      {
        "name": "workspace_outside_limit",
        "pass": true,
        "evidence": "recette155-final-runtime-open.json",
        "reused_prior_runtime": true
      },
      {
        "name": "expiry_600s",
        "pass": true,
        "evidence": "expiry155-r3-runtime.json",
        "reused_prior_runtime": true
      }
    ],
    "scope_limit": "Les trois chemins nominaux déjà exercés sont conservés. Le correctif modifie la présentation Human, les écritures bloquées et le règlement avant écriture ; leurs scénarios concernés passent sur les sources finales. Aucun second FULL ni nouveau délai de600s lancé. Les trois captures précédentes ne sont pas présentées comme runtime du binaire final.",
    "tasks_path": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/tasks.md",
    "tasks_sha256": "8d63662e3d278cdbb5f04d725344c27ab250b324c5e99bd64ef17ae35b0225bb",
    "tasks_total": 42,
    "tasks_checked": 42,
    "t040_checked": true
  }
]
```
