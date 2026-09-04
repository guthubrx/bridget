# Data Model: Droits (révisé après contre-revue)

## Table `control_state` (daemon, SQLite, SPEC-087 étendue)

Colonnes ajoutées : `agent_posture TEXT NOT NULL CHECK (agent_posture IN ('discovery','complete'))`, `auto_reassignment INTEGER NOT NULL CHECK (auto_reassignment IN (0,1))`. Base neuve : `discovery`, `0`. Base existante : `complete`, `1` (comportement observable avant migration). Écriture par `ControlStateSet` sous la même génération que pause et plafond ; `control_events.kind` gagne `rights_set`.

## Trame `ControlStateFrame` (transport, additif)

`agent_posture: Option<AgentPosture>` et `auto_reassignment: Option<bool>`, défaut `None` = inconnu. `ControlStateSet` gagne `agent_posture: Option<AgentPosture>` et `auto_reassignment: Option<bool>`.

## Profil (dérivé, jamais stocké)

`profile_for(agent_posture, auto_reassignment, auto_objectives_cap)` ∈ {prudent, balanced, confident, custom}, matrice `PROFILE_MATRIX` (research R6).

## Fichier `server-rights-tests.json` (relais, 0600) : tentatives de test

```json
{ "test_id": "<uuid>", "line": "shell" | "files" | "internet" | "bridget", "agent_id": "<uuid>",
  "message_id": "<id rendu par /v1/send>", "expected_command": "<commande canonique>", "expected_sha256": "<hex>",
  "token": "BRIDGET-TEST-<court>", "started_at": 1788500000,
  "outcome": "pending" | "passed" | "refused_provider_sandbox" | "refused_server_runtime" | "refused_bridget" | "unreachable" | "busy" | "no_agent" | "unknown_expired",
  "raw": "<ligne brute ≤ 512>", "finished_at": 1788500030 | null }
```

Une tentative par ligne ; la précédente est remplacée. Expiration : 120 s après `started_at` sans terminaison.

## Actes du journal (wrapper Codex)

- Acte `command` : `{ kind: "command", text: <commande>, item_id }` à `item/started` ; MIS À JOUR à `item/completed` type `commandExecution` : `state: completed | failed`, `exit_code`, `output_tail` (≤ 512 caractères), `item_id`.
- Acte `refusal` : `{ kind: "refusal", layer: "provider_sandbox", evidence: "output_and_exit", provider, posture, prevented: "shell", raw, item_id, gesture: { kind: "rights_line", target: "shell" }, attributed_to: "bridget" }`, écrit seulement quand la ligne reconnue ET la fin en échec concernent le même `item_id`. Un par `(tour, item)`.

## Objet `Refus` (interface)

`{ layer, prevented, gesture: { kind: "local_toggle" | "rights_line" | "none", target }, raw, at, attributed_to, evidence, provider, posture }`. `local_toggle` ne survit à la normalisation que si le rendu local l'a construit.

## Préférences locales (navigateur)

`bridget.content-security.v1` inchangé. Nouveaux : `bridget.rights.local-profile.v1` (dernier profil appliqué localement) et `bridget.rights.expert.v1` (mode expert de la page).
