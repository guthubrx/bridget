# Archives techniques de l’adaptateur T3

Document d’origine : [specs/181-questions-mobile/reuse-audit.md](../../../../../../specs/181-questions-mobile/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

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

<a id="extrait-2"></a>

## Extrait 2

| Source | Regle | Impact sur le plan |
|---|---|---|
| AGENTS.md | Serveur et Cargo wrapper | Code/compilations serveur ; Mac plateforme uniquement |
| AGENTS.md | Source T3 avant correctif | Lecture ciblée faite ; adaptation native Bridget distincte |
| Constitution XIX | Abstractions justifiées | Un domaine partagé par MCP, CLI et adaptateurs ; aucun framework |
| Constitution XX | Charge cognitive | Même état métier dans chaque interface ; brouillons locaux |
| my-specify-all | Anti-doublon et pipeline | Audit avant tasks ; recherches avant création |
| preuves-correction | Circuit réel et HTML | Recette émulateur explicitement autorisée par l’utilisateur |
| Instruction utilisateur | Autorité native Bridget | Test natif par le parcours natif Bridget obligatoire ; pas de partage de brouillons |

<a id="extrait-3"></a>

## Extrait 3

| Requete | Portee | Resultat |
|---|---|---|
| rg AskUserQuestion/request_user_input/UserInput | crates, android, T3 | Callbacks et formulaire T3 trouvés |
| rg human_inbox/resolve/ChildRequest | domaine, protocol, daemon | Contrats087/155 distincts et inadaptés aux demandes principales libres |
| rg bridget_ask_user/user_input_requests/QuestionForm | crates et Android | Aucun équivalent complet |
| rg notification/RemoteInput/speech | Android et docs officielles | Notification actuelle générique, API native sans nouvelle bibliothèque |
| rg draft | T3 contracts et ChatView | État de réponse client local ; partage exclu par utilisateur |

<a id="extrait-4"></a>

## Extrait 4

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Table et module user_input | CREER | human_inbox.rs:290 et native_child_requests.rs:48 ont une autorité différente ; ne pas les détourner | 2026-10-25 |
| Outil et commande questions | CREER | mcp.rs:2526 et referent_control.rs:29 réutilisés pour identité/attente ; domaine humain distinct | 2026-10-25 |
| Formulaire mobile partagé | REUTILISER/extraire | MainActivity.kt:605 existe ; extraction permet les trois points d’entrée sans dupliquer | 2026-10-25 |
| Brouillons interappareils | Exclure | Choix utilisateur explicite ; dépendance T3 évitée | 2026-10-25 |

<a id="extrait-5"></a>

## Extrait 5

Callbacks181 : réutiliser ChildProviderRequest/Answer, ManagedChildRequests et push_control_message_until. Le normaliseur de formulaire reste dans l’adaptateur d’autorité ; ne pas modifier le format natif du transport155. Le magasin central se nomme user_input_requests, sans nom de fournisseur. La fenêtre TUI Codex opaque conserve son répondant natif ; le chemin universel terminal est l’outil MCP Bridget. Aucun pilotage clavier ni protocole T3 central.
