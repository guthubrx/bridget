# Recherche - Session157 catalogue, quotas, remplacement

Date : 2026-10-11. Sources : lecture du checkout principal `1f2c661d` (avant la
session 150), et de T3 `origin/main` par `git show`. Aucun appel à un
fournisseur. Les lignes citées sont celles de cette base. La session 150 les
déplace.

## R1 - Pourquoi `discovery: true` et `permission_source_unavailable` coexistent

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

## R2 - Un seul verdict, comme T3

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

## R3 - Disponibilité réelle : le préflight est réparti en deux endroits

- `submit_spawn` (`lifecycle.rs:209-226`) vérifie capacités et observation
  fournisseur, puis le binaire. Il ignore l'erreur de `build_environment`
  (`if let Ok(env) = ...`, `:221`). La garde de facturation
  (`forbidden_environment_variable` donnant `BillingGuard`), le contrôle de
  protocole géré (`NegotiationFailed`) et la propagation de `EnvUnfit` sont dans
  `prepare_spawn_parts` (`:342-380`, `:359-365`), appelée plus tard.
  Contre-revue 157, objection 3 : retenue et vérifiée.
- **Décision** : `launch_preflight(registry_épinglé, agent_type, source)` regroupe,
  dans l'ordre : type connu, capacités, observation fournisseur, garde de
  facturation, protocole géré, `build_environment` (erreur propagée),
  `command_exists`. `submit_spawn` et `prepare_spawn_parts` l'appellent, donc il
  n'existe plus deux versions. Il s'exécute sur la définition épinglée de la
  tâche (celle de `resolved_definition`), pas sur le registre global : les
  arguments épinglés peuvent changer l'issue. Le contrôle `cwd.is_dir()` en est
  exclu (contrôle du parent).
- Les codes réutilisent `refusal_record` (`lifecycle.rs:561`) : `command_missing`,
  `billing_guard`, `env_unfit`, `unsupported_capability`, `negotiation_failed`.
- **Authentification** : aucune sonde n'existe. Décision : non vérifiée, et dite
  (`checks.auth: false`). Question 3 de la spec.
- **Effet de bord voulu** : un binaire absent, une clé API interdite ou un
  environnement invalide sont refusés à l'admission au lieu de créer une tâche
  qui échoue.
- **Cas de test exigés** : facturation interdite avec binaire présent ;
  environnement invalide (HOME absent) avec binaire présent ; protocole non
  géré ; chaque cas égal entre catalogue et admission.

## R4 - Quotas : où vivent les faits, et pourquoi il faut une mémoire

- **Sources** : flux Claude (`rate_limit_event`, statut `allowed`,
  `allowed_warning`, `rejected`, fenêtre `five_hour` ou `seven_day`,
  `claude_stream_json.rs:1355-1367`) ; Codex app-server (`primary/300m`,
  `secondary/10080m`, statut `available` ou `rateLimitReachedType`,
  `codex_app_server.rs:2643-2686`) ; hook `claude-statusline` (statut fixe
  `unknown`, pourcentage et heure de retour, `cli.rs:4967`, `:5115`). Les trois
  arrivent par `handle_rate_limit` (`daemon.rs:6600-6660`).
- **Stockage actuel** : `Presence.rate_limits` (`daemon.rs:566`), par fenêtre,
  par présence. Pas d'instant d'observation. Perdu avec la présence.
- **Problème** : quand un enfant échoue sur un quota, il se déconnecte. Le fait
  part avec lui. Sans mémoire par fournisseur, le catalogue oublie au moment
  où il serait utile.
- **Décision** : un module `provider_limits.rs` (map `agent_type -> fenêtre ->
  {fait, observed_at}`), écrite par `handle_rate_limit` en plus de la présence.
  Lecture O(log n). Volatile (pas de table SQL) : le hook statusline republie à
  chaque tour et Codex republie au démarrage, ce qui répare la mémoire après un
  redémarrage. Le module a 3 usages réels (écriture, catalogue, admission).
- **Règle d'épuisement** (FR-008, corrigée après la contre-revue 157,
  objection 5) : le statut Codex est global. `rateLimitReachedType` est un champ
  du snapshot, recopié sur `primary` et `secondary`
  (`codex_app_server.rs:2639-2665`). Le lire par fenêtre ferait attendre une
  semaine pour une limite de 5 heures. Une fenêtre est épuisée seulement si :
  heure de retour future ET (statut `rejected` d'une source qui l'attache à sa
  fenêtre, c'est-à-dire le flux Claude `stream-json`, OU pourcentage >= 100).
  La mémoire garde la source de chaque fait (`handle_rate_limit` reçoit
  `source`). Un statut Codex non `available` sans fenêtre à 100 % lève
  `limit_reached` mais jamais `exhausted`. Le daemon n'invente pas la fenêtre
  responsable. Test exigé : statut global atteint + fenêtre courte à 100 % +
  fenêtre hebdomadaire à 61 % : seule la courte est épuisée, son retour est
  celui de la courte.
- **Clé** : `Presence.agent_type` (`daemon.rs:524`) égal au nom du registre.
  Hypothèse à tester (T004). Limite : deux fournisseurs sur le même compte ne
  partagent pas leur quota.
- **Confidentialité** : le catalogue ne rend jamais de nom de présence, seulement
  des fenêtres agrégées par fournisseur.

## R5 - Rendre un refus avec détail

- `failure(code)` rend `{"status":"refused","code":...}` (`:34`). Le pont MCP
  garde seulement `result["code"]` (`delegation_mcp.rs:86-92`). Le détail
  (heure de retour, essais) serait perdu.
- **Décision révisée** : garder Err(String) enrichi code:JSON (message/detail).
  Le failure existant décode ce porteur ; le catalogue lit le diagnostic sans
  recalculer les droits, le pont rend code puis detail. Les listes utilisent
  le code canonique. Aucun type/helper supplémentaire. Codes nus, brief et
  workspace compatibles ; quota/fallback restent hors record_refusal durable.
  Les tests de contrat T016/T017 déjà prévus portent ces formes.

## R6 - Remplacement

- **Règle d'engagement** : cible demandée refusée pour un code de la liste fermée
  « indisponibilité » (FR-013). Tout autre refus de la cible demandée est rendu
  tel quel (`model_unavailable`, droits du parent, `cwd_*`).
- **Candidats suivants** : tout refus d'un candidat explicitement listé fait passer
  au suivant, et est noté dans `tried`.
- **Où** : dans l'admission, à la place de l'appel unique à `resolve_target`,
  après la garde de pause et les gardes de droits du parent (les contrôles du
  parent ne dépendent pas de la cible). Au plus 4 appels : O(1).
- **Persistance** : `canonical` est stocké à part du `request` de la tâche
  (`delegation.rs:114-130`). On peut donc écrire dans `record.request` la cible
  effective (`agent_type`, `model`, `effort`, `fallback` vidé). `task_view`,
  `spawn_task` et la projection Lineage (`delegation_lineage.rs:32`) lisent la
  cible effective sans changement. La demande d'origine est dans
  `substitution.requested`.
- **Rejeu** : `by_agent_request` rend la tâche enregistrée si l'enveloppe
  (avec `fallback`) est identique. Aucun nouveau calcul.
- **Rejeté** : bascule automatique sans paramètre (contrat 148), bascule en
  cours de mission (question 2), liste de remplacement stockée dans le registre
  (cache un choix du parent).

## R7 - Reprise sur limite d'usage : reportée en 160

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

## R8 - Coût du catalogue (Article XVIII et XXII)

- `child_policy` pour (Codex vers Claude) appelle `capture_context`, qui hache
  l'exécutable (jusqu'à 512 Mio, `native_permissions.rs:79`, `executable_revision`).
  Le catalogue l'appelle déjà 2 fois par fournisseur. Le verdict `discovery` en
  ajoute une troisième.
- **Décision** : mesurer avant (T001) et après (T012) sur 10 fournisseurs de test.
  Si le rapport dépasse 2 (SC-007), calculer chaque politique une seule fois par
  fournisseur et dériver les trois postures d'un même contexte capturé. Pas
  d'optimisation avant la mesure.

## R9 - Hors périmètre constaté

- Persistance des quotas après redémarrage (dette notée).
- Quota partagé entre fournisseurs d'un même compte.
- Attente d'un quota futur à l'admission (une tâche admise puis suspendue
  avant de démarrer), et reprise sur quota (160).

## R10 - Ordre des contrôles d'un candidat (contre-revue 157, objection 2)

- `for_inherited_delegation` vérifie `model_unavailable` et `effort_unavailable`
  (`registry.rs:337-341`) APRÈS `child_policy` dans l'admission actuelle
  (`native_delegation.rs:435-438`). Si le préflight ou la politique viennent
  avant, un modèle absent plus un binaire absent rend `command_missing`, un
  motif remplaçable : le remplacement masque une erreur d'appelant que le
  contrat 148 interdit de remplacer.
- **Décision** : ordre fixe par candidat : type connu (`registry.get`), modèle,
  effort, puis préflight, politique, quota. Tests : modèle absent + binaire
  absent, effort invalide + quota épuisé, modèle absent + politique indisponible
  : zéro substitution, code `model_unavailable` ou `effort_unavailable`.
- Pour un candidat de `fallback`, tout refus (y compris modèle ou effort) fait
  passer au suivant : le parent a listé ce candidat, et `tried` rend le code.
