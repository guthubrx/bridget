# Archives techniques de l’adaptateur T3

Document d’origine : [specs/133-relais-sous-agents/reuse-audit.md](../../../../../../specs/133-relais-sous-agents/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Marqueur enfant privé | écriture atomique de marqueur typé | `crates/bridget-daemon/src/mcp_identity.rs:52`, `crates/bridget-daemon/src/mcp_identity.rs:414` | Même protection, schéma séparé pour l'autorité réduite. |
| Résolution MCP/CLI | résolution de filiation bornée | `crates/bridget-daemon/src/mcp_identity.rs:233`, `crates/bridget-daemon/src/mcp_identity.rs:337` | Étendre le parcours existant, ne pas créer un résolveur concurrent. |
| Inventaire fournisseur | `ProcessEvidence` et inventaire T3 | `crates/bridget-daemon/src/t3code_identity.rs:20`, `crates/bridget-daemon/src/t3code_identity.rs:531` | Les PID, naissances, sessions et lignées sont déjà collectés. |
| Parent unique | `select_bindings` | `crates/bridget-daemon/src/t3code_identity.rs:437` | Réutiliser la correspondance unique du principal. |
| Détection des fournisseurs imbriqués | `primary_provider_processes` | `crates/bridget-daemon/src/t3code_identity.rs:608` | Étendre la classification au lieu de rescanner l'OS. |
| Admission des outils | répartiteur MCP unique | `crates/bridget-daemon/src/mcp.rs:286`, `crates/bridget-daemon/src/mcp.rs:409` | Refus avant l'appel du handler. |
| Actions enfant permises | handlers `send` et `who` | `crates/bridget-daemon/src/mcp.rs:446`, `crates/bridget-daemon/src/mcp.rs:540` | Aucun nouvel outil public. |
| Provenance | champ optionnel de l'enveloppe existante | `crates/bridget-core/src/message.rs:64`, `crates/bridget-core/src/message.rs:115` | Même pattern que `thread_notice`, avec compatibilité serde. |
| Idempotence | sérialisation canonique existante | `crates/bridget-daemon/src/communication.rs:34`, `crates/bridget-daemon/src/communication.rs:74` | Ajouter la provenance au contrat canonique seulement. |
| Rendu destinataire | trois points de rendu existants | `crates/bridget-transport/src/acp.rs:2065`, `crates/bridget-transport/src/codex_app_server.rs:1298`, `crates/bridget-daemon/src/t3code.rs:3060` | Aucun moteur de rendu supplémentaire. |

<a id="extrait-2"></a>

## Extrait 2

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 064 | propriété parent-enfant | Le parent reste propriétaire, sans réutiliser le registre durable. |
| 090 | sous-agents internes sans identité | La session 133 remplace uniquement l'interdiction totale par un relais borné. |
| 098 | pont T3 en lecture seule | La preuve vient de l'inventaire existant ; T3 n'est pas modifié. |
| 099 | preuve auxiliaire privée | La connexion au daemon réutilise la preuve du parent après admission locale. |
| 101 | rattachement T3 par session native | Même source de preuve et mêmes bornes. |
| 114 | fournisseurs imbriqués exclus du principal | Les imbriqués restent exclus du principal et reçoivent une preuve distincte. |
| 115 | CLI résout une identité attestée | La CLI doit maintenant refuser explicitement la preuve enfant. |

<a id="extrait-3"></a>

## Extrait 3

| Requete | Portee | Resultat |
|---|---|---|
| `rg AgentPidMarker, ResolvedIdentity` | `crates/bridget-daemon` | marqueur et résolution uniques trouvés. |
| `rg ProcessEvidence, primary_provider_processes` | `crates/bridget-daemon` | inventaire T3 et exclusion des imbriqués trouvés. |
| `rg bridget_send, bridget_who, dispatch_with_executor` | `crates/bridget-daemon` | point d'admission unique et handlers existants. |
| `rg BridgetMessage, canonical_message_control` | `crates/` | enveloppe et idempotence existantes. |
| `rg prompt_for, communication_prompt, envelope` | `crates/` | trois rendus à étendre. |
| `rg sous-agents, parent, preuve de rattachement` | `specs/` | sessions 064, 090, 099, 101, 114, 115 pertinentes. |
