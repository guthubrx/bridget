# Audit de code — session155

**Score global** : A (99.85/100)
**Couverture basse du diff** : 27.46% ; note sur cet échantillon déclaré.
**Livraison bloquée** : False

## Tendance

Première session pour ce scope155. Aucune note154 transposée.

## Résumé exécutif

| Domaine | Note | Numérique | Findings |
|---|---|---:|---:|
| security | A | 100 | 0 |
| llm_security | A | 100 | 0 |
| supply_chain | A | 100 | 0 |
| observability | A | 100 | 0 |
| reliability | A | 100 | 0 |
| tests | A | 100 | 0 |
| performance | A | 99 | 1 |
| complexity | A | 100 | 0 |
| quality | A | 100 | 0 |
| architecture | A | 100 | 0 |
| ai_hygiene | A | 100 | 0 |
| duplication | A | 99 | 1 |
| minimalism | A | 100 | 0 |
| ux | N/A | N/A | 0 |

## Findings

### PERF-082 — LOW — Le tick renouvelle une seconde pour chaque émission séquentielle

crates/bridget-daemon/src/daemon/native_child_requests.rs:907
Le lot de followups est borné à512 lignes, mais chaque appel emit recrée son propre budget d’une seconde. Le tick natif attend la fin de cette boucle avant ses propres transitions. Plusieurs writers lents peuvent donc accumuler leur attente. La condition de saturation n’a pas été mesurée ; les timeouts et la fermeture des sockets limitent chaque émission.

```text
for (id, conn, writer, frame) in outgoing {
    emit(state, &id, &conn, &writer, &frame, false);
}
```

Remédiation : Conserver ce risque comme limite connue. Avant toute optimisation, utiliser les logs de tick existants pour établir une baseline réelle ; si un retard est observé, borner le budget du lot tout en conservant les claims et les recontrôles FR027. Aucun patch de confort demandé dans cette lecture.
Vérification : Lecture finale907–909/929 et native_delegation2402. Aucun délai mesuré ni benchmark.

### DUP-001 — LOW — Renvoi après reconnexion copié dans deux branches

crates/bridget-daemon/src/wrapper.rs:4367
Les lignes4367–4379 et4611–4623 sont identiques : elles relisent les demandes waiting, excluent uncertain et remettent les trames au daemon. Les deux chemins terminent une reconnexion avec le même writer et le même état. Aucun helper équivalent de ce renvoi collectif n’a été trouvé ; send_wrapper_message reste le point unique d’envoi d’une trame.

```text
                if forward_child_requests {
                    let pending: Vec<_> = child_requests
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .waiting
                        .iter()
                        .filter(|entry| !entry.uncertain)
                        .map(|entry| entry.request.clone())
                        .collect();
                    for request in pending {
                        send_wrapper_message(&writer, WrapperToDaemon::ChildRequest { request });
                    }
                }
```

Remédiation : Réunir ce seul bloc dans une fermeture locale au parcours géré, appelée après les deux reconnexions. Garder les deux branches et leurs contrôles. Aucun module, export, framework, table ou outil neuf.
Vérification : Les deux appels doivent conserver le filtre !uncertain et envoyer le même tuple ; rejouer les tests existants native155_wrapper et la reconnexion149 concernés, sans nouveau scénario.


## Minimalisme & Frugalité

Potentiel minimalisme : ~0 lignes suppressibles signalées dans les findings MIN.
Somme des estimations fournies ; aucun décompte global ni conclusion de checklist inventés.

## Vertus LLM & Responsabilité Future

Synthèse fournie par le module12 :
```json
{
  "minimalism_summary": {
    "removable_lines_estimate": 0,
    "basis": "0 ligne directement supprimable à comportement constant étayée dans les seules lectures admissibles ; aucune conclusion globale. DUP-001 reste exclusivement au module07.",
    "cross_domain_note": "DUP-001 appartient uniquement au module07 et propose une réduction locale après refactor ; aucun finding MIN doublonné."
  },
  "virtues": {
    "cognitive_load": "Le champ cwd explicite supprime une ambiguïté de lecture humaine ; null et verified=false évitent une confiance implicite.",
    "volume": "Ajout limité au transfert, contrôle type/borne et citation ; assertions dans les scénarios existants. Les autres volumes restent hors relecture ciblée.",
    "abstractions": "Aucune abstraction nouvelle ; validations séparées car frontière fournisseur puis DTO/droits. Pas de factorisation générale justifiée par deux expressions Debug.",
    "human_responsibility": "Contrats précisent1024octets/champ,4096faits, absence/null, contrôle visible et aucun confinement. L’intention et les limites sont expliquées.",
    "ununderstood_zones": "Aucune glue opaque étayée dans le delta cwd. Les anciennes analyses ont leur provenance historique ; les autres hunks ne sont pas crédités fraîchement."
  }
}
```

## Limites et preuves historiques

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

## Points positifs

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_delegation.rs:1385 — q² et borne8Kio annoncés exactement.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests.rs:529 — Debugcwd borné1024octets, facts4096.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/native_permissions.rs:73 — cwd chaîne<=1024octets ou null ; autre forme refusée sans I/O ni changement des droits.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests.rs:529 — cwd cité entier avec Debug ou null, cwd_verified=false ; budgetglobal4096 maintenu.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-transport/src/codex_app_server.rs:2605 — absence conservée comme null, aucun fallback vers dossier session/parent.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_delegation_chain155_tests.rs:487 — fixture existante vérifie fin du dossier et contrôles visibles, sans confondre citation et confinement.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests.rs:611 — Le canal Human reste horsstate aprèscwd.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/human_inbox.rs:549 — Budgetcommun3s et stderr retenu1024octets maintenus.
- native_child_requests.rs:521–546 : commande/cwd entiers cités avec Debug, cwd_verified=false, effets complets et refusfacts>4096.
- native_permissions.rs:50–80 et codex_app_server.rs:2463–2475,2602–2618 : cwd string/null exact, forme invalide et1025octets refusés ; aucun gain de droit149.
- native_child_requests.rs:920–1002 : contrôle avantwrite et reprise M2 restent conservateurs aprèsdelta.
- codex_app_server.rs:429–452,1082–1093,3238–3310 : O_NONBLOCK, budgetSTDIO3s partagé et incertitude conservés aprèsdelta.
- chain155:398–543, human155:1–117, codex:7705–7737 : oracles existants inspectés pourcwd complet/null/contrôles/1025octets ; leurs noms PASS recoupés dansROOTcwd.
- native155-cwd-r1.json/log :71PASS=63daemon+1E2E+7transport,214sources stables et SHA égaux à la capture finale ; aucun full final déduit.

## Suppressions baseline
```json
[]
```


## Corrections et traces du cycle1
```json
{
  "resolved_findings": [
    {
      "original_id": "PERF-041",
      "original_fingerprint": "6f75818e3962",
      "original": {
        "id": "PERF-041",
        "category": "performance",
        "subcategory": "algorithmic_complexity",
        "severity": "MEDIUM",
        "file": "crates/bridget-daemon/src/daemon/native_delegation.rs",
        "line": 1385,
        "title": "Respond annonce un coût linéaire mais compare les IDs en O(q²)",
        "description": "Le commentaire du chemin Respond annonce O(log tâches + log demandes + B). La validation parcourt les clés de réponse et recherche chacune dans la liste des IDs attendus. Cette partie coûte O(q²+B), sous le verrou métier ; q reste borné par les limites de projection et de réponse. Le problème retenu est l’annotation inexacte, pas un ralentissement runtime démontré.",
        "evidence": "|| input\n    .keys()\n    .any(|key| !ids.iter().any(|id| id.as_str() == Some(key.as_str())))\n|| ids.iter().any(|id| {\n    id.as_str().is_none_or(|id| {",
        "exploitability": "N/A",
        "impact": "L’annotation sous-estime le coût de vérification ; les limites4096/8192 octets empêchent une croissance non bornée sur le chemin normalisé.",
        "path_type": "hot",
        "remediation": "Corriger uniquement le commentaire existant en O(log tâches + log demandes + q² + B), en citant les bornes déjà présentes. Aucun nouvel index, helper ou scénario.",
        "verification": "Relire le commentaire contre les boucles existantes et les limites à codex_app_server.rs:2605 et native_delegation.rs:1486 ; vérification documentaire sans nouveau test.",
        "effort": "XS",
        "regression_risk": "LOW",
        "complexity_current": "O(log t + log n + q² + B)",
        "complexity_target": "Même code et mêmes bornes ; annotation exacte",
        "references": [
          "Constitution Article XVIII",
          "audit-code-v14 module04",
          "crates/bridget-transport/src/codex_app_server.rs:2605",
          "crates/bridget-daemon/src/daemon/native_delegation.rs:1486"
        ],
        "fingerprint": "6f75818e3962"
      },
      "status": "resolved",
      "proof": {
        "file": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_delegation.rs",
        "line": 1385,
        "sha256": "51905c9443f3a47ede4799b176827039192ab20c583f6cf2b2fb9d1e40ccf99b",
        "evidence": "            // O(log tâches + log demandes + q² + B), réponse<=8Kio ; même propriétaire\n            // direct que recover_owner, sans écriture pour un refus ou un rejeu.\n            if [task_id, request_id].iter().any(|id| {"
      }
    },
    {
      "original_id": "PERF-081",
      "original_fingerprint": "41c2c45e2996",
      "original": {
        "id": "PERF-081",
        "category": "performance",
        "subcategory": "blocking_io_under_business_lock",
        "severity": "HIGH",
        "counter_audit": "CONFIRMED",
        "file": "crates/bridget-daemon/src/daemon/native_child_requests.rs",
        "line": 527,
        "title": "Le canal humain peut garder le verrou commun sans limite",
        "description": "receive garde le MutexGuard DaemonState pendant publish_human. Le nouveau dépôt Human appelle daemon_deposit_human_inbox puis deposit_and_notify, qui attend synchroniquement le processus du canal via write_all et wait_with_output. La transaction du dépôt est déjà committée ; le problème porte sur le verrou métier commun. Précondition : canal valide configuré et processus externe qui ne termine pas. La lecture indépendante de root et audit_performance151 confirme le flux ; aucun blocage runtime n’a été reproduit.",
        "evidence": "if verdict == ChildRequestVerdict::Human {\n    publish_human(&st, &task, &row)?;\n} else {\n    publish_parent(&mut st, &task, &row, &mut controls)?;\n    st.delegation_store.wait_epoch().bump();\n}",
        "exploitability": "LIKELY",
        "impact": "Sous cette précondition, une demande Human peut empêcher les autres lecteurs, les réponses, les échéances et le contrôle du daemon d’acquérir DaemonState. Aucun temps de blocage mesuré.",
        "path_type": "hot",
        "remediation": "Séparer le dépôt/la liaison durable de la notification. Réutiliser deposit_and_notify avec canal None sous state, préparer le résumé, puis appeler push_to_channel après libération de state et enregistrer la tentative après reprise brève du verrou. Conserver les clés, le CAS et les recontrôles ; aucun nouvel outil ou framework. Root fournit le raccord final dans les fonctions existantes.",
        "remediation_code": "// Dans la section existante de dépôt, sous state :\nlet deposited = crate::human_inbox::deposit_and_notify(\n    st.store.connection(), request, None,\n).map_err(|error| error.to_string())?\n .map_err(|refusal| format!(\"human_inbox_refused:{refusal:?}\"))?;\nst.delegation_store.link_child_inbox(&row.request_id, &deposited.item_id)?;\n// Si deposited.created : préparer (config, summary, item_id, now)\n// dans la liste locale de notifications, sans lancer le canal ici.\n// Après la fin du bloc qui détient state :\nfor (config, summary, item_id, now) in human_notifications {\n    let result = crate::human_inbox::push_to_channel(&config, &summary);\n    let st = state.lock().unwrap_or_else(|error| error.into_inner());\n    let _ = crate::human_inbox::record_notify_attempt(\n        st.store.connection(), &item_id,\n        &crate::human_inbox::NotifyAttempt {\n            at: now, ok: result.is_ok(), error: result.err(),\n        },\n    );\n}",
        "verification": "Root : étendre le scénario Human existant avec le canal privé déjà utilisé ; pendant l’attente du canal, une lecture/échéance indépendante doit acquérir state. Vérifier le même item lié et la même trace de tentative après retour ; aucune exécution faite par cet auditeur.",
        "effort": "S",
        "regression_risk": "MEDIUM",
        "references": [
          "audit-code-v14 module08 checklist DB/I-O",
          "crates/bridget-daemon/src/daemon/native_child_requests.rs:344",
          "crates/bridget-daemon/src/daemon/native_child_requests.rs:278",
          "crates/bridget-daemon/src/daemon.rs:9943",
          "crates/bridget-daemon/src/human_inbox.rs:157",
          "crates/bridget-daemon/src/human_inbox.rs:545",
          "crates/bridget-daemon/src/human_inbox.rs:557"
        ],
        "counter_audit_evidence": {
          "source": "Message du principal du2026-10-12 : contre-lecture root et audit_performance151 confirmée ; rapport cache counter-audit155-channel.md/json annoncé en clôture.",
          "scope": "Flux/verrou confirmés par lecture ; canal configuré et non-terminaison conditionnels, aucun runtime reproduit."
        },
        "fingerprint": "41c2c45e2996"
      },
      "status": "resolved",
      "proof": {
        "file": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests.rs",
        "line": 611,
        "sha256": "3be800076fe7523adc083dce913f4d38988dc7da540fb6d4fa6bc3c2adb2af51",
        "evidence": "    })();\n    if let Some((config, summary)) = human_notification {\n        // Le programme externe ne détient jamais le verrou métier. Seule la\n        // consignation locale de sa tentative reprend brièvement state.\n        let pushed = crate::human_inbox::push_to_channel(&config, &summary);"
      }
    },
    {
      "original_id": "SEC-001",
      "original_fingerprint": "573cf471637c",
      "original": {
        "id": "SEC-001",
        "category": "security",
        "subcategory": "incomplete_human_authorization_facts",
        "severity": "HIGH",
        "counter_audit": "CONFIRMED",
        "file": "crates/bridget-daemon/src/daemon/native_child_requests.rs",
        "line": 457,
        "title": "Human reçoit des faits tronqués sans indication de perte",
        "description": "Une commande source de513–1024 octets passe la normalisation, puis sa projection Human conserve512 octets. Les contrôles et chemins peuvent aussi être transformés ou coupés ; effects.command/file_write sont omis. L’accord répond au callback fournisseur original, donc porte sur une demande plus complète que les faits présentés. La chaîne est confirmée statiquement ; aucun rendu ou exploit runtime observé.",
        "exploitability": "LIKELY",
        "exploitation_scenario": "Demande commandExecution valide sous dangerFullAccess, avec suffixe significatif après512 octets et offre accept/decline. Human voit une commande recoupée sans marqueur ; accept s’applique au callback original. Aucune commande exécutée par l’audit.",
        "impact": "L’humain peut autoriser un effet absent de sa présentation. Le plafond de droits149 reste inchangé.",
        "path_type": "hot",
        "remediation": "Refuser avec details_incomplete avant Human si la projection change ou coupe une commande/chemin. Copier les trois effets normalisés dans facts sous la garde4096 existante. Réutiliser les gardes et fonctions existantes.",
        "remediation_code": "let display_changed = request.details[\"command\"].as_str().is_some_and(|raw| quoted_text(raw,512) != raw)\n    || request.details[\"paths_cited\"].as_array().is_some_and(|raw| raw.len() != paths.len()\n        || raw.iter().zip(&paths).any(|(value,shown)| value.as_str() != Some(shown.as_str())));\nif verdict == ChildRequestVerdict::Human && display_changed {\n    verdict = ChildRequestVerdict::Forbidden;\n    code = Some(\"details_incomplete\");\n}\nlet mut presented: Value = serde_json::from_str(&facts).map_err(|error| error.to_string())?;\npresented[\"effects\"] = request.details[\"effects\"].clone();\nfacts = presented.to_string();\n// Insérer avant la garde facts.len()>4096 existante et avant toute publication Human.",
        "verification": "Contre-relire absence de publication Human lors de transformation/perte et copie des trois effets. Réemployer les oracles155 existants de faits et texte hostile ; aucun nouveau test exécuté ici.",
        "effort": "S",
        "regression_risk": "MEDIUM",
        "references": [
          "counter-audit155-display-stdio.json:H1",
          "codex_app_server.rs:2553–2562,1012–1027",
          "native_child_requests.rs:13–26,443–467,276–290"
        ],
        "runtime_reproduced": false,
        "status": "open",
        "evidence": "        let mut facts=json!({\"method\":quoted_text(request.details[\"method\"].as_str().unwrap_or(\"unsupported\"),128),\n            \"duration\":if request.details[\"duration\"]==\"once\"{\"once\"}else{\"unsupported\"},\n            \"command\":quoted_text(request.details[\"command\"].as_str().unwrap_or(\"\"),512),\n            \"paths_cited\":paths,\"paths_verified\":false,\"network\":request.details[\"effects\"][\"network\"].as_bool().unwrap_or(false),\n            \"child_posture\":if snapshot.child_policy[\"sandbox_policy\"][\"type\"]==\"readOnly\"{\"read_only\"}else{\"development\"},\n            \"parent_sandbox\":snapshot.parent.provider_policy[\"sandbox_policy\"][\"type\"],\"verdict_code\":code,\n            \"question_ids\":if verdict==ChildRequestVerdict::Parent{request.question_ids.clone()}else{Vec::new()}}).to_string();",
        "counter_audit_evidence": {
          "path": "/Users/moi/.cache/bridget-delivery155.um4xccv4/counter-audit155-display-stdio.json",
          "sha256": "85f7de19475ff0b8bedc4da6f13cb83bf1d09aa0107ca22f07cde02f4c97d91b",
          "method": "independent static source review",
          "runtime_reproduced": false
        },
        "path_type_basis": "sensitive callback path; runtime frequency not measured",
        "fingerprint": "573cf471637c"
      },
      "status": "resolved",
      "proof": {
        "file": "crates/bridget-daemon/src/daemon/native_child_requests.rs",
        "line": 521,
        "sha256": "3be800076fe7523adc083dce913f4d38988dc7da540fb6d4fa6bc3c2adb2af51",
        "evidence": "        let raw_command = request.details[\"command\"].as_str().unwrap_or(\"\");\n        // Debug rend les contrôles et caractères de direction visibles, sans\n        // modifier ni couper la commande qui sera exécutée par le fournisseur.\n        let command = if raw_command.is_empty() {\n            String::new()\n        } else {\n            format!(\"{raw_command:?}\")\n        };\n        let cwd = request.details[\"cwd\"]\n            .as_str()\n            .map(|cwd| format!(\"{cwd:?}\"));\n        let mut facts=json!({\"method\":quoted_text(request.details[\"method\"].as_str().unwrap_or(\"unsupported\"),128),\n            \"duration\":if request.details[\"duration\"]==\"once\"{\"once\"}else{\"unsupported\"},\n            \"command\":command,\n            \"cwd\":cwd,\"cwd_verified\":false,\n            \"paths_cited\":paths,\"paths_verified\":false,\"paths_cited_truncated\":paths_cited_truncated,\"paths_cited_changed\":paths_cited_changed,\n            \"effects\":{\"command\":request.details[\"effects\"][\"command\"].as_bool(),\"file_write\":request.details[\"effects\"][\"file_write\"].as_bool(),\"network\":request.details[\"effects\"][\"network\"].as_bool()},\n            \"network\":request.details[\"effects\"][\"network\"].as_bool(),\n            \"child_posture\":if snapshot.child_policy[\"sandbox_policy\"][\"type\"]==\"readOnly\"{\"read_only\"}else{\"development\"},\n            \"parent_sandbox\":snapshot.parent.provider_policy[\"sandbox_policy\"][\"type\"],\"verdict_code\":code,\n            \"question_ids\":if verdict==ChildRequestVerdict::Parent{request.question_ids.clone()}else{Vec::new()}}).to_string();\n        if raw_command.len() > 1024 || facts.len() > 4096 {\n            verdict = ChildRequestVerdict::Forbidden;\n            code = Some(\"details_incomplete\");\n            facts=json!({\"method\":\"unsupported\",\"verdict_code\":\"details_incomplete\",\"question_ids\":[]}).to_string();\n        }",
        "reviewed_at_utc": "2026-10-12T03:37:28.083936+00:00"
      }
    },
    {
      "original_id": "REL-001",
      "original_fingerprint": "a33380318509",
      "original": {
        "id": "REL-001",
        "category": "reliability",
        "subcategory": "unbounded_private_stdio_callback",
        "severity": "HIGH",
        "counter_audit": "CONFIRMED",
        "file": "crates/bridget-transport/src/codex_app_server.rs",
        "line": 1065,
        "title": "Le callback155 peut rester bloqué sur le STDIO privé",
        "description": "answer_provider_request tient la table transport pendant write_value ; le wrapper tient aussi sa table dans son lecteur de commandes. ChildStdin est bloquant et writeln/flush n’ont pas d’échéance. shutdown attend les mêmes table/writer. write_value est une dette héritée dont le callback155 étend l’impact ; le scénario reste conditionnel et non reproduit.",
        "exploitability": "LIKELY",
        "exploitation_scenario": "Fournisseur vivant qui ne lit plus stdin ; pipe plein pendant une réponse. L’écriture attend sans borne et bloque le lecteur géré, ses contrôles et la progression de l’arrêt. Aucun banc de saturation exécuté.",
        "impact": "Une session peut rester sans ACK ni clôture sûre. Aucune élévation de droits ou panne globale du daemon n’est affirmée.",
        "path_type": "hot",
        "remediation": "Borner le verrou writer et les écritures réelles du pipe avec une échéance absolue et mode non bloquant selon le patron existant. Préserver la sérialisation, l’incertitude de ligne partielle, alive=false et l’absence de replay. request_with_timeout seul ne borne pas le write préalable.",
        "remediation_code": "use std::os::fd::AsRawFd;\nlet fd = stdin.as_raw_fd();\nlet flags = unsafe { libc::fcntl(fd,libc::F_GETFL) };\nif flags < 0 || unsafe { libc::fcntl(fd,libc::F_SETFL,flags|libc::O_NONBLOCK) } < 0 {\n    return Err(TransportError::Io(std::io::Error::last_os_error().to_string()));\n}\n// Avant boxing du STDIO. Réemploi du write_value existant, avec le même deadline\n// avant writer.try_lock() et chaque write, offset conservé, WouldBlock/Interrupted\n// traités jusqu'à expiration ; aucune réinitialisation du délai par fragment.\n// Sur tout échec partiel conserver le chemin existant :\npending[index].uncertain = true;\nself.alive.store(false, Ordering::SeqCst);\nself.observations.1.notify_all();\nreturn Err(TransportError::Io(format!(\"child_request_write_uncertain: {error}\")));\n// Adapter l'acquisition du writer dans write_value à try_lock jusqu'au deadline.\n// Garder la sérialisation answer/cancel. Le writer socket garde sa propre borne.\n// Proposition d'intégration non compilée, pas un patch exécuté.",
        "verification": "Contre-relire O_NONBLOCK avant boxing, acquisition/écriture bornées par la même horloge et échec partiel uncertain sans ACK. Réemployer les oracles callback/answer-cancel155 existants ; aucun nouveau scénario de confort exécuté.",
        "effort": "M",
        "regression_risk": "MEDIUM",
        "references": [
          "counter-audit155-display-stdio.json:H2",
          "codex_app_server.rs:427–437,967–1074,3177–3185,781–789,1272–1293",
          "wrapper.rs:408–425,4521–4530,5560–5617"
        ],
        "runtime_reproduced": false,
        "status": "open",
        "evidence": "        // Le verrou sérialise answer/fin/resolved. Une écriture partielle ne\n        // prouve aucun refus : garder l'incertitude et fermer l'admission.\n        if let Err(error) = write_value(&self.writer, reply) {\n            pending[index].uncertain = true;\n            self.alive.store(false, Ordering::SeqCst);\n            self.observations.1.notify_all();\n            return Err(TransportError::Io(format!(\n                \"child_request_write_uncertain: {error}\"",
        "counter_audit_evidence": {
          "path": "/Users/moi/.cache/bridget-delivery155.um4xccv4/counter-audit155-display-stdio.json",
          "sha256": "85f7de19475ff0b8bedc4da6f13cb83bf1d09aa0107ca22f07cde02f4c97d91b",
          "method": "independent static source review",
          "runtime_reproduced": false
        },
        "path_type_basis": "sensitive callback path; runtime frequency not measured",
        "fingerprint": "a33380318509"
      },
      "status": "resolved",
      "proof": {
        "file": "crates/bridget-transport/src/codex_app_server.rs",
        "line": 3238,
        "sha256": "ac13e0cfd0becdec45a3e24ae7973ddbb94c18e74b00ce40ddf0a93fca76c5e8",
        "evidence": "fn write_value(writer: &Writer, value: Value) -> Result<(), TransportError> {\n    // Un budget TOTAL pour le verrou, les écritures STDIO partielles et flush.\n    // Le SocketWriter interactif conserve son propre flush borné à3s ; il\n    // peut ajouter ces3s après contention, sans prétendre au budget STDIO.\n    let deadline = Instant::now() + Duration::from_secs(3);\n    let mut bytes =\n        serde_json::to_vec(&value).map_err(|error| TransportError::Io(error.to_string()))?;\n    bytes.push(b'\\n'); // mêmes octets JSON compacts + LF que writeln!(\"{value}\").\n    let mut guard = loop {\n        if Instant::now() >= deadline {\n            return Err(TransportError::Io(\"budget writer Codex dépassé\".into()));\n        }\n        match writer.try_lock() {\n            Ok(guard) => break guard,\n            Err(std::sync::TryLockError::Poisoned(error)) => {\n                return Err(TransportError::Io(error.to_string()));\n            }\n            Err(std::sync::TryLockError::WouldBlock) => {\n                let millis = deadline\n                    .saturating_duration_since(Instant::now())\n                    .as_millis()\n                    .clamp(1, 25) as i32;\n                if unsafe { libc::poll(std::ptr::null_mut(), 0, millis) } < 0 {\n                    let error = std::io::Error::last_os_error();\n                    if error.kind() != std::io::ErrorKind::Interrupted {\n                        return Err(TransportError::Io(error.to_string()));\n                    }\n                }\n            }\n        }\n    };\n    let result = (|| {\n        let output = guard.as_mut().ok_or(TransportError::AgentDead)?;\n        let mut offset = 0;\n        loop {\n            if Instant::now() >= deadline {\n                return Err(TransportError::Io(\"budget writer Codex dépassé\".into()));\n            }\n            let progress = if offset < bytes.len() {\n                output.write(&bytes[offset..]).map(Some)\n            } else {\n                output.flush().map(|()| None)\n            };\n            match progress {\n                Ok(None) => return Ok(()),\n                Ok(Some(0)) => {\n                    return Err(TransportError::Io(\"écriture Codex sans progrès\".into()));\n                }\n                Ok(Some(count)) => offset += count,\n                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,\n                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {\n                    // Box<dyn Write> ne donne aucun fd. poll sans descripteur\n                    // attend au plus25ms sous la MÊME échéance, sans busy-loop.\n                    let millis = deadline\n                        .saturating_duration_since(Instant::now())\n                        .as_millis()\n                        .clamp(1, 25) as i32;\n                    if unsafe { libc::poll(std::ptr::null_mut(), 0, millis) } < 0 {\n                        let error = std::io::Error::last_os_error();\n                        if error.kind() != std::io::ErrorKind::Interrupted {\n                            return Err(TransportError::Io(error.to_string()));\n                        }\n                    }\n                }\n                Err(error) => return Err(TransportError::Io(error.to_string())),\n            }\n        }\n    })();\n    // Une trame partielle n'est pas une réponse négative attestée. Fermer le\n    // canal empêche qu'une écriture suivante complète ses octets incertains.\n    if result.is_err() {\n        let _ = guard.take();\n    }",
        "reviewed_at_utc": "2026-10-12T03:37:28.083936+00:00"
      }
    }
  ],
  "persistent_history": [
    {
      "original_id": "DUP-001",
      "original_fingerprint": "c0dcaaf9b24b",
      "original": {
        "id": "DUP-001",
        "category": "quality",
        "subcategory": "duplication",
        "severity": "LOW",
        "file": "crates/bridget-daemon/src/wrapper.rs",
        "line": 4367,
        "title": "Renvoi après reconnexion copié dans deux branches",
        "description": "Les lignes4367–4379 et4611–4623 sont identiques : elles relisent les demandes waiting, excluent uncertain et remettent les trames au daemon. Les deux chemins terminent une reconnexion avec le même writer et le même état. Aucun helper équivalent de ce renvoi collectif n’a été trouvé ; send_wrapper_message reste le point unique d’envoi d’une trame.",
        "evidence": "                if forward_child_requests {\n                    let pending: Vec<_> = child_requests\n                        .lock()\n                        .unwrap_or_else(|e| e.into_inner())\n                        .waiting\n                        .iter()\n                        .filter(|entry| !entry.uncertain)\n                        .map(|entry| entry.request.clone())\n                        .collect();\n                    for request in pending {\n                        send_wrapper_message(&writer, WrapperToDaemon::ChildRequest { request });\n                    }\n                }",
        "exploitability": "N/A",
        "impact": "Une évolution du filtre ou de la remise doit être reportée dans deux copies ; risque de divergence de maintenance. Aucun défaut runtime actuel ni taux global de duplication n’est déduit.",
        "path_type": "cold",
        "remediation": "Réunir ce seul bloc dans une fermeture locale au parcours géré, appelée après les deux reconnexions. Garder les deux branches et leurs contrôles. Aucun module, export, framework, table ou outil neuf.",
        "verification": "Les deux appels doivent conserver le filtre !uncertain et envoyer le même tuple ; rejouer les tests existants native155_wrapper et la reconnexion149 concernés, sans nouveau scénario.",
        "effort": "XS",
        "regression_risk": "LOW",
        "duplication_type": "STRICT",
        "duplication_lines": 13,
        "occurrences_count": 2,
        "all_locations": [
          "crates/bridget-daemon/src/wrapper.rs:4367",
          "crates/bridget-daemon/src/wrapper.rs:4611"
        ],
        "fingerprint": "c0dcaaf9b24b"
      },
      "status": "persisting",
      "proof": {
        "file": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/wrapper.rs",
        "line": 4367,
        "sha256": "659a1e09f6ac4c351d6d906e1065f4614680e016cf36cd0c066b039df074278d",
        "evidence": "                if forward_child_requests {\n                    let pending: Vec<_> = child_requests\n                        .lock()\n                        .unwrap_or_else(|e| e.into_inner())\n                        .waiting\n                        .iter()\n                        .filter(|entry| !entry.uncertain)\n                        .map(|entry| entry.request.clone())\n                        .collect();\n                    for request in pending {\n                        send_wrapper_message(&writer, WrapperToDaemon::ChildRequest { request });\n                    }\n                }"
      },
      "current_id": "DUP-001",
      "current_fingerprint": "c0dcaaf9b24b"
    },
    {
      "original_id": "PERF-082",
      "original_fingerprint": "19cf86e44509",
      "original": {
        "id": "PERF-082",
        "category": "performance",
        "subcategory": "sequential_ipc_budget",
        "severity": "LOW",
        "file": "crates/bridget-daemon/src/daemon/native_child_requests.rs",
        "line": 811,
        "title": "Le tick renouvelle une seconde pour chaque émission séquentielle",
        "description": "Le lot de followups est borné à512 lignes, mais chaque appel emit recrée son propre budget d’une seconde. Le tick natif attend la fin de cette boucle avant ses propres transitions. Plusieurs writers lents peuvent donc accumuler leur attente. La condition de saturation n’a pas été mesurée ; les timeouts et la fermeture des sockets limitent chaque émission.",
        "evidence": "for (id, conn, writer, frame) in outgoing {\n    emit(state, &id, &conn, &writer, &frame, false);\n}",
        "exploitability": "THEORETICAL",
        "impact": "Retard potentiel du tick natif et des échéances. Le budget nominal des attentes IPC peut cumuler jusqu’à k secondes, k<=512 ; les relectures sources/SQL ont un coût distinct non mesuré. Ce n’est pas une durée runtime observée.",
        "path_type": "hot",
        "remediation": "Conserver ce risque comme limite connue. Avant toute optimisation, utiliser les logs de tick existants pour établir une baseline réelle ; si un retard est observé, borner le budget du lot tout en conservant les claims et les recontrôles FR027. Aucun patch de confort demandé dans cette lecture.",
        "verification": "Vérification structurelle : budget renouvelé à :833 et appel préalable au tick natif à native_delegation.rs:2402. Une éventuelle mesure future réutilise les logs existants ; aucun benchmark ou scénario exécuté ici.",
        "effort": "S",
        "regression_risk": "MEDIUM",
        "references": [
          "crates/bridget-daemon/src/daemon/native_child_requests.rs:833",
          "crates/bridget-daemon/src/daemon/native_delegation.rs:2402",
          "audit-code-v14 module08 profiling-first"
        ],
        "fingerprint": "19cf86e44509"
      },
      "status": "persisting",
      "proof": {
        "file": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests.rs",
        "line": 907,
        "sha256": "3be800076fe7523adc083dce913f4d38988dc7da540fb6d4fa6bc3c2adb2af51",
        "evidence": "    for (id, conn, writer, frame) in outgoing {\n        emit(state, &id, &conn, &writer, &frame, false);\n    }"
      },
      "current_id": "PERF-082",
      "current_fingerprint": "19cf86e44509"
    }
  ]
}
```