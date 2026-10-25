# Recherche - Session157 catalogue, quotas, remplacement

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/157-catalogue-quotas/research.md#extrait-1).

## R1 - Pourquoi `discovery: true` et `permission_source_unavailable` coexistent

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/157-catalogue-quotas/research.md#extrait-2).

## R2 - Un seul verdict, par le contrat natif

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/157-catalogue-quotas/research.md#extrait-3).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/157-catalogue-quotas/research.md#extrait-4).

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
