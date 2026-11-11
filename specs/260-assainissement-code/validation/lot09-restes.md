# Fonctions longues et fonctions à nombreux paramètres - restes du lot 9

Relevé du 11 novembre 2026 par `outils/longues_fonctions.py` (mesure de contrôle). Listes brutes : `lot09-restes-longues.txt` (44 fonctions de 300 lignes ou plus, hors tests) et `lot09-restes-params.txt`.

## Structures d'arguments (T100 à T104)

| Fonction | Sort |
|----------|------|
| `prompt_request` (acp.rs, 20 paramètres) | fait : `PromptRequestArgs` |
| `resolve_target` (native_delegation.rs, 17) | fait : `ResolveTargetArgs` |
| `reconnect_managed_session` (wrapper.rs, 17) | fait : `ReconnectManagedSessionArgs` |
| `connect_and_register_at` (wrapper.rs, 16) | fait : `ConnectAndRegisterAtArgs` |
| `connect_and_register_with_domain_at` (wrapper.rs, 15) | fait : `ConnectAndRegisterWithDomainAtArgs` |
| `handle_register_with_channel` (daemon.rs, 17) | arrêtée : 66 appels (plus de 30, règle du guide) |
| `post` (threads.rs, 15) | arrêtée : le nom `post` désigne plusieurs fonctions, l'outil refuse un appel à un seul argument |
| `apply` (wrapper_workspace_repair.rs, 15) | arrêtée : même cause (`apply` à un argument ailleurs) |
| `spawn_reader` (claude_stream_json.rs, 15) | arrêtée : même cause (plusieurs `spawn_reader`) |

## Fonctions de 300 lignes ou plus

Traitée dans ce lot : `handle_wrapper_message_with_peer` (5 010 -> 1 294 lignes, lot 5).
**Non traitées** (T105 à T111, découpage manuel de fonctions de 600 à 1 400 lignes avec de nombreuses variables locales partagées) : `launch_session_with_status` (1 359 lignes ; une boucle de 783 lignes), `spawn_reader` de `codex_app_server.rs` (1 098), `tick` (981, verrou global tenu), `launch` (796), `admit_delegate` (777), `apply_specialist_inner` (727), `handle_idempotent_send_with_result_authority` (625), et 36 autres. Critère SC09 (aucune fonction de plus de 1 000 lignes) : **non atteint** (2 fonctions restent au-dessus : `launch_session_with_status` et `spawn_reader`).

Exemptions écrites dans le code : `tools` (mcp.rs, données), deux `init_schema` (idempotency.rs, store.rs : SQL). Huit fonctions de 12 à 14 paramètres portent un commentaire « regroupement différé ».
