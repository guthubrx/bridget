# Revue code adaptateur de bureau Lineage 149 — ronde 2

Date : 2026-10-10. Sous-agent review dédié. Ronde indépendante.
La ronde 1 a échoué avant tout verdict (« Claude API rate limit reached »).
Elle n'a produit aucun verdict ni finding fiable. Rien n'est hérité de r1.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-lineage-code-review-r2.md#extrait-1).

Verdict : **APPROVE**.

Aucune contradiction réelle et aucun bug reproductible n'a été trouvé dans le
périmètre L. Deux findings LOW et six INFO restent. Ils ne violent pas le
contrat `lineage.md`. Leur mode d'échec est sûr. Sol arbitre.

---

## 1. Périmètre et méthode

Périmètre L seulement, comme demandé :

- Contrats : `bridgetLineage.ts`, `rpc.ts`, `index.ts`, `clientRpcPermissions.ts`, `RpcAuthorization.ts`.
- Services serveur : `BridgetReader.ts`, `BridgetLineage.ts`.
- Orchestration serveur : `Orchestrator.ts`, `ProjectionStore.ts`, `ThreadLaunchService.ts`, `runtimeLayer.ts`, `server.ts`, `ws.ts`.
- État client : `orchestration.ts`, `threadExecution.ts`, `subagentRuntime.ts`, `threadWorkflows.ts`, `client.ts`.
- UI web + mobile : `BridgetTaskJournal.tsx` (×2), `ThreadRelationshipsControl.tsx`, `ChatView.tsx`, `ThreadDetailScreen.tsx`.
- MCP serveur : sections `listItemFromShell` et `threadDetail` de `OrchestratorMcpService.ts` uniquement.

Hors périmètre, non lus : sections SessionIdentity/mcpSession/adapters Codex
et Claude (volet P, déjà APPROVE par un autre GLM, durcissement C01/C02 Sol
en cours). Pont Rust natif non lu (Sol GP natif en cours). Aucun autre volet P.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-lineage-code-review-r2.md#extrait-2).

Limites de mission respectées : aucun edit de production, aucun patch, aucun
test lancé, aucun build, aucun lint, aucune commande Git, aucun cochage de
tâche, aucune config de production, aucun restart, aucune opération provider.
Un seul fichier écrit : ce rapport. Aucun oracle modifié.

---

## 2. Findings

Chaque finding donne : sévérité, chemin absolu, ligne, déclencheur
(attendu vs réel), fix minimal pour Sol.

### F-1 (LOW) — Page journal maximale refusée par la borne d'octets du reader

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-lineage-code-review-r2.md#extrait-3).

### F-2 (LOW) — Aucun retrait projeté quand une tâche disparaît du snapshot

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-lineage-code-review-r2.md#extrait-4).

### Findings INFO (aucune action exigée, transcrits pour Sol)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-lineage-code-review-r2.md#extrait-5).

---

## 3. Vérifications positives, par axe d'audit demandé

Chaque point est vérifié dans le code du worktree, avec preuve de ligne.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-lineage-code-review-r2.md#extrait-6).

**Garde read-only atomique, tous transports.** `Orchestrator.ts:10288–10295`
: la garde précède le `switch` des commandes (l.10301). Tout type de commande
passe par elle, y compris MCP et continuations internes (commentaire l.10288).
`fork`/`merge_back` vérifient les deux identifiants. Les fils virtuels refusent
tout sauf `thread.visit` et `thread.stop` →
`OrchestratorSubagentThreadReadOnlyError`. prompt/start/send/resume/fork/
changement de provider/rollback sont couverts par construction, pas par
énumération. `ThreadLaunchService.ts` refuse en plus un launch sur fil virtuel
(défense en profondeur).

**Frontière de confiance du marqueur.** Les commandes internes
(`bridget.lineage.sync`/`unavailable`) sont exclues du payload client :
`dispatchCommand` prend `OrchestrationV2Command`, pas `OrchestrationV2ServerCommand`
(`orchestrationV2.ts:3588–3591`). Un client ne peut forger ni sync ni
ownership. Le marqueur sort par `OrchestratorMcpService.listItemFromShell`/
`threadDetail` en champ optionnel, pour validation stricte côté Rust (T022,
hors revue). Le reader décode **toutes** les sorties daemon avec
`onExcessProperty: "error"` : l'écart « outputs strippés » du rapport
contract-tests r1 §5 est fermé au niveau service.

**Machine à états du snapshot, staging, panne, resync, séquences.**
`BridgetReader.lineageSnapshotPages` : pagination par curseur, page ≤ 100
sinon `snapshot_changed`, égalité generation/seq/root exigée entre pages,
> 4096 → `resource_limit`, cycle de curseur ou page vide avec curseur →
`invalid_output`, snapshot fusionné avec `next_cursor: null`. La machine
watch exige premier événement `ready` seq 0, refuse un `ready` tardif, exige
des seq croissantes dans une génération et un `resync` au changement de
génération. `BridgetLineage.synchronize` resynchronise sur `changed`, marque
`unavailable` sur toute erreur read/watch/journal (panne → « Dernier état
connu », histoire conservée, jamais de fausse disponibilité).

**Preuve d'ownership.** `Orchestrator.ts:10181–10196` : `parent_agent_id`
doit égaler `root_owner_agent_id` (racine) ou le `child_agent_id` du parent
(imbriqué) ; remontée d'ancêtres profondeur ≤ 8 avec ensemble anti-cycle ;
refus `OrchestratorDispatchError` sinon. Snapshot atomique et unique exigé
(l.10179–10180 : doublon ou `next_cursor` non nul → refus). Garde anti
cross-root (l.10209–10210).

**Identité projet sans forge par l'appelant.** `BridgetReader.resolveContext`
: le binding vient du store serveur, pas du client ; le fil de binding doit
exister, non supprimé, et ne pas être lui-même un fil virtuel
(`bridgetTaskRef !== undefined` refusé — aucun emprunt d'autorité).
`lineageArgs` passe `--project-root workspaceRoot` depuis le registre projet
serveur. Le client ne fournit ni chemin ni identité d'autorité ; les inputs
fermés refusent tout champ d'autorité en excès (prouvé par les 22 tests
contrats r1).

**Bornes de ressources, pas de polling caché.** `idleTtlMs: 0` et
`staleTimeMs: 0` sur read/watch/journal (aucun cache résiduel). Retry watch
borné : codes techniques seulement, `< 3` tentatives. Journal client :
cap 4096 événements avec compteur d'éviction, gap explicite collant,
déduplication par seq. UI web : gating sur `visibilitychange` (hidden → abort
+ reprise au dernier seq), abort des requêtes au démontage. Mobile :
`useIsFocused` + `AppState` foreground. Lecture résultat paginée par
offset/`next_offset` (« Copier le résultat chargé » copie le chargé, pas plus).
Aucun polling caché : la watch est une souscription, le journal un suivi
explicite. Pas de N+1 : `getBridgetTaskThreads` batché,
`LIMIT 4097` avec erreur au-delà de 4096.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-lineage-code-review-r2.md#extrait-7).

**État modèle dérivé du natif, sans Run ni ProviderTurn.**
`activeProviderThreadId: null`, `runId: null`, `prompt: ""`, `result: null`
sur les fils projetés (`Orchestrator.ts:10216–10237`) ; le contenu de résultat
ne passe que par `show`. Statut shell : `nativeShellStatus`
(`ProjectionStore.ts`, `pending` → `preparing`, valeurs toutes dans
`OrchestrationV2ShellThreadStatus`, vérifié `orchestrationV2.ts:461–472` et
`:1821`). `deriveProviderSubagentStatus` : `bridgetTask` → statut/dates natifs,
aucune run. `subagent.updated` : statut = `bridgetTaskStatus(task.status)`,
les 6 valeurs de présentation sont dans l'union du schéma
(`orchestrationV2.ts:692–701`). Effort depuis les options du modelSelection
synthétique `bridget:${agent_type}`. Erreur native remontée dans `progress`.
MCP `threadDetail` : statut natif-dérivé pour les fils à marqueur.

**Préservation du monde existant.** Champs `bridgetTaskRef`/`bridgetTask`/
`bridgetLineage` optionnels partout ; aucun fil ordinaire 148 converti.
Masque sidebar conservé via `lineage.relationshipToParent: "subagent"`.
Branches `app_owned`/`provider_native` intactes. Scopes : read/watch/journal
→ `AuthOrchestrationReadScope`, cancel → `AuthOrchestrationOperateScope` via
le spread `CLIENT_GUARDED_RPC_SCOPES` (source unique, `clientRpcPermissions.ts`
+ `RpcAuthorization.ts`). Cycles de services : `layerBridgetReader` fourni au
niveau serveur ; l'Orchestrator lit `BridgetReader` et `Crypto` en
`serviceOption` → échec fermé si absent, aucun cycle. `ChildProcessSpawner`
est une dépendance existante du reader (`BridgetReader.ts:84`, usages 238/458),
cohérente avec le lecteur bridget 148.

---

## 4. Complexité, minimalisme, responsabilité future

- Complexité maîtrisée. La garde read-only est une seule porte avant le
  switch, pas une liste de refus par transport. La réconciliation sync est
  linéaire, annotée. Les bornes sont concentrées dans le reader.
- Minimalisme correct. Aucun framework nouveau, aucun modèle ajouté. Les
  champs optionnels évitent toute migration des fils 148. La UI réutilise les
  primitives existantes (atoms, command, streams).
- Responsabilité future : deux points à trancher par Sol (F-1, F-2), les deux
  documentables en quelques lignes de fix ou d'invariant de contrat. Le
  mapping des statuts et les bornes sont centralisés, donc maintenables.

---

## 5. Limites de cette revue

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-lineage-code-review-r2.md#extrait-8).

---

## 6. Verdict

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-lineage-code-review-r2.md#extrait-9).

Le code tient le contrat `lineage.md` : projection seule, identités
déterministes, inputs fermés, sorties daemon décodées strictement, ownership
prouvé, bornes de ressources réelles, annulation routée sans double stop,
état modèle dérivé du natif sans Run ni ProviderTurn, monde 148 intact.

Les findings F-1 et F-2 sont LOW, à échec sûr, à arbitrer par Sol. Aucun
blocant. Aucune correction exigée pour approuver.
