# Contre-revue adverse — agent bridget-revue (Codex)

## Revue du plan, tour 1

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/adversarial-review-bridget-revue.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/adversarial-review-bridget-revue.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/adversarial-review-bridget-revue.md#extrait-3).

## Arbitrage A/B (demandé par l'humain)

- Question : v1 sans émission (A) ou émission avec attestation d'ascendance (B) ?
- Réponse de bridget-revue : **A**, car le contrat public n'offre aucune clé exacte fil ↔ processus MCP ; B conserverait une identité fausse ou usurpable. Accord des deux agents sur A.

## Revue du plan, tour 2 (révision 2)

- Date : 2026-09-14 08:24 (CEST) · Verdict reçu : **BLOCKED** (6 objections).

| objection | verifiee comment | retenue oui/non | raison |
|---|---|---|---|
| 1. Corrélation réponse ↔ demande impossible par `turnId` | `orchestration.ts:494,601,2099`, `decider.ts:1308` | oui | corrélation par le `messageId` choisi par le pont, dans l'ordre du fil ; ambiguïté → demande laissée ouverte, journalisée |
| 2. Attente d'inactivité dans la fermeture synchrone bloquerait la boucle ; remises concurrentes | `wrapper.rs:751` | oui | file et worker par fil, remises sérialisées, tests concurrence et déconnexion |
| 3. Origine non locale possible, jeton en clair | `serverRuntimeState.ts:47` | oui | seule `http://127.0.0.1:<port>` reconstruite est acceptée |
| 4. « Aucune écriture dans la base » contredit `session issue` ; reçu tronqué = jeton orphelin | `cli/auth.ts:162`, `AuthSessions.ts:215` | oui | formulation corrigée ; libellé unique, récupération et révocation par libellé |
| 5. Curseur initialisé à l'état courant perd des événements ; dernier identifiant sans ordre | plan.md, data-model.md | oui | repère d'installation + séquence durable par fil, projection de tout ce qui suit le repère |
| 6. Lignes périmées dans research.md et contrat confondant accusé et réponse | relecture | oui | nettoyé |

## Revue du plan, tour 3 (révision 3)

- Date : 2026-09-14 08:29 (CEST) · Verdict reçu : **APPROVE_WITH_CHANGES** (4 changements locaux, architecture et option A confirmées).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/adversarial-review-bridget-revue.md#extrait-4).

Issue : plan accepté avec ces changements, appliqués avant la génération des tâches. Trois tours, 18 objections, toutes vérifiées et retenues ; aucune rejetée.


## Tour 4 — Contre-revue de l'implémentation (2026-09-14, 10:46)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/adversarial-review-bridget-revue.md#extrait-5).

**Verdict reçu : BLOCKED**, six objections. Cinq retenues et corrigées, une close par la revue elle-même.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/adversarial-review-bridget-revue.md#extrait-6).

Aucune objection écartée. Le point 6 a en outre révélé un défaut que la revue n'avait pas visé : la politique était figée à l'ouverture du lien, et le test l'a mis en évidence.
