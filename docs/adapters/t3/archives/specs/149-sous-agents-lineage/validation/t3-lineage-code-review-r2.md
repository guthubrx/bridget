# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-lineage-code-review-r2.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-lineage-code-review-r2.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Base revue : `33f6d04e116430bf7f0011902d6af3868163f2d1` (livraison 148).
Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`.

<a id="extrait-2"></a>

## Extrait 2

Méthode : lecture ciblée des fichiers et diffs du périmètre. Contrat
`contracts/lineage.md` v1 comme source de vérité. Références croisées :
`plan.md` (G-L r3 APPROVE, corrections F1–F7), `tasks.md`, `test-strategy.md`,
`validation/t3-lineage-contract-tests-r1.md`, `validation/plan-lineage-r1.md`
et `plan-lineage-r3.md`. Aucune exploration générale redondante.

<a id="extrait-3"></a>

## Extrait 3

- Chemin : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bridget/BridgetReader.ts`
- Lignes : 433 (branche journal de `lineageRead` : `maxBytes = 16 * 1024`), 380 (`maxOutputBytes = maxBytes + 1024`), 455–480 et 506 (suivi `--follow` : `maxLine = 16 * 1024`).
- Déclencheur. Contrat `lineage.md` l.137 : « Une lecture journal rend au plus 100 événements et 16 KiB de contenu ». La borne de 16 KiB porte sur le **contenu** décodé. Attendu : toute page valide au contrat passe le transport du reader. Réel : la borne du reader porte sur les **octets NDJSON encodés**. Une page de 100 événements dont le contenu total approche 16 KiB dépasse 16 KiB une fois encodée (clés, guillemets, échappements). Le stdout est alors tronqué ou la ligne scindée → refus `invalid_output` ou `command_failed`. La même page passe côté daemon.
- Preuve de l'incohérence interne : la branche `show` alloue 24 KiB pour une fenêtre de 16 KiB (l.433), soit une marge ×1,5. La branche journal alloue 16 KiB, soit une marge ×1,0.
- Gravité : LOW. L'échec est sûr : sortie tronquée refusée, aucune donnée fausse admise, la UI conserve le contenu déjà lu. Impact = disponibilité sur les pages maximales seulement.
- Fix minimal pour Sol : donner à la branche journal la même marge que `show` (par ex. 24 KiB) à BridgetReader.ts:433, et porter `maxLine` du suivi (l.506) à la même valeur. Confirmer côté daemon que la borne de page est appliquée sur le contenu, pas sur l'encodage.

<a id="extrait-4"></a>

## Extrait 4

- Chemin : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Orchestrator.ts`
- Lignes : 10200–10202 (`previousThreads` inclut tous les fils projetés du root, l.10199–10200) puis 10204–10251 : la boucle de réconciliation n'itère que sur `tasks.values()` (tâches présentes). Aucun passage sur les fils projetés absents du snapshot.
- Déclencheur. Attendu : la projection reflète la liste native. Réel : si le daemon retire une tâche de `lineage list` (politique de rétention, nettoyage), le fil virtuel T3 reste projeté avec son dernier statut et `available: true`, indéfiniment. Les resynchronisations suivantes ne le corrigent pas. Seul `bridget.lineage.unavailable` (erreur de transport) bascule `available: false`, et il parle de transport, pas de retrait par tâche.
- Gravité : LOW. Effet de présentation seul. Aucune exécution T3. Le contrat `lineage.md` n'exige pas de retrait par tâche et la politique de rétention du daemon appartient au volet Sol natif (hors revue). Le refus de l'opération sur une tâche disparue reste honnête côté daemon.
- Fix minimal pour Sol, au choix : (a) après la boucle des tâches, émettre pour chaque `previousThreads` absent de `tasks` un `thread.metadata-updated` avec `bridgetLineage: { ...available: false }` ; ou (b) documenter dans `lineage.md` l'invariant de rétention « une tâche listée reste listée » et laisser le code tel quel. Ne pas inventer un troisième mécanisme.

<a id="extrait-5"></a>

## Extrait 5

| # | Objet | Chemin + ligne | Constat |
|---|---|---|---|
| I-1 | Course snapshot_changed en watch | `BridgetReader.ts` l.455+ (machine à états) | Trois `snapshot_changed` en course terminent le flux. Permission du contrat, resync au reconnect, retry client borné à 3. Comportement acceptable. |
| I-2 | Page > 100 entrées → `snapshot_changed` | `BridgetReader.ts` `lineageSnapshotPages` | Code sémantiquement approximatif pour une violation de borne, mais voie fermée et sûre (relance la synchronisation). |
| I-3 | Bornes schéma en longueur UTF-16 vs octets | `packages/contracts/src/bridgetLineage.ts` (curseurs, résultat) | Le reader revérifie les octets pour `show` (l.438). Décodage tolérant en longueur sur la sortie d'un daemon de confiance, fermeture réelle au niveau service. Acceptable ; rester cohérent si durcissement demandé. |
| I-4 | Cancel : `requestId` neuf à chaque tentative UI | `apps/web/src/components/BridgetTaskJournal.tsx` l.159 ; mobile idem | Pas de clé d'idempotence entre tentatives T3. L'annulation native reste idempotente par tâche. Le contrat exige `request_id` ; l'UUID local couvre l'action humaine. Acceptable. |
| I-5 | Code mort `output.timedOut` | `BridgetReader.ts` `runLineage` | `ProcessRunner` échoue en `ProcessTimeoutError` avant retour, donc la branche n'est jamais atteinte. Inoffensif. Nettoyage cosmétique possible. |
| I-6 | Stop d'un root avec enfants natifs actifs et Bridget injoignable | `Orchestrator.ts` `thread.stop` (chemin root) | Le stop échoue entièrement si le snapshot lignage est indisponible. Refus honnête exigé par le contrat (aucun fallback silencieux). Positif, pas un défaut ; noté pour mémoire. |

<a id="extrait-6"></a>

## Extrait 6

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

<a id="extrait-7"></a>

## Extrait 7

**Routage d'annulation de l'arbre.** `thread.stop` sur fil virtuel →
`cancelNative` unique puis retour sans stop provider T3. Stop d'un root :
vérification d'enfants natifs actifs, annulation des sous-arbres actifs au
niveau du plus haut ancêtre actif (commentaire l.~: « A failed ancestor can
still have an independently active descendant »), profondeur < 8. Le moteur
natif cascade les descendants. Pas de double stop avec
`ThreadManagementService.stopDelegatedTasks` (saute `bridget_native`).
Transport indisponible + enfants actifs → refus honnête, aucun fallback (I-6).

<a id="extrait-8"></a>

## Extrait 8

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

<a id="extrait-9"></a>

## Extrait 9

**APPROVE** pour le périmètre L T3 Lineage 149 à la base `33f6d04e`.
