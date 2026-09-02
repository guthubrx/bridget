# Contrat : origine d'une délégation déposée au guichet

Extension additive de `ServiceRequestPayload::Delegate` :

```json
{ "goal": "…", "review_target": null, "explicit_target": null, "required_tags": [],
  "duration": "normale", "suite": { … }, "depends_on": [], "references": [],
  "origin": null | { "kind": "human", "message_id": "…",
                     "observed": { "message_id": "…", "ts": 1788400000, "sender": "humain", "target": "maicie", "body": "…" },
                     "attestation": { "version": 1, "issuer_scope": "…", "canonical_request_sha256": "…", "signature": "…" } },
  "focus": null | { "on_conflict": "replace" | "queue", "project_id": "…" } }
```

Règles daemon (`handle_wrapper_message`, bras `ServiceRequest`) :

1. Si l'émetteur attribué n'est pas le principal humain et que `origin` ou `focus` est non nul : `ServiceRefusal::HumanOriginForbidden`, rien n'est déposé.
2. Si l'émetteur est le principal humain : le daemon ignore tout `origin` fourni, enregistre le message humain au ledger (émetteur `humain`, cible `maicie`, corps = `goal`), fabrique `origin.human` avec `signature = human_message_content_seal(observed)` et `canonical_request_sha256` calculé sur les octets canoniques du dépôt **sans** le champ `origin` (le hash ne se définit pas sur un document qui le porte déjà, ADR-014), puis dépose.
3. `required_contract_version()` rend `REVIEW_DELEGATE_CONTRACT_VERSION` dès que `origin` ou `focus` est présent.

Règles Maicie (`guichet.rs` puis `greffe_service.rs`) :

1. `origin = null` → `ObjectiveOpeningPermit::auto_generated()`, soumis au budget.
2. `origin.human` → `ObjectiveOpeningPermit::human_request(attestation, observed, issuer_scope, canonical_request_sha256, consumption)` ; `consumption` vient de `human_origin_consumptions`. Tout refus `HumanOriginRefusal` devient `GuichetRefusalReason::HumanOriginInvalid` (code public unique, motif interne journalisé).
3. `focus` présent avec origine humaine valide → objectif inscrit dans `focus_queue` en position 0 (`replace` : l'ancien focus passe en position 1) ou en fin de file (`queue`). `focus` sans origine humaine → refus `HumanOriginInvalid`.
4. Pour un focus, `review_target` absent est calculé par `review_git` sur `origin/<branche par défaut>` du projet ; l'instruction porte le bloc `IDENTIFIANTS DE DÉPÔT`.
5. Un focus humain dont les agents correspondants sont tous momentanément indisponibles répond `waiting_for_agent` avec son `objective_id`, sans `delegation_id`, sans participant ni message. Il reste ouvert en tête de la file et la relève dépose ensuite l’item `focus_waiting_agent`.

Mémoire projet appliquée : « additif sur le fil n'est pas additif à la source ». Tous les initialiseurs `ServiceRequestPayload::Delegate { … }` (protocole, guichet, tests) reçoivent `origin: None, focus: None`. Le gate `cargo test --workspace --no-run` sur la composition fait partie des tâches.
