# Contrat 098 — t3code vu de Bridget (seul point de dépendance, révision 2)

## Lu
- `~/.t3/userdata/server-runtime.json` : `origin`, `pid` ; `version` = format du fichier, non l'API.
- `GET /api/orchestration/snapshot` (Bearer) : fils (`id`, `projectId`, `title`, `worktreePath`, `archivedAt`, `deletedAt`, `updatedAt`, `latestTurn`, `session{providerName, providerInstanceId, status, activeTurnId}`) ; `archivedAt`/`deletedAt` renseignés = fin de vie ; changement détecté par `updatedAt`/`latestTurn`.
- `GET /api/orchestration/threads/:threadId?turnLimit=N` (Bearer) : `thread.messages[{id, role, text, streaming, turnId}]` (utilisateur : `turnId` nul ; assistant : `turnId` du tour), `thread.latestTurn{turnId, state, assistantMessageId}`, `page`.
Chaque réponse est validée champ par champ ; une forme inattendue est un refus nommé.

## Écrit
- `POST /api/orchestration/dispatch` (Bearer) : `{type:"thread.turn.start", commandId, threadId, message:{messageId, role:"user", text, attachments:[]}, runtimeMode, interactionMode, createdAt}` → `{sequence}` ; `commandId` = identifiant de remise ; jamais envoyé tant qu'un `activeTurnId` est observé, dans une borne.
- `t3 auth session issue --subject bridget --label bridget-<installation> --ttl <d> --json` (jeton, identifiant de session, expiration) ; `t3 auth session list --json` pour retrouver une session par `client.label` ; `t3 auth session revoke <id>` au retrait ou après un reçu illisible. Seule cette commande officielle touche la base d'authentification de t3code.

## Refusé
- Toute autre route ; `settings.json` ; `providerInstances` ; bases de t3code ; `T3CODE_DEV_AUTH_TOKEN` ; RPC WebSocket ; toute identité déduite d'une variable d'environnement.

## Garanties Bridget
- Une identité par fil, présence `t3code | cli`, retirée à l'archivage, stable après redémarrage.
- Remise : un seul tour par identifiant de remise ; l'accusé de remise suit l'acceptation HTTP du `dispatch` (le tour est démarré), ce qui n'est pas la réponse de l'assistant ; 401 → un renouvellement puis un seul nouvel essai ; remises d'un même fil sérialisées.
- Réponse liée renvoyée par le pont seulement pour le tour assistant de même rang que le message du pont (`messageId`), une fois ce tour complet (`streaming = false`, `latestTurn.state = completed`) ; rang non établi → demande laissée ouverte et ambiguïté journalisée ; la corrélation en attente survit à un redémarrage du pont.
- Adresse : uniquement `http://127.0.0.1:<port>` ; toute autre origine refusée.
- Journal : messages du fil, sans rejeu d'historique, sans doublon, sans jeton.
