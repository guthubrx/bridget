Toutes les preuches sont closes. Voici le rapport final.

---

# Revue de sécurité et conformité — session148 (identité & délégation)

**Verdict : NON APPROUVÉ en l'état.** Un défaut bloquant empêche la fonctionnalité sur le serveur réel. L'architecture de sécurité est saine par ailleurs. Aucun fail-open trouvé.

---

## Finding 1 — CRITIQUE (bloquant) : le client MCP Bridget ne gère pas l'en-tête de session `mcp-session-id`

**Fichiers :**
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/session-148-identite-delegation/crates/bridget-daemon/src/t3code_mcp.rs:85-111` (fonction `post`) et `:148` (`notifications/initialized`)
- Serveur : `.../node_modules/effect/.../src/ai/internal/mcpRuntime.ts:443-450`, `.../src/ai/McpServer.ts:869-887`, `.../src/ai/McpProtocol.ts:229`
- Montage : `/Users/moi/11.Repositories/t3code-local/.worktrees/session-148-identite-delegation/apps/server/src/mcp/McpHttpServer.ts:846-850`

**Preuve (sources des deux côtés) :**

`post()` envoie exactement 4 en-têtes : `Authorization`, `Accept`, `Content-Type`, `MCP-Protocol-Version`. Il ne lit jamais les en-têtes de réponse. Il ne transmet jamais `mcp-session-id`.

Le serveur Effect inscrit uniquement `McpProtocol.v2025_06_18`. Ce protocole est `Stateful` (McpProtocol.ts:229). Son admission refuse tout POST non-initialize sans l'en-tête :

```ts
// mcpRuntime.ts:446-450
if (isInitialize ? hasSession : !hasSession && admission.protocol?.runtime._tag !== "Stateless") {
  return reject(400)
}
```

Le patch pnpm `effect@4.0.1` n'ajoute que `DELETE /mcp` et des correctifs ping. Il ne touche pas cette admission.

**Scénario d'échec, pas à pas :**
1. `initialize` part sans en-tête → admis (initialize n'exige pas de session) → réponse 200 avec en-tête `mcp-session-id` que le client **ignore**.
2. `notifications/initialized` part sans en-tête → serveur répond **400**.
3. `post()` refuse (`!(200..300)`, ligne 97-99) → `T3SessionUnavailable`.

**Conséquence :** chaque appel d'outil sous montage T3 échoue sur le serveur réel. US1 et SC001 sont inatteignables. L'échec est fermé (fail-closed) : aucune faille d'identité, mais la fonctionnalité est morte.

**Pourquoi les tests passent quand même :** ils sont synthétiques. Le serveur TCP factice de `t3code_mcp.rs:255-328` répond 202 à `notifications/initialized` **sans vérifier l'en-tête** (ligne 303-306). `BridgetSession.test.ts` mocke la couche service, sans transport HTTP.

**Test négatif requis :** interop réelle contre le serveur T3 (ou un faux serveur qui applique la règle d'admission : 400 sur tout POST non-initialize sans `mcp-session-id`). Attendu : échec avant correction, succès après.

**Direction de correction :** capturer `mcp-session-id` dans la réponse d'initialize. Le renvoyer sur `notifications/initialized` et `tools/call`.

---

## Finding 2 — MAJEUR : sessions HTTP accumulées et coût par appel

**Fichier :** `t3code_mcp.rs:130-157` (`attest`, appelé à chaque résolution d'identité)

Chaque appel d'outil refait un `initialize` complet (3 allers-retours HTTP). Les sessions créées ne sont jamais terminées. Elles s'accumulent dans la map `bySessionId` du serveur. Le patch Effect fournit justement `DELETE /mcp` pour ce nettoyage.

**Recommandation :** une session par processus façade, réutilisée, terminée par `DELETE` en fin de vie. Test négatif : vérifier que la map serveur ne croît pas après N appels d'outils.

---

## Observations mineures

1. **Thread supprimé non testé.** `sessionIdentity` (OrchestratorMcpService.ts:1780-1782) refuse les threads `deletedAt`. `BridgetSession.test.ts` ne couvre que l'archivage. Ajouter le cas « thread supprimé ».
2. **Cible test du crate cassée pendant la revue.** `native_delegation.rs:238` provoque `E0502` en mode test. Ce fichier est le moteur de délégation natif, travail en cours d'un autre agent, hors volet identité. Conséquence directe : les tests unitaires du module identité n'ont pas pu être exécutés. Ils ont été lus uniquement.

---

## Propriétés approuvées (avec limites)

1. **Fail-closed intégral.** Paire d'env partielle, token invalide, T3 arrêté, daemon injoignable : tout refuse sans repli sur l'identité PID (`t3code_mcp.rs:22-41`). FR003 respectée.
2. **Anti-SSRF solide.** Allowlist exacte de 3 formes loopback sur le port du runtime validé par PID vivant. Token contraint en charset et longueur. CRLF refusé (test ligne 196-203). Redirections refusées (minreq `max_redirects(0)` → erreur). Timeout 3 s. Corps plafonné à 64 KiB. SSE limité à une seule frame JSON.
3. **Rejeu et révocation.** Registre en mémoire, hash SHA-256, `revokeThread` à chaque émission, fenêtre de vie 24 h. Côté Bridget, la preuve est revalidée **à chaque appel d'outil**. Le test `http_keeps_sessions_distinct_and_checks_revocation_on_next_call` prouve la distinction de sessions et la révocation à l'appel suivant — sur faux serveur seulement.
4. **Bug originel couvert.** Processus Codex partagé sur 17 sessions : chaque façade reçoit son propre token par thread. Les identités restent distinctes malgré le PID commun.
5. **Rename et reconnexion sûrs.** `stable_uuid` (UUIDv5 sur `thread_id`) ignore le titre du thread. Après reconnexion du daemon, `resolve_binding` revalide l'enregistrement auxiliaire et la connexion authentifiée (`t3code_mcp.rs:44-61`). Le daemon exige une route vivante avec credential identique (`daemon.rs` `register_auxiliary`).
6. **Secrets protégés.** Token jamais loggé côté Rust (erreur opaque unique). Côté T3 : token limité à l'env du sous-processus (`BridgetMcp.ts:44-51`), rien au repos (`mcpSession.ts:46-58`, registry = hash seul).
7. **Périmètre serveur.** `bridget_session` : args fermés vides, refus client externe, refus sans capacité orchestration, refus si le run actif n'appartient pas au provider (`OrchestratorMcpService.ts:1777-1791`).
8. **Indépendance du moteur.** `t3code_mcp` est un connecteur pur, sans appel d'orchestration T3. FR025/FR026 respectées.

---

## Limites de la revue

- Montage T3 déclaré non final. Le code bougeait pendant la revue.
- Tests unitaires non exécutés (cible test cassée par le WIP hors volet).
- Interop réelle non exécutée (interdit prod/modèle). Le finding 1 est établi par lecture croisée des deux sources, vérifiée deux fois à la verbatim.
- Moteur de délégation natif hors volet, conformément à la consigne.
