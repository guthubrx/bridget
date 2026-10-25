# Audit de reutilisation de l'existant — 181-questions-mobile

## Decision

Statut: PASS
Date: 2026-10-25
Feature dir: /home/moi/10.Scripts/64.bridget/.worktrees/181-questions-mobile/specs/181-questions-mobile

Le plan étend les transports, l’authentification, le magasin local mobile et les notifications existants. Les questions libres d’agents principaux exigent un stockage distinct des décisions087 et des délégations155. Aucun service ou moteur vocal supplémentaire.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 14 |
| Items audites | 14 |
| Reutilisations deja prevues | 10 |
| Existants potentiellement pertinents | 4 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 7 |
| Specs existantes applicables | 5 |

## Reutilisations correctement identifiees

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/reuse-audit.md#extrait-1).

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Magasin questions libres | Boîte humaine087 | crates/bridget-daemon/src/human_inbox.rs:290 | CREER table additive : resolve accepte seulement une option offerte, pas un formulaire libre |
| Réponse principale native | Demandes enfant155 | crates/bridget-daemon/src/daemon/native_child_requests.rs:48 | CREER domaine séparé : garde Task/enfant/permissions héritées ne doit pas être affaiblie |
| Outil MCP de question | bridget_request_wait | crates/bridget-daemon/src/mcp.rs:2526 | CREER bridget_ask_user, réutiliser attente bornée/annulation et proofclient ; request_wait attend une réponse interagent, pas humaine |
| Commande humaine terminal | Contrôle humain CLI | crates/bridget-daemon/src/referent_control.rs:29 | CREER sous-commande questions, réutiliser principal humain existant ; ne pas permettre au producteur de se répondre |

## Duplications evidentes

Aucune. Les éléments proches ont des contrats d’autorité différents ; les réutilisations sont explicites avant implémentation.

## Memoires et regles applicables

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/reuse-audit.md#extrait-2).

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 087 | Décisions humaines durables | Réutiliser principes de reçu/ACK sans changer options offertes |
| 155 | Callback natif et permissions | Réutiliser trames, conserver l’isolement enfant |
| 177 | Auth mobile et notifications | Même appareil associé et poussée minimale |
| 178 | Rendu et sécurité | Garder les composants de texte et protections |
| 180 | Commandes durables/brouillons | Même logique de résultat inconnu et rejeu |

## Journal de recherche

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/reuse-audit.md#extrait-3).

## Arbitrages

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/reuse-audit.md#extrait-4).

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Gate implémentation — fondations

Recherche par nom user_input/UserInput et responsabilité human_inbox/ChildRequest effectuée avant création. CREER daemon/user_input.rs pour la seule autorité de connexion ; réutiliser daemon live_connection_identity/control_client_actor et communication/client.rs pour le transport négocié. Créer UserInputV1 et les deux trames de résultat : pas de protocole équivalent ; même matrice fermée. Les fonctions de validation et stockage restent dans user_input.rs, sans nouvelle couche partagée.

Attente et reçu MCP : REUTILISER WaitPermit, wait_client_gone et delegation_store.wait_epoch (daemon.rs:202/220/274), check_tool_call et write_shared_response (mcp.rs:148/403). Aucun nouvel ordonnanceur d’attente. Le petit état local de reçu MCP est nécessaire pour ne confirmer la livraison qu’après flush réussi ; il ne contient ni question ni réponse et reste limité à l’appel. La commande questions réutilise le principal bridget-control-cli et sa négociation.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/reuse-audit.md#extrait-5).
