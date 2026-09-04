# Contrat rights-v1 (relais UI), révisé après contre-revue

## `GET /v1/control/rights`

Réponse : `ControlStateRead` projeté (`generation`, `paused`, `auto_objectives_cap`, `agent_posture`, `auto_reassignment`, `profile` dérivé), pour chaque ligne serveur `{ key, requested, actual, mechanism, storage, reason_if_differs }`, `dogfooding.bridget`, `runtime_capability`, et la dernière tentative de test par ligne, dont l'issue est résolue à cette lecture (voir `test`). Aucune ligne locale : elles sont composées côté navigateur.

## `POST /v1/control/rights/apply`

Corps : `{ "expected_generation": n, "profile": "prudent" | "balanced" | "confident" | "custom", "agent_posture": "...", "auto_reassignment": bool, "auto_objectives_cap": n }`. Un profil nommé doit porter exactement ses valeurs de matrice, sinon `400 invalid_request`. Le relais émet UN `ControlStateSet` (capacité `ControlStateV1`, périmètre `bridget-ui-control`) portant posture, réassignation et plafond ; `paused` n'est jamais transmis. Refus : `403 human_principal_required` (consigné dans le journal du relais avec le pair), `409 generation_mismatch`. Pas d'aperçu séparé : la page calcule l'aperçu depuis la matrice.

## `POST /v1/control/rights/test`

Corps : `{ "line": "shell" | "files" | "internet" | "bridget", "agent_id": "<uuid>" }`. Réponse immédiate : la tentative (`outcome: pending`) ou `409 busy` (agent en plein tour à l'admission), `404 no_agent`, `200 refused_bridget` sans envoi pour `bridget` avec dogfooding désactivé. Résolution à chaque `GET` : journal de l'agent filtré sur `message_id` ; `passed` exige un acte `command` au texte exactement égal à `expected_command`, terminé `completed` avec `exit_code: 0` et le jeton dans `output_tail` ; `refused_provider_sandbox` exige le même item terminé en échec ET un acte `refusal` du même `message_id` ; `unreachable` : `curl` terminé en échec avec code réseau ; `unknown_expired` : 120 s sans terminaison. Aucune autre combinaison n'est classée.

## Signalement de sandbox (liste fermée, non attesté)

| Motif (ligne de sortie) | Condition supplémentaire | Rendu |
|---|---|---|
| `bwrap: ` + `Operation not permitted` ou `Permission denied` | fin du MÊME `commandExecution` en échec (`exit_code ≠ 0` ou `status: failed`) | « Signalement de sandbox (non attesté) », geste Tester |
| `sandbox-exec: ` ou `Sandbox: ` + `deny` | idem | idem |

Une ligne seule, une commande réussie, ou une fin absente ne produisent rien. Ajouter un motif est une modification de ce contrat.

## Provenance du geste local

`gesture.local_toggle` n'est honoré par l'interface que lorsqu'il est construit par le rendu local des références de contenu. Tout refus reçu (journal, daemon, message, serveur relié) est normalisé sans geste local.
