# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/validation/glm-revue-identite-r1.md](../../../../../../../specs/148-identite-delegation/validation/glm-revue-identite-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**Fichiers :**
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/session-148-identite-delegation/crates/bridget-daemon/src/t3code_mcp.rs:85-111` (fonction `post`) et `:148` (`notifications/initialized`)
- Serveur : `.../node_modules/effect/.../src/ai/internal/mcpRuntime.ts:443-450`, `.../src/ai/McpServer.ts:869-887`, `.../src/ai/McpProtocol.ts:229`
- Montage : `/Users/moi/11.Repositories/t3code-local/.worktrees/session-148-identite-delegation/apps/server/src/mcp/McpHttpServer.ts:846-850`

<a id="extrait-2"></a>

## Extrait 2

**Conséquence :** chaque appel d'outil sous montage T3 échoue sur le serveur réel. US1 et SC001 sont inatteignables. L'échec est fermé (fail-closed) : aucune faille d'identité, mais la fonctionnalité est morte.

<a id="extrait-3"></a>

## Extrait 3

**Pourquoi les tests passent quand même :** ils sont synthétiques. Le serveur TCP factice de `t3code_mcp.rs:255-328` répond 202 à `notifications/initialized` **sans vérifier l'en-tête** (ligne 303-306). `BridgetSession.test.ts` mocke la couche service, sans transport HTTP.

<a id="extrait-4"></a>

## Extrait 4

**Test négatif requis :** interop réelle contre le serveur T3 (ou un faux serveur qui applique la règle d'admission : 400 sur tout POST non-initialize sans `mcp-session-id`). Attendu : échec avant correction, succès après.

<a id="extrait-5"></a>

## Extrait 5

**Fichier :** `t3code_mcp.rs:130-157` (`attest`, appelé à chaque résolution d'identité)

<a id="extrait-6"></a>

## Extrait 6

1. **Fail-closed intégral.** Paire d'env partielle, token invalide, T3 arrêté, daemon injoignable : tout refuse sans repli sur l'identité PID (`t3code_mcp.rs:22-41`). FR003 respectée.
2. **Anti-SSRF solide.** Allowlist exacte de 3 formes loopback sur le port du runtime validé par PID vivant. Token contraint en charset et longueur. CRLF refusé (test ligne 196-203). Redirections refusées (minreq `max_redirects(0)` → erreur). Timeout 3 s. Corps plafonné à 64 KiB. SSE limité à une seule frame JSON.
3. **Rejeu et révocation.** Registre en mémoire, hash SHA-256, `revokeThread` à chaque émission, fenêtre de vie 24 h. Côté Bridget, la preuve est revalidée **à chaque appel d'outil**. Le test `http_keeps_sessions_distinct_and_checks_revocation_on_next_call` prouve la distinction de sessions et la révocation à l'appel suivant — sur faux serveur seulement.
4. **Bug originel couvert.** Processus Codex partagé sur 17 sessions : chaque façade reçoit son propre token par thread. Les identités restent distinctes malgré le PID commun.
5. **Rename et reconnexion sûrs.** `stable_uuid` (UUIDv5 sur `thread_id`) ignore le titre du thread. Après reconnexion du daemon, `resolve_binding` revalide l'enregistrement auxiliaire et la connexion authentifiée (`t3code_mcp.rs:44-61`). Le daemon exige une route vivante avec credential identique (`daemon.rs` `register_auxiliary`).
6. **Secrets protégés.** Token jamais loggé côté Rust (erreur opaque unique). Côté T3 : token limité à l'env du sous-processus (`BridgetMcp.ts:44-51`), rien au repos (`mcpSession.ts:46-58`, registry = hash seul).
7. **Périmètre serveur.** `bridget_session` : args fermés vides, refus client externe, refus sans capacité orchestration, refus si le run actif n'appartient pas au provider (`OrchestratorMcpService.ts:1777-1791`).
8. **Indépendance du moteur.** `t3code_mcp` est un connecteur pur, sans appel d'orchestration T3. FR025/FR026 respectées.

<a id="extrait-7"></a>

## Extrait 7

- Montage T3 déclaré non final. Le code bougeait pendant la revue.
- Tests unitaires non exécutés (cible test cassée par le WIP hors volet).
- Interop réelle non exécutée (interdit prod/modèle). Le finding 1 est établi par lecture croisée des deux sources, vérifiée deux fois à la verbatim.
- Moteur de délégation natif hors volet, conformément à la consigne.
