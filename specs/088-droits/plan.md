# Implementation Plan: Droits lisibles et vérifiables

**Branch**: `088-droits` | **Date**: 2026-09-03 | **Spec**: `specs/088-droits/spec.md`
**Input**: spec 088, research R1-R7.

## Summary

Rendre lisibles six couches d'autorisation existantes sans en créer une septième : une forme unique de refus (couche, chose empêchée, geste) rendue par une seule fonction d'interface ; une page « Droits » du centre de contrôle qui pilote les mécanismes existants par trois profils et un mode expert ; une action « Tester » qui envoie à un agent un geste à jeton et lit le résultat dans le journal des actes. Les seuls objets nouveaux : deux colonnes de `control_state` (posture d'agent, réassignation automatique) écrites par `ControlStateSet`, un fichier de tentatives de test côté relais, un acte `refusal` conditionné à la fin en échec de la commande, la fin de commande journalisée, deux champs additifs optionnels sur `ControlStateFrame`, et un petit module de motifs. Révisé le 2026-09-03 après la contre-revue de Jim (`adversarial-review-jim.md`).

## Technical Context

**Language/Version**: Rust 2024 (daemon, transport, Maicie), JavaScript sans framework (interface, tests `node --test`)
**Primary Dependencies**: aucune nouvelle
**Storage**: fichiers JSON du relais (`control_settings.rs`), `control_state` SQLite (087), `localStorage` navigateur
**Testing**: `cargo test -p bridget-transport`, `cargo test -p bridget-daemon --lib --features test-support`, `cargo test -p maicie`, `node --test app.js` (136 attendus + nouveaux)
**Target Platform**: serveur Linux (daemon, relais, agents), navigateur et Bridget Desktop
**Constraints**: aucune ligne locale modifiable depuis le serveur ; principal humain seul écrivain des droits serveur ; aucun geste de test qui écrit ou dépense ; compilation sur le serveur (`CARGO_TARGET_DIR=/home/moi/bridget-referent/bridget/target`)
**Scale/Scope**: 11 lignes fermées, 3 profils, 4 gestes de test, 2 motifs de refus

## Constitution Check

- Art. III (cycle SpecKit) : respecté, spec puis plan puis audit puis tasks.
- Art. VII (ADR) : ADR-028 « Refus explicites et page Droits » créée avec ce plan.
- Art. XVIII (complexité) : toutes les boucles portent sur des inventaires fermés (11 lignes, 2 motifs, N agents connectés) ; la lecture du journal des actes pendant un test est bornée à 30 s et à un agent.
- Art. XIX (minimalisme) : pas de registre de droits par agent, pas de posture intermédiaire, pas de client réseau ; les lignes réutilisent leurs mécanismes ; `renderRefusal` remplace trois rendus ad hoc au lieu de s'y ajouter.
- Art. XX : chaque tâche a un résultat observable et un mutant pour les gardes.

## Reutilisation de l'existant

| Besoin | Existant réutilisé | Preuve |
|---|---|---|
| Réglages du référent avec génération et journal | `control_state`, `control_events`, `ControlMutation` | `crates/bridget-daemon/src/referent_control.rs:222` |
| Garde du principal humain sur une route | `control_request`, périmètre `bridget-ui-control` | `crates/bridget-daemon/src/ui.rs` (routes `/v1/control/state`) |
| Plafond d'objectifs | `ControlStateSet { auto_objectives_cap }` | `crates/bridget-daemon/src/referent_control.rs:222` |
| Modifier Bridget | `dogfooding_bridget_descriptor`, routes `/v1/control/dogfooding/*` | `control_settings.rs:183`, `ui.rs:2529` |
| Posture d'agent | `project_discovery_definition`, préfixe `PROJECT_DISCOVERY_AGENT_PREFIX` | `crates/bridget-daemon/src/registry.rs:1110-1160` |
| Préférences locales de contenu | `readContentSecurityPreferences`, `writeContentSecurityPreferences` | `app.js:7126-7160` |
| Navigation du centre de contrôle | `CONTROL_CENTER_NAVIGATION`, `controlSection`, `controlSetting` | `app.js:7099, 8366-8410` |
| Sortie brute des commandes Codex | bras `item/commandExecution/outputDelta` | `crates/bridget-transport/src/codex_app_server.rs:2198` |
| Journal des actes | `act_kind.rs`, `record_active_act` | `crates/bridget-transport/src/act_kind.rs`, `codex_app_server.rs:2781` |
| Envoi d'un message humain à un agent | route `/v1/send` | `ui.rs` |
| Réassignation différée par motif | `reduire_reassignation`, motif `pause` (087) | `plugins/maicie/src/domain.rs:1351` |

## Project Structure

```text
specs/088-droits/{spec,plan,research,data-model,quickstart,reuse-audit,tasks}.md, contracts/rights-v1.md
docs/decisions/028-refus-explicites-et-page-droits.md
crates/bridget-transport/src/refusals.rs          # motifs fermés + Refusal (nouveau, ~80 lignes)
crates/bridget-transport/src/protocol.rs          # ControlStateFrame/Set : agent_posture, auto_reassignment (Option)
crates/bridget-transport/src/act_kind.rs          # acte refusal
crates/bridget-transport/src/codex_app_server.rs  # fin de commande journalisée ; refusal si ligne ET échec du même item
crates/bridget-daemon/src/control_settings.rs     # matrice des profils, profile_for, tentatives de test
crates/bridget-daemon/src/referent_control.rs     # colonnes agent_posture, auto_reassignment ; rights_set
crates/bridget-daemon/src/daemon.rs               # bras ControlStateSet/Read étendus ; posture lue au lancement géré
crates/bridget-daemon/src/ui.rs                   # routes /v1/control/rights{,/apply,/test}
crates/bridget-daemon/src/fleet.rs                # définition découverte quand la posture l'exige
crates/bridget-daemon/assets/ui/{app.js,index.html,theme.css}  # renderRefusal, page Droits, Tester
plugins/maicie/src/{bridget_client.rs,control.rs,store.rs,routines.rs}  # ControlStateWire, ControlSnapshot, effet Reassignment, motif droits
tests/features/088-droits.feature
```

## Phases

### Phase A - Refus explicites (US1)

1. `refusals.rs` : `RefusalLayer` fermé, `Refusal`, `recognize_sandbox_refusal(line)`. Tests : motifs, faux positifs, ligne vide.
2. Wrapper Codex : journaliser la fin d'un `commandExecution` (`item/completed` : `state`, `exit_code`, `output_tail`, `item_id`) ; mémoriser par item les lignes reconnues ; à la fin en ÉCHEC d'un item qui a une ligne reconnue, écrire un acte `refusal` (`evidence: output_and_exit`). Témoins : ligne + échec ⇒ un acte ; ligne + succès ⇒ aucun ; échec sans ligne ⇒ aucun ; deux lignes même item ⇒ un. Préalable : mesurer sur le serveur la forme réelle de `item/completed` pour `commandExecution` (trace d'un tour réel) ; si elle est absente, aucun acte et le dire.
3. `app.js` : `renderRefusal` unique ; `local_toggle` honoré seulement avec `allowLocalToggle` posé par `renderContentReferences` ; `normalizeRefusal` retire tout geste local d'une source non locale ; carte de lien, carte d'acte `refusal` (« Signalement de sandbox (non attesté) », geste Tester), refus du plan de contrôle. Tests Node dont l'essai adverse : un refus distant portant `local_toggle` ne modifie pas `bridget.content-security.v1`.

### Phase B - Page Droits (US2)

4. `protocol.rs` : `AgentPosture` fermé ; `ControlStateFrame { agent_posture: Option, auto_reassignment: Option }` ; `ControlStateSet { agent_posture: Option, auto_reassignment: Option }`. `--no-run` workspace pour tous les initialiseurs.
5. `referent_control.rs` : migration des colonnes (neuve : `discovery`/0 ; existante : `complete`/1), `ControlMutation` étendue, `ControlEventKind::RightsSet`, `read` renvoie `Some(...)`. Tests : migration des deux cas, `set` posture seule, rejeu idempotent, `NothingToChange`.
6. `daemon.rs` : bras `ControlStateSet` transmet les champs ; lancement géré : si posture `discovery` et définition découverte disponible ⇒ définition `PROJECT_DISCOVERY_AGENT_PREFIX + type`, sinon refus explicite au lancement (pas de repli silencieux). Test d'effet + mutant.
7. `control_settings.rs` : `RightsProfile`, `PROFILE_MATRIX`, `profile_for` (pas de fichier de droits). Tests.
8. Maicie : `ControlStateWire` et `ControlSnapshot::Read` portent `auto_reassignment: Option<bool>` ; `AutonomousEffect::Reassignment` dans `admit_autonomous_effect` : `None` ou `Some(false)` ⇒ `Deferred { motif: "droits" }` ; l'appelant de `reduire_reassignation` consulte la garde. Test d'effet + mutant.
9. `ui.rs` : `GET /v1/control/rights`, `POST /v1/control/rights/apply` (un seul `ControlStateSet`, `paused` jamais transmis, refus consigné). Tests.
10. `app.js` + `index.html` : page Droits, `RIGHTS_LINES`, profils, `Personnalisé`, lignes locales, mode expert, « Demandé / Réel » ; `theme.css`. Tests Node.

### Phase C - Tester (US3)

11. `control_settings.rs` : `RightsTestAttempt`, `RightsTestOutcome`, `test_gesture(...)`, lecture/écriture 0600 de `server-rights-tests.json`, `resolve_attempt(attempt, journal_records, now)` pure : `passed` / `refused_provider_sandbox` / `unreachable` / `unknown_expired` / `pending` selon le contrat. Tests unitaires de la fonction pure sur des journaux simulés : commande démarrée sans fin ⇒ `pending` ; fin en échec après ligne ⇒ refus ; autre tour avec jeton ⇒ `pending` ; expiration.
12. `ui.rs` : `POST /v1/control/rights/test` (admission : présence libre, dogfooding pour `bridget`, envoi par `post_ui_message`, `message_id` enregistré) ; `GET` résout par lecture du journal filtré sur `message_id`. Tests de route.
13. `app.js` : Tester, résultat daté, `pending`, « ancien ». Tests Node.

### Phase D - Documentation et validation

13. `tests/features/088-droits.feature` ; `docs/regles-chantier.md` ; `quickstart.md` déroulé avec le référent ; ADR-028 statut Accepté.

## Complexity Tracking

Aucune violation. Le seul module nouveau (`refusals.rs`) porte une règle de reconnaissance fermée testée seule ; il a trois usages (wrapper Codex, route de test, rendu).
