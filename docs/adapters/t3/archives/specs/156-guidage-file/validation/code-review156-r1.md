# Archives techniques de l’adaptateur T3

Document d’origine : [specs/156-guidage-file/validation/code-review156-r1.md](../../../../../../../specs/156-guidage-file/validation/code-review156-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- **Pause, mission et rythme.** La garde `crates/bridget-daemon/src/guidance.rs:60-139` contrôle, dans l'ordre : preuve enfant, identité, humain attaché, client nu, cible elle-même, autre projet, parent, pause, mission active (interruption seulement), puis rythme. Les fenêtres de 30 s et 600 s utilisent une borne incluse, avec `LAST` et `THIRD_LATEST` (`idempotency.rs:1064-1102`). Seuls les événements `admitted` d'un acteur autre que `human` sont comptés. La pause interne n'est possible qu'avec `conn=None`, et seule la pause du référent appelle ce chemin (`daemon.rs:10997`). Le chemin client passe toujours `Some(conn_id)` (`daemon.rs:17666`).
- **Rejeu sur Control.** Le canon, puis `reserve_control_command` et `Replayed`, passent avant la garde (`daemon.rs:10196-10237`). Un rejeu ne réévalue donc rien et ne compte rien.
- **SendIdempotent.** L'événement `admitted` est écrit dans la même transaction que la remise (`send_delivery.rs:89+`). Il n'existe donc que si la remise est commitée.
- **Cancel.** Le retrait de la demande a toujours lieu. L'interruption n'est demandée (`interrupt_active=true`) que si la garde admet et que `admitted` est bien écrit (`daemon.rs:19750-19811`). Une demande déjà annulée n'est plus `open`, donc il n'y a pas de seconde interruption.
- **Provenance.** `normalize_guidance_origin` (`daemon.rs:8992`) retire `System` dans tous les cas. Il retire aussi `Human` sur Control et pour tout acteur qui n'est ni un humain attaché ni l'UI. Le producteur de confiance `System` vient seulement de l'origine déjà normalisée (`daemon.rs:10038`).
- **mid_turn_ok.** La valeur est recalculée à chaque remise (`daemon.rs:9270`). Après une reconnexion, si la connexion d'origine n'est pas prouvée, le résultat est refusé par défaut.
- **Codex.** Le repli à échéance est filtré sur `Human` avec `SteerCurrent` (`codex_app_server.rs`, autour de la ligne 1812). Un steer d'agent avec échéance ne produit donc pas d'interruption, ce qui répond à F1.
- **T3.** Le steer est refusé hors Claude ou hors V2 (`t3code.rs:1949-1957`). Pour Cursor en tour actif, la remise attend, même avec `mid_turn_ok=true` (`t3code.rs:1969-1977`, `3245`).
- **QueueList.** Elle est en lecture seule : un `SELECT` sur les lignes `send_deliveries` en état `dispatching`. Le filtrage suit la règle R5 : la file entière pour un humain, l'UI, la cible ou le parent ; sinon les seuls messages propres, ou les reçus du même scope pour un client nu (`daemon.rs:9318-9389`).

<a id="extrait-2"></a>

## Extrait 2

- Le code de `wrapper.rs`, `acp.rs` (dont la partie Cancel sous le verrou ACP), `managed_session.rs`, `protocol.rs`, `mcp.rs`, `cli.rs`, `delegation_mcp.rs`, `referent_control.rs`, `t3code_contract*.rs`, et le corps de `prepare_dispatch`.
- Les tests, unitaires et E2E, et les corrections de fixtures.
- Tous les documents : spec, plan, reuse-audit, tasks, implementation, `tests-156.md`, `converge156-pass2.*`, `T032-root-validation.json`, `converge156-root-final.json` et la contre-revue historique.
