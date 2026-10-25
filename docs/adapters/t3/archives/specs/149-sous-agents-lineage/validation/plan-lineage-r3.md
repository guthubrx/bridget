# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/plan-lineage-r3.md](../../../../../../../specs/149-sous-agents-lineage/validation/plan-lineage-r3.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Sources relues : constitution globale 1.9.0, skill bridget
(`~/.codex/skills/bridget/SKILL.md`), AGENTS.md du worktree, spec.md, plan.md,
tasks.md, contracts/lineage.md, test-strategy.md (amendée r1), ADR 149,
rapport r1 `plan-lineage-r1.md`. Code relu par sondes ciblées : worktree
Bridget base 148 `6807c22b` et worktree T3 base 148, tous deux inchangés.

<a id="extrait-2"></a>

## Extrait 2

- `tasks.md:65` (T040) : recette UI web sur serveur de recette isolé — daemon
  Bridget fixture et serveur T3 fixture sur port éphémère, base privée, même
  gabarit que l'interop R4 ; interface ouverte dans un navigateur ou la preview
  T3. Phrases de garde présentes : « Cette preuve est une recette UI web ; elle
  ne valide pas la coque desktop native. Aucune seconde application T3 n'est
  lancée ; aucune relance de l'application de production. »
- `plan.md:135` : reformulé à l'identique, avec isolement des providers et
  bases, et « Aucun redémarrage des processus existants n'est permis ».
- `test-strategy.md:9` : amendement G-L r1 consigné ; `test-strategy.md:163-167`
  ajoute S149-28 avec ses oracles et le chemin de preuve future
  `validation/ui-recipe149.md` (fichier absent aujourd'hui, vérifié — correct :
  aucune preuve prétendue).
- Conformité skill : la règle absolue (SKILL.md lignes 505-515) interdit une
  seconde *application* T3, qui partage `~/.t3/userdata`. Un serveur Node/Effect
  fixture sur port éphémère avec base privée est le gabarit de l'interop R4,
  déjà exécuté en 148 (6 PASS). Le gabarit existe :
  `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/BridgetRustInterop.testkit.ts`.
  Aucune violation.

<a id="extrait-3"></a>

## Extrait 3

- `tasks.md:51` : T029 vise le geste `thread.stop` de
  `Orchestrator.ts` et le service BridgetLineage. Vérifié dans le code :
  `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Orchestrator.ts:474`
  contient `case "thread.stop"`. ThreadLifecycleService reste responsable des
  set-*-mode et n'y touche jamais un enfant en cours (garde FR019 écrite).

<a id="extrait-4"></a>

## Extrait 4

| Point | Sonde | Conclusion |
|---|---|---|
| `thread.stop` côté T3 | `Orchestrator.ts:474` | T029 réalisable au bon endroit |
| Gabarit fixture serveur T3 | `BridgetRustInterop.testkit.ts` présent | T040/S149-28 réalisables sans seconde application |
| `watch_ready_seq` | `protocol.rs:2227-2232` | Contrat 149 réutilise un mécanisme testé |
| Masque colonne de gauche | `Sidebar.logic.ts` : `isSidebarSubagentThread` sur `relationshipToParent === "subagent"` | `plan.md:102` reste vrai sur le code 148 |

<a id="extrait-5"></a>

## Extrait 5

Conforme. Pagination O(log N + P) par index root/parent, réconciliation T3 O(n)
avec indexation unique, `descendants_busy` borné (visite 4096). Aucune double
boucle nouvelle. Les annotations de complexité restent à poser dans le code
(T016–T018, T025), comme noté en r1.

<a id="extrait-6"></a>

## Extrait 6

- Table de mapping `tasks.md:87-107` : 19 lignes FR/US, vérifiées par comptage.
  FR001–FR019 toutes présentes, dont FR019 (F2 fermée).
- 45 tâches comptées (T001–T045), progression affichée 0/45, aucune cochée.
- Stratégie : 28 scénarios S149-01 à S149-28, section 10 complète
  (`test-strategy.md:271-303`), tous les FR couverts par au moins un scénario
  ou la régression 148. S149-28 couvre FR001–FR005, FR013, FR015 (SC001, SC002,
  SC006) et ne remplace aucune preuve de service.
- Preuves de phase annoncées et futures uniquement : ui-recipe149.md,
  native-lineage.md, t3-lineage.md, interop149.md, provider-write149.md,
  standalone149.md, recovery149.md, review-final149.md. Aucune n'existe encore.
  Aucun succès réel n'est revendiqué par ce rapport.
