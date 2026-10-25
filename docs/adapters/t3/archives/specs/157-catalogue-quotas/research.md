# Archives techniques de l’adaptateur T3

Document d’origine : [specs/157-catalogue-quotas/research.md](../../../../../../specs/157-catalogue-quotas/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-11. Sources : lecture du checkout principal `1f2c661d` (avant la
session 150), et de T3 `origin/main` par `git show`. Aucun appel à un
fournisseur. Les lignes citées sont celles de cette base. La session 150 les
déplace.

<a id="extrait-2"></a>

## Extrait 2

- **Constat** : le catalogue calcule `discovery` = `for_spawn_posture(Discovery).is_ok()`
  ET (`permission.is_some()` OU `fact.is_some()`)
  (`daemon/native_delegation.rs:381`, `:388`). Il n'appelle pas `child_policy`
  pour cette posture. Il l'appelle seulement pour `inherit` (posture absente) et
  `development` (`:382-385`).
- **Admission** : si un fait attesté existe, elle calcule
  `child_policy(fait, cible, cwd, env, posture)` puis
  `for_inherited_delegation` (`:435-438`). Pour (parent `claude_stream_json`,
  cible `codex_app_server`), `child_policy` fait
  `policy.get("launch_context").ok_or("permission_source_unavailable")`
  (`native_permissions.rs:253-254`), quelle que soit la posture.
- **Pourquoi le fait T3 échoue** : `launch_context` est une clé optionnelle d'une
  politique Claude (`bridget-transport/src/native_permissions.rs:75`). Le daemon
  la remplit pour les agents lancés par Bridget
  (`native_permission_observer.rs:85`). On n'a pas la preuve qu'un fil T3 la
  fournit. C'est l'hypothèse la plus simple et elle explique le constat.
- **Deux causes possibles pour le même code** (contre-revue 157, objection 7) :
  (a) `launch_context` absent du fait Claude (`:254`) ; (b) `launch_context`
  présent mais `claude_inputs_without_rules` appelle `read_json_source` sur une
  source illisible lors du passage Claude vers Codex. Le refus est
  `permission_source_unavailable`. Une source contenant des règles peut aussi
  rendre ce code. La relecture `recheck_context` de Claude vers Claude produit
  `settings_revision_changed` si la source change ou devient illisible. Ce
  dernier code reste distinct et non remplaçable.
- **Conclusion** : dans les deux cas, le catalogue ne passe pas par la même
  fonction que l'admission. `inherit_refusal` contenait déjà le bon code. La
  correction est un verdict unique (R2), pas un patch de `discovery`. Le
  diagnostic conserve la distinction : `constraints[].message` cite la cause
  (« fait sans launch_context » ou « source de permissions illisible »).
- **Statut de la preuve** : scénario de code, non confirmé sur l'incident réel.
  T007 écrit un test rouge par cause. T002 cherche un fait réel expurgé (clés
  présentes seulement, aucune valeur) pour dire laquelle s'est produite. Sans
  fait réel, aucun document ne dit « cause confirmée ».

<a id="extrait-3"></a>

## Extrait 3

- **T3** : `providerConstraints` est appelé par le catalogue
  (`OrchestratorMcpService.ts:1782-1800`, `canRunChildTask`, `constraints`) et
  par la délégation (`:1146-1153`). Aucune dérive possible.
- **Décision** : extraire de l'admission (`:431-440`) une fonction
  `resolve_target(st, owner, instance, fact, permission, agent_type, posture, ...)`
  qui rend la cible résolue (définition épinglée, instantané de droits, posture
  effective) ou un code de refus. L'admission et le catalogue l'appellent.
- **Limite de la garantie** (contre-revue 157, objection 4) : une fonction
  commune ne suffit pas. Le fait attesté dépend de `request_id` :
  le catalogue appelle `parent_fact(..., None, ...)` (`:377`), la délégation
  `parent_fact(..., Some(request_id), ...)` (`:420`), et les faits limités à une
  demande sont filtrés par `scope == request_id` (`:56`). Le `cwd` change aussi
  la politique : le catalogue utilise `fact.cwd`, la délégation le `cwd` demandé
  (`native_permissions.rs:230` exige l'égalité pour Claude vers Claude). Le
  binaire peut disparaître entre deux appels. La garantie est donc bornée :
  mêmes entrées, mêmes faits non limités à une demande, `cwd` du
  `verdict_context`, état inchangé. Le catalogue déclare ce contexte. L'admission
  reste l'autorité.
- **Alternative rejetée** : corriger `discovery` seul. Elle laisse la même
  classe de dérive pour le binaire et le quota.

<a id="extrait-4"></a>

## Extrait 4

- **Décision** (Article XIX, convergence après deux rondes de contre-revue) :
  la reprise sur quota n'est pas dans la 157. Elle devient la session future
  160. Les exigences d'entrée sont dans `spec.md`, section « Session future
  160 ».
- **Pourquoi** : la ronde 1 a montré que 152 ne fournit pas la relance d'une
  tâche `failed` (152 `spec.md` FR-017, `data-model.md` I1 à I4). La conception
  corrigée de la 157 a ensuite produit six défauts, tous dans la reprise :
  compteur de rondes perdu, reconstruction d'une exécution `failed` exclue par
  les requêtes de reprise (`execution_store.rs:1325`, `:1393`), identité de
  relance non vérifiée, borne de pannes de 152 consommée, double comptage de
  `suspended_total_secs`, et suppression du suivi avec notification du parent
  après `rejected()` (`daemon.rs:18095`). Chacun exige une décision commune
  avec 152.
- **Ce que la 157 garde** : la mémoire de limites par fenêtre (R4) et le refus à
  l'admission. Aucune partie de `fail_or_wait` n'est nécessaire au
  remplacement : le remplacement ne joue qu'à l'admission (R6), jamais après un
  échec de mission. `fail_or_wait` est donc retirée.
- **Reste de la recherche utile à la 160** : le modèle T3
  (`limitRecoveryCommand` : classe `usage_limit`, garde « heure de retour
  postérieure à la fin du run », identité `thread:run:resetMs` dérivée de faits
  persistés) ; les chemins d'échec d'une mission (`wrapper.rs:6218-6262` puis
  `rejected()`, état d'exécution `failed` lu dans le tick, refus de remise de
  mission) ; l'ordre d'arrivée non garanti entre fait de quota et échec.
