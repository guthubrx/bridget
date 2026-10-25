Toutes les preuches sont closes. Voici le rapport final.

---

# Revue de sécurité et conformité — session148 (identité & délégation)

**Verdict : NON APPROUVÉ en l'état.** Un défaut bloquant empêche la fonctionnalité sur le serveur réel. L'architecture de sécurité est saine par ailleurs. Aucun fail-open trouvé.

---

## Finding 1 — CRITIQUE (bloquant) : le client MCP Bridget ne gère pas l'en-tête de session `mcp-session-id`

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-revue-identite-r1.md#extrait-1).

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

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-revue-identite-r1.md#extrait-2).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-revue-identite-r1.md#extrait-3).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-revue-identite-r1.md#extrait-4).

**Direction de correction :** capturer `mcp-session-id` dans la réponse d'initialize. Le renvoyer sur `notifications/initialized` et `tools/call`.

---

## Finding 2 — MAJEUR : sessions HTTP accumulées et coût par appel

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-revue-identite-r1.md#extrait-5).

Chaque appel d'outil refait un `initialize` complet (3 allers-retours HTTP). Les sessions créées ne sont jamais terminées. Elles s'accumulent dans la map `bySessionId` du serveur. Le patch Effect fournit justement `DELETE /mcp` pour ce nettoyage.

**Recommandation :** une session par processus façade, réutilisée, terminée par `DELETE` en fin de vie. Test négatif : vérifier que la map serveur ne croît pas après N appels d'outils.

---

## Observations mineures

1. **Thread supprimé non testé.** `sessionIdentity` (OrchestratorMcpService.ts:1780-1782) refuse les threads `deletedAt`. `BridgetSession.test.ts` ne couvre que l'archivage. Ajouter le cas « thread supprimé ».
2. **Cible test du crate cassée pendant la revue.** `native_delegation.rs:238` provoque `E0502` en mode test. Ce fichier est le moteur de délégation natif, travail en cours d'un autre agent, hors volet identité. Conséquence directe : les tests unitaires du module identité n'ont pas pu être exécutés. Ils ont été lus uniquement.

---

## Propriétés approuvées (avec limites)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-revue-identite-r1.md#extrait-6).

---

## Limites de la revue

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-revue-identite-r1.md#extrait-7).
