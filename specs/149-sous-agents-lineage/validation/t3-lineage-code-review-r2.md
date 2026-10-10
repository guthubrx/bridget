# Revue code T3 Lineage 149 — ronde 2

Date : 2026-10-10. Sous-agent review dédié. Ronde indépendante.
La ronde 1 a échoué avant tout verdict (« Claude API rate limit reached »).
Elle n'a produit aucun verdict ni finding fiable. Rien n'est hérité de r1.

Base revue : `33f6d04e116430bf7f0011902d6af3868163f2d1` (livraison 148).
Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`.

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

Méthode : lecture ciblée des fichiers et diffs du périmètre. Contrat
`contracts/lineage.md` v1 comme source de vérité. Références croisées :
`plan.md` (G-L r3 APPROVE, corrections F1–F7), `tasks.md`, `test-strategy.md`,
`validation/t3-lineage-contract-tests-r1.md`, `validation/plan-lineage-r1.md`
et `plan-lineage-r3.md`. Aucune exploration générale redondante.

Limites de mission respectées : aucun edit de production, aucun patch, aucun
test lancé, aucun build, aucun lint, aucune commande Git, aucun cochage de
tâche, aucune config de production, aucun restart, aucune opération provider.
Un seul fichier écrit : ce rapport. Aucun oracle modifié.

---

## 2. Findings

Chaque finding donne : sévérité, chemin absolu, ligne, déclencheur
(attendu vs réel), fix minimal pour Sol.

### F-1 (LOW) — Page journal maximale refusée par la borne d'octets du reader

- Chemin : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bridget/BridgetReader.ts`
- Lignes : 433 (branche journal de `lineageRead` : `maxBytes = 16 * 1024`), 380 (`maxOutputBytes = maxBytes + 1024`), 455–480 et 506 (suivi `--follow` : `maxLine = 16 * 1024`).
- Déclencheur. Contrat `lineage.md` l.137 : « Une lecture journal rend au plus 100 événements et 16 KiB de contenu ». La borne de 16 KiB porte sur le **contenu** décodé. Attendu : toute page valide au contrat passe le transport du reader. Réel : la borne du reader porte sur les **octets NDJSON encodés**. Une page de 100 événements dont le contenu total approche 16 KiB dépasse 16 KiB une fois encodée (clés, guillemets, échappements). Le stdout est alors tronqué ou la ligne scindée → refus `invalid_output` ou `command_failed`. La même page passe côté daemon.
- Preuve de l'incohérence interne : la branche `show` alloue 24 KiB pour une fenêtre de 16 KiB (l.433), soit une marge ×1,5. La branche journal alloue 16 KiB, soit une marge ×1,0.
- Gravité : LOW. L'échec est sûr : sortie tronquée refusée, aucune donnée fausse admise, la UI conserve le contenu déjà lu. Impact = disponibilité sur les pages maximales seulement.
- Fix minimal pour Sol : donner à la branche journal la même marge que `show` (par ex. 24 KiB) à BridgetReader.ts:433, et porter `maxLine` du suivi (l.506) à la même valeur. Confirmer côté daemon que la borne de page est appliquée sur le contenu, pas sur l'encodage.

### F-2 (LOW) — Aucun retrait projeté quand une tâche disparaît du snapshot

- Chemin : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Orchestrator.ts`
- Lignes : 10200–10202 (`previousThreads` inclut tous les fils projetés du root, l.10199–10200) puis 10204–10251 : la boucle de réconciliation n'itère que sur `tasks.values()` (tâches présentes). Aucun passage sur les fils projetés absents du snapshot.
- Déclencheur. Attendu : la projection reflète la liste native. Réel : si le daemon retire une tâche de `lineage list` (politique de rétention, nettoyage), le fil virtuel T3 reste projeté avec son dernier statut et `available: true`, indéfiniment. Les resynchronisations suivantes ne le corrigent pas. Seul `bridget.lineage.unavailable` (erreur de transport) bascule `available: false`, et il parle de transport, pas de retrait par tâche.
- Gravité : LOW. Effet de présentation seul. Aucune exécution T3. Le contrat `lineage.md` n'exige pas de retrait par tâche et la politique de rétention du daemon appartient au volet Sol natif (hors revue). Le refus de l'opération sur une tâche disparue reste honnête côté daemon.
- Fix minimal pour Sol, au choix : (a) après la boucle des tâches, émettre pour chaque `previousThreads` absent de `tasks` un `thread.metadata-updated` avec `bridgetLineage: { ...available: false }` ; ou (b) documenter dans `lineage.md` l'invariant de rétention « une tâche listée reste listée » et laisser le code tel quel. Ne pas inventer un troisième mécanisme.

### Findings INFO (aucune action exigée, transcrits pour Sol)

| # | Objet | Chemin + ligne | Constat |
|---|---|---|---|
| I-1 | Course snapshot_changed en watch | `BridgetReader.ts` l.455+ (machine à états) | Trois `snapshot_changed` en course terminent le flux. Permission du contrat, resync au reconnect, retry client borné à 3. Comportement acceptable. |
| I-2 | Page > 100 entrées → `snapshot_changed` | `BridgetReader.ts` `lineageSnapshotPages` | Code sémantiquement approximatif pour une violation de borne, mais voie fermée et sûre (relance la synchronisation). |
| I-3 | Bornes schéma en longueur UTF-16 vs octets | `packages/contracts/src/bridgetLineage.ts` (curseurs, résultat) | Le reader revérifie les octets pour `show` (l.438). Décodage tolérant en longueur sur la sortie d'un daemon de confiance, fermeture réelle au niveau service. Acceptable ; rester cohérent si durcissement demandé. |
| I-4 | Cancel : `requestId` neuf à chaque tentative UI | `apps/web/src/components/BridgetTaskJournal.tsx` l.159 ; mobile idem | Pas de clé d'idempotence entre tentatives T3. L'annulation native reste idempotente par tâche. Le contrat exige `request_id` ; l'UUID local couvre l'action humaine. Acceptable. |
| I-5 | Code mort `output.timedOut` | `BridgetReader.ts` `runLineage` | `ProcessRunner` échoue en `ProcessTimeoutError` avant retour, donc la branche n'est jamais atteinte. Inoffensif. Nettoyage cosmétique possible. |
| I-6 | Stop d'un root avec enfants natifs actifs et Bridget injoignable | `Orchestrator.ts` `thread.stop` (chemin root) | Le stop échoue entièrement si le snapshot lignage est indisponible. Refus honnête exigé par le contrat (aucun fallback silencieux). Positif, pas un défaut ; noté pour mémoire. |

---

## 3. Vérifications positives, par axe d'audit demandé

Chaque point est vérifié dans le code du worktree, avec preuve de ligne.

**Origines dans les dispatchs indirects.** Les consommateurs filtrent les
origines de façon positive. `Orchestrator.ts:9961` : livraison de complétion
uniquement `=== "provider_native"` ; les tâches `bridget_native` sautent la
branche, aucun résultat n'est réacheminé au parent T3.
`ThreadManagementService.ts:850` : `stopDelegatedTasks` saute `bridget_native`,
pas de double stop avec le routage `thread.stop`.
`ProviderEventIngestor.ts:358`, `ProviderRuntimeRecoveryService.ts:107`,
`ThreadDeletion.ts:134/143`, `OrchestratorMcpService.ts:408/1230/1921/2014/2021`
: branches `app_owned` uniquement. Aucun filtre négatif résiduel du type
`!== "app_owned"` qui engloutirait `bridget_native`. Le point F6 du plan r1
est bien corrigé (`Orchestrator.ts:8322` : `=== "provider_native"`).

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

**Routage d'annulation de l'arbre.** `thread.stop` sur fil virtuel →
`cancelNative` unique puis retour sans stop provider T3. Stop d'un root :
vérification d'enfants natifs actifs, annulation des sous-arbres actifs au
niveau du plus haut ancêtre actif (commentaire l.~: « A failed ancestor can
still have an independently active descendant »), profondeur < 8. Le moteur
natif cascade les descendants. Pas de double stop avec
`ThreadManagementService.stopDelegatedTasks` (saute `bridget_native`).
Transport indisponible + enfants actifs → refus honnête, aucun fallback (I-6).

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

1. Aucun test lancé, aucun build, aucun lint, aucun typecheck. Les preuves
   d'exécution existantes appartiennent à d'autres : 22/22 PASS contrats
   lineage + 47/47 PASS régression bridget 148
   (`validation/t3-lineage-contract-tests-r1.md`). Les tests serveur/client/UI
   et permissions sont en cours par d'autres agents ; cette revue n'exige pas
   qu'ils soient verts et n'a pas doublé leurs tests.
2. Pont Rust natif non lu (Sol GP natif en cours). La preuve T3 seule ne
   prouve pas le comportement autonome du daemon ; c'est hors revue par
   mission.
3. Sections SessionIdentity/mcpSession/adapters Codex et Claude non lues
   (volet P, APPROVE ailleurs).
4. F-2 dépend de la politique de rétention du daemon, non encore écrite
   (volet natif). Le finding reste ouvert jusqu'à cet arbitrage.
5. Aucun oracle modifié. Aucun cochage. Aucun commit. Une seule écriture :
   ce rapport.

---

## 6. Verdict

**APPROVE** pour le périmètre L T3 Lineage 149 à la base `33f6d04e`.

Le code tient le contrat `lineage.md` : projection seule, identités
déterministes, inputs fermés, sorties daemon décodées strictement, ownership
prouvé, bornes de ressources réelles, annulation routée sans double stop,
état modèle dérivé du natif sans Run ni ProviderTurn, monde 148 intact.

Les findings F-1 et F-2 sont LOW, à échec sûr, à arbitrer par Sol. Aucun
blocant. Aucune correction exigée pour approuver.
