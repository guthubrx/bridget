# Archives techniques de l’adaptateur T3

Document d’origine : [specs/156-guidage-file/reuse-audit.md](../../../../../../specs/156-guidage-file/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Conclusion courte : 24/24 items confrontés à l’existant. Aucun doublon non arbitré.
Le plan réutilise les magasins, transports, preuve UI, liens, canon et ACK.
Les ajouts prévus portent une règle ou une lecture absente. Claude seul a le
guidage T3 natif ; Cursor explicite/implicit busy et REST/V1 sont fermés selon
les décisions root. Ce PASS concerne la conception, pas l’implémentation156.

<a id="extrait-2"></a>

## Extrait 2

Décisions item par item du rapport frais. Les chemins courts de la colonne
preuve ont pour racine absolue /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/ ;
les chemins complets, plages et limites sont dans research.md et son reçu T002.
| Item | Décision | Preuve/raison |
|---|---|---|
| MessageIntent/canon | REUTILISER | message.rs:54 ; daemon.rs:9035/23888 |
| Refus/rejeu | REUTILISER | daemon.rs:8132 ; idempotency_records |
| Remise/reprise/ACK | REUTILISER | daemon.rs:4496/4593/16915 |
| Contrôle actif | REUTILISER | daemon.rs:9429 ; wrapper.rs:5833 |
| Identité/provenance UI/Attach | REUTILISER | daemon.rs:8607/8768/16289/14722 |
| Parent lien et natif | REUTILISER | agent_links.rs:79 ; delegation.rs: 174/193 |
| Pause/AgentGuidance | ETENDRE | referent_control.rs:129 |
| Transports deliver | REUTILISER | codex_app_server.rs:902 ; claude_stream_json.rs:310 ; acp.rs:614 |
| dispatch pont T3 | ETENDRE | t3code_contract_v2.rs:253 ; t3code.rs:1960/3126 |
| GuidanceFacts/authorize/refus/module | CREER JUSTIFIE | Pas garde inter-agent équivalente ; règle pure >=3 puits |
| guidance_events + deux index | CREER JUSTIFIE | control_events fermé pause/droits referent_control.rs:44 ; pas reçu de rejeu |
| record/count/purge audit | CREER dans magasin EXISTANT | run_maintenance daemon.rs:7630 ; aucune couche service |
| send_deliveries source | REUTILISER | idempotency.rs:527 ; phases inchangées |
| index recipient/phase | CREER JUSTIFIE | index opération/clé :541/546 non équivalents |
| queued_deliveries_for_recipient | CREER JUSTIFIE | dispatching_deliveries_for_instance send_delivery.rs:394 mute ; lecture pure distincte |
| QueueListV1/QueueList/Result | CREER JUSTIFIE | patron capability protocol.rs:3992 ; pas lecture file publique |
| NotPermitted contrôle | ETENDRE | résultat contrôle existant daemon.rs:9429 |
| mid_turn_ok remises | ETENDRE compatible | protocol.rs:4228/4309 ; option de projection hors canon |
| CancelDelivery.interrupt_active | ETENDRE | protocol.rs:4322 ; daemon.rs:18764 |
| cancel_queued | CREER JUSTIFIE | ManagedSession existant ; ACP cancel_delivery :511 touche actif |
| MCP intent/queue | ETENDRE/CREER demandé | mcp.rs ; patterns schema/auth/whitelist existants |
| CLI intent/queue | ETENDRE | cli.rs:3496/4358 |
| tests/E2E | CREER/ETENDRE prévus | native_delegation_e2e existant ; aucun scénario de confort |
| docs/ADR156 | ETENDRE/CREER demandé | documentation existante ; ADR du choix nécessaire |
