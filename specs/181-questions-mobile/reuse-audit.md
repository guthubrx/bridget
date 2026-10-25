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

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Contrat/matrice de rôle | protocol.rs et daemon.rs | crates/bridget-daemon/src/daemon.rs:19793 | Étendre le protocole fermé et les matrices, pas de canal alternatif |
| Identité MCP | Résolution et preuve communication | crates/bridget-daemon/src/mcp.rs:618 | Aucune identité depuis le texte du modèle |
| Callbacks Codex | ChildProviderRequest/Answer | crates/bridget-transport/src/codex_app_server.rs:982 | Adapter uniquement UserInput d’agent principal, préserver155 |
| Transport Claude | control_request can_use_tool | crates/bridget-transport/src/claude_stream_json.rs:1575 | Reconnaître AskUserQuestion séparément ; autres outils toujours contrôlés |
| Adaptateur T3 | Client RPC et parseur V2 | crates/bridget-daemon/src/t3code_contract.rs:692 ; t3code_contract_v2.rs:117 | Aucun protocole T3 au centre du magasin |
| Handler/API mobile | Questions et receipt | crates/bridget-daemon/src/daemon/mobile_questions.rs:70 ; mobile/http.rs:770 | Étendre les routes et la commande existante |
| Interface questions Android | Questions et composeur | android/app/src/main/java/app/cartae/bridget/MainActivity.kt:605 | Extraire/étendre pour réemploi dans le fil, sans seconde logique de formulaire |
| Brouillons/commandes | LocalStore | android/app/src/main/java/app/cartae/bridget/MobileViewModel.kt:675 | Persistance/rejeu même clé réemployés |
| Notification et RemoteInput | MobileNotifications | android/app/src/main/java/app/cartae/bridget/BridgetMessagingService.kt:140 | Poussée générique puis lecture authentifiée ; receiver non exporté ajouté au même parcours |
| Dictée | API Android système | android/app/src/main/java/app/cartae/bridget/MainActivity.kt:65 | Reprendre les lanceurs ActivityResult ; aucune dépendance vocale |

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

| Source | Regle | Impact sur le plan |
|---|---|---|
| AGENTS.md | Serveur et Cargo wrapper | Code/compilations serveur ; Mac plateforme uniquement |
| AGENTS.md | Source T3 avant correctif | Lecture ciblée faite ; adaptation native Bridget distincte |
| Constitution XIX | Abstractions justifiées | Un domaine partagé par MCP, CLI et adaptateurs ; aucun framework |
| Constitution XX | Charge cognitive | Même état métier dans chaque interface ; brouillons locaux |
| my-specify-all | Anti-doublon et pipeline | Audit avant tasks ; recherches avant création |
| preuves-correction | Circuit réel et HTML | Recette émulateur explicitement autorisée par l’utilisateur |
| Instruction utilisateur | Indépendance T3 | Test natif sans T3 obligatoire ; pas de partage de brouillons |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 087 | Décisions humaines durables | Réutiliser principes de reçu/ACK sans changer options offertes |
| 155 | Callback natif et permissions | Réutiliser trames, conserver l’isolement enfant |
| 177 | Auth mobile et notifications | Même appareil associé et poussée minimale |
| 178 | Rendu et sécurité | Garder les composants de texte et protections |
| 180 | Commandes durables/brouillons | Même logique de résultat inconnu et rejeu |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| rg AskUserQuestion/request_user_input/UserInput | crates, android, T3 | Callbacks et formulaire T3 trouvés |
| rg human_inbox/resolve/ChildRequest | domaine, protocol, daemon | Contrats087/155 distincts et inadaptés aux demandes principales libres |
| rg bridget_ask_user/user_input_requests/QuestionForm | crates et Android | Aucun équivalent complet |
| rg notification/RemoteInput/speech | Android et docs officielles | Notification actuelle générique, API native sans nouvelle bibliothèque |
| rg draft | T3 contracts et ChatView | État de réponse client local ; partage exclu par utilisateur |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Table et module user_input | CREER | human_inbox.rs:290 et native_child_requests.rs:48 ont une autorité différente ; ne pas les détourner | 2026-10-25 |
| Outil et commande questions | CREER | mcp.rs:2526 et referent_control.rs:29 réutilisés pour identité/attente ; domaine humain distinct | 2026-10-25 |
| Formulaire mobile partagé | REUTILISER/extraire | MainActivity.kt:605 existe ; extraction permet les trois points d’entrée sans dupliquer | 2026-10-25 |
| Brouillons interappareils | Exclure | Choix utilisateur explicite ; dépendance T3 évitée | 2026-10-25 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Gate implémentation — fondations

Recherche par nom user_input/UserInput et responsabilité human_inbox/ChildRequest effectuée avant création. CREER daemon/user_input.rs pour la seule autorité de connexion ; réutiliser daemon live_connection_identity/control_client_actor et communication/client.rs pour le transport négocié. Créer UserInputV1 et les deux trames de résultat : pas de protocole équivalent ; même matrice fermée. Les fonctions de validation et stockage restent dans user_input.rs, sans nouvelle couche partagée.

Attente et reçu MCP : REUTILISER WaitPermit, wait_client_gone et delegation_store.wait_epoch (daemon.rs:202/220/274), check_tool_call et write_shared_response (mcp.rs:148/403). Aucun nouvel ordonnanceur d’attente. Le petit état local de reçu MCP est nécessaire pour ne confirmer la livraison qu’après flush réussi ; il ne contient ni question ni réponse et reste limité à l’appel. La commande questions réutilise le principal bridget-control-cli et sa négociation.

Callbacks181 : réutiliser ChildProviderRequest/Answer, ManagedChildRequests et push_control_message_until. Le normaliseur de formulaire reste dans l’adaptateur d’autorité ; ne pas modifier le format natif du transport155. Le magasin central se nomme user_input_requests, sans nom de fournisseur. La fenêtre TUI Codex opaque conserve son répondant natif ; le chemin universel terminal est l’outil MCP Bridget. Aucun pilotage clavier ni protocole T3 central.
