# Revue G-L r3 — plan lineage session149

Revue : GLM 5.3 Flash (`glm-5.3-flash`). Date : 2026-10-10. Périmètre : G-L
(Lineage, lectures natives, journal, filtre du pont). G-P reste hors de ce
rapport et hors de décision ici.

Contexte : la ronde 2 a échoué avant tout travail (« The provider could not
start this turn »). Elle n'a produit ni verdict ni rapport fiable. La présente
ronde refait la revue sur les documents corrigés depuis r1. Elle ne reprend
aucun contexte d'un fil enfant. Ownership de cette ronde : ce fichier seul.

Sources relues : constitution globale 1.9.0, skill bridget
(`~/.codex/skills/bridget/SKILL.md`), AGENTS.md du worktree, spec.md, plan.md,
tasks.md, contracts/lineage.md, test-strategy.md (amendée r1), ADR 149,
rapport r1 `plan-lineage-r1.md`. Code relu par sondes ciblées : worktree
Bridget base 148 `6807c22b` et worktree T3 base 148, tous deux inchangés.

## Verdict : APPROVE

Les sept findings de r1 sont fermés dans les cinq documents corrigés. Aucune
nouvelle objection justifiée trouvée. Le volet G-L est réalisable sur le code
148 existant. La gate partielle G-L ouvre bien T016–T035, avec un ownership
unique par fichier et sans course d'écriture sur un même fichier.

## Fermeture des findings r1

### F1 (HIGH) — FERMÉ. Recette desktop remplacée par une recette UI web

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

### F2 (MEDIUM) — FERMÉ. FR019 mappée

- `tasks.md:107` : ligne `| FR019 | T007,T010,T013,T015,T023,T027–T028,T039 |`
  présente, avec les tâches demandées en r1. Toutes ces tâches existent dans
  T001–T045. FR019 est aussi citée dans T029 (`tasks.md:51`) et l'explication
  de mapping couvre FR018 et FR019 (`tasks.md:109`).

### F3 (LOW) — FERMÉ. Comptes de scénarios à jour

- `plan.md:139` et `tasks.md:115` citent « S149-01 à S149-28 ». Plus aucune
  occurrence de 21 scénarios dans plan.md ni tasks.md. La fiche spec affiche
  « Tests: 0/28 » (`spec.md:12`), conforme aux 28 scénarios de la stratégie.

### F4 (LOW) — FERMÉ. T029 vise le bon point d'arrêt

- `tasks.md:51` : T029 vise le geste `thread.stop` de
  `Orchestrator.ts` et le service BridgetLineage. Vérifié dans le code :
  `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Orchestrator.ts:474`
  contient `case "thread.stop"`. ThreadLifecycleService reste responsable des
  set-*-mode et n'y touche jamais un enfant en cours (garde FR019 écrite).

### F5 (INFO) — FERMÉ. Domaine de séquence du watch fixé

- `contracts/lineage.md:156-161` : domaine unique = mutation du magasin
  `native_delegation_projection_meta`, partagé par snapshot de liste, show et
  watch ; génération persistée et compatible format 147 ; aucun croisement avec
  le watch humain 147 (magasin de vue de fil) ; « Ready seq0 reste un marqueur
  d'ouverture, pas la dernière séquence du magasin ». `plan.md:90` porte la
  même règle. Précepte code confirmé : `watch_ready_seq` n'accepte que 0
  (`crates/bridget-transport/src/protocol.rs:2227-2232`). Aucun changement de
  schéma requis, comme attendu.

### F6 (INFO) — FERMÉ. Audit des consommateurs d'origine consigné

- T024 (`tasks.md:46`) : avant chaque ajout d'origine, auditer les comparaisons
  directes sur les unions d'origine, branches start, resume, remise de résultat
  et outbox ; une reprise `app_owned` ne prend jamais un fil natif.
- T026 (`tasks.md:48`) : même audit pour `bridget_native`, n'activant aucune
  branche.
- T035 (`tasks.md:57`) : tests couvrant chaque branche consommatrice d'origine
  pour `app_owned`, `provider_native` et `bridget_native`.
- Les trois consignes forment la garde de relecture demandée en r1. Rien à
  changer dans les contrats, comme écrit en r1.

### F7 (LOW) — FERMÉ. Fiche synthèse à jour

- `spec.md:10-12` : « Tâches: 0/45 (0%) », « Tests: 0/28 (0%) », fichiers
  spec/tasks/plan cochés. Cohérent avec les 45 tâches et les 28 scénarios.
  T001 reste propriétaire de la clôture formelle finale.

## Intégrité du périmètre de correction

Horodatages des fichiers : plan.md, spec.md, tasks.md, test-strategy.md et
contracts/lineage.md portent la passe de correction (~08:55-08:56). permissions.md
et data-model.md restent à 08:32, research à 08:17-08:28 : intacts, conformément
au périmètre annoncé. Aucune référence résiduelle à `desktop149.md` dans les
documents actifs (seul le rapport r1 la cite, historique). Toutes les mentions
« desktop » restantes sont des négations explicites portant une règle.

## Réalisabilité G-L — sondes de code

Le code des deux worktrees est inchangé depuis r1 ; les preuves de r1 tiennent.
Sondes refaites cette ronde :

| Point | Sonde | Conclusion |
|---|---|---|
| `thread.stop` côté T3 | `Orchestrator.ts:474` | T029 réalisable au bon endroit |
| Gabarit fixture serveur T3 | `BridgetRustInterop.testkit.ts` présent | T040/S149-28 réalisables sans seconde application |
| `watch_ready_seq` | `protocol.rs:2227-2232` | Contrat 149 réutilise un mécanisme testé |
| Masque colonne de gauche | `Sidebar.logic.ts` : `isSidebarSubagentThread` sur `relationshipToParent === "subagent"` | `plan.md:102` reste vrai sur le code 148 |

Gates partielles : T016–T035 (phases 2 et 3) n'éditent aucun fichier de
permission — aucun `bridgetPermissions.ts`, aucun `mcpSession.ts`, aucune
section `sessionIdentity`. L'export du contrat permission par Sol Lineage est
porté par T004 seule (phase 1, sous G-P). Les chaînes de dépendance
(`tasks.md:77-83`) séquentialisent chaque fichier : orchestrationV2.ts par T024
puis lecture seule ; Orchestrator.ts par T026 puis T028/T029, même owner,
ordre écrit. Aucune course même-fichier détectée. Un changement de droits 148
n'est requis par aucune tâche G-L.

## Axes obligatoires

### Complexité (article XVIII)

Conforme. Pagination O(log N + P) par index root/parent, réconciliation T3 O(n)
avec indexation unique, `descendants_busy` borné (visite 4096). Aucune double
boucle nouvelle. Les annotations de complexité restent à poser dans le code
(T016–T018, T025), comme noté en r1.

### Minimalisme & Frugalité (article XIX)

Conforme. Aucune dépendance nouvelle, aucun journal parallèle, aucun cache de
corps, une seule ligne de projection meta. Les corrections r1 n'ont rien ajouté
de superflu : S149-28 remplace la recette desktop au lieu de s'y ajouter, et les
phrases de garde de T040 portent chacune une règle réelle (périmètre de preuve,
seconde application, relance).

Potentiel minimalisme : ~5 lignes documentaires à comportement constant
(reformulation FR018/FR019 dupliquée entre `plan.md:39` et `tasks.md:109-110`;
la spec reste l'autorité). Aucune ligne de code concernée : aucun code 149
n'existe encore. Non bloquant.

### Vertus LLM & Responsabilité Future (article XX)

Conforme. Le volume documentaire ajouté est borné et chaque phrase de garde est
explicable. Les deux charges futures repérées en r1 (domaine de séquence,
consommateurs d'origine) sont couvertes par écrit (F5, F6). Rappel non
bloquant : aucun scénario n'est déjà vert ; la première exécution de chaque
oracle, y compris S149-28, reste la première preuve (`test-strategy.md:96`).

## Mapping 19 FR / 7 SC / 7 US → 45 tâches, 28 scénarios

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

## Limites de cette revue

Revue documentaire et sondes de lecture seule. Aucun test, aucun build, aucun
modèle ni fournisseur lancé. Aucune fixture exécutée. Aucune mutation de doc,
code, checklist ou tâche hors ce fichier de rapport. Les checklists et tâches
restent non cochées. Le verdict porte sur le volet G-L ; G-P et le contrat
permissions sont hors périmètre de décision. L'APPROVE vaut l'architecture et
les contrats, pas un comportement exécuté : toute preuve reste à produire par
GLM après libération des fichiers.
