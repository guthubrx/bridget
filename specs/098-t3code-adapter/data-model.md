# Modèle 098 (révision 2)

- **Environnement t3code** : `port` et `pid` lus dans `server-runtime.json` (son `version` = format du fichier), adresse reconstruite `http://127.0.0.1:<port>` ; origine non locale ou PID absent = refus nommé. Absent = t3code non démarré, état explicite.
- **Jeton** : sujet `bridget`, libellé (`client.label`) d'installation unique `bridget-<id>` permettant de retrouver et révoquer la session même sans reçu.
- **Jeton** : valeur, `session_id`, `expires_at`, sujet `bridget` ; 0600 dans l'état privé ; renouvellement borné ; révocation par `session_id`.
- **Manifeste d'installation** : étapes posées (jeton, service) pour un retrait exact et un rollback par étape.
- **Fil hébergé** : `thread_id`, `provider_name`, `provider_instance_id`, `worktree_path`, `status`, `active_turn_id` ; dérivés : `agent_id` (UUID v5), `instance_id`, domaine.
- **Remise** : réutilise `send_deliveries` ; `commandId` = `delivery_id`, `messageId` choisi par le pont ; phases `dispatching → acked` après `DispatchResult`, `indeterminate` sinon.
- **État durable par fil** (`t3code-state.json`, un seul fichier, écriture atomique) : section `cursor` (séquence des identifiants de messages projetés, dans l'ordre du fil) et section `pending` (corrélations en attente : `delivery_id`, identifiant de demande, émetteur, `messageId`, `threadId`), écrite avant le `dispatch`, reprise au démarrage, effacée après réponse liée ou ambiguïté.
- **Repère d'installation** : instant et séquence de snapshot à l'installation ; tout ce qui est postérieur est projeté.
- **File de remise par fil** : remises en attente, servies une à la fois par un worker ; état de chaque remise (attente d'inactivité, envoyée, acquittée, indéterminée).
- **Journal par fil** : `turn_start {from}` (message humain ou remise Bridget), `update {kind:text}` (assistant), `turn_end` ; lacune annoncée si des identifiants disparaissent.
