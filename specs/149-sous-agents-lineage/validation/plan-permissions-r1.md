# Revue G-P r1 — plan permissions, attestation et héritage

Date : 2026-10-10. Revueuse : GLM 5.3 Flash, indépendante.
Périmètre : `contracts/permissions.md`, `plan.md`, `data-model.md`, `test-strategy.md`,
`tasks.md`, `spec.md`, `research-native.md`, `research-t3.md`, ADR149.
Verdict demandé : APPROVE ou REQUEST_CHANGES sur le volet G-P.

**Verdict : REQUEST_CHANGES.** Le contrat est solide et fidèle au wire réel.
Deux exigences contractuelles n'ont aucun oracle de test. Deux preuves
d'environnement manquent. Aucun blocage d'architecture. Les corrections sont
documentaires et de couverture, pas de conception.

Ronde r1. Aucun test exécuté, aucun build, aucune recette réelle. Aucun code ni
document de contrat modifié. Ce rapport est le seul fichier écrit par cette revue.
Aucune tâche n'est cochée par cette revue. Aucun compteur de relecture n'est inventé.

## 1. Preuves vérifiées à la source

Lecture seule. Les valeurs de secrets ne sont jamais imprimées.

| Fait vérifié | Source précise | Résultat |
|---|---|---|
| Parent externe borné à `Discovery` | `crates/bridget-daemon/src/daemon/native_delegation.rs:8-30` | Confirmé : `SameProject` → maximum `Discovery`. |
| Héritage development 148 par signature Codex exacte | `native_delegation.rs:70-86` | Confirmé : workspace-write sans réseau, sans racines, tmp exclus. |
| Refus `development_grant_required` pour un parent lecteur | `native_delegation.rs:205-210` | Confirmé. Code 148 à remplacer en 149, comme prévu. |
| Development réservé à `codex_app_server` | `crates/bridget-daemon/src/registry.rs:192-216` | Confirmé. |
| `permissions=allow` pose un bypass global | `crates/bridget-daemon/src/wrapper.rs:4787-4795` | Confirmé. La garde 149 « jamais bypass universel » vise le bon code. |
| `bridget_session` v1 atteste l'identité seulement, parseur fermé | `crates/bridget-daemon/src/t3code_mcp.rs:12-20`, `apps/server/src/mcp/OrchestratorMcpService.ts:1776-1789` | Confirmé : `version:1`, 4 champs camelCase, `deny_unknown_fields`. |
| `posture` obligatoire aujourd'hui | `crates/bridget-daemon/src/delegation_mcp.rs:9,55` | Confirmé. Le rend facultatif est bien un delta 149. |
| Le pilote Claude natif ignore `can_use_tool` | `crates/bridget-transport/src/claude_stream_json.rs` (grep : zéro occurrence ; seul l'accusé d'interruption est traité vers 1049-1075) | Confirmé. Le refus corrélé 149 est un comportement nouveau. |
| Le profil est épinglable par définition | `registry.rs:61-64` (`claude_config_dir` injecté, non secret) | Confirmé : la voie same-family peut garantir le même `config_dir`. |
| Source Codex = paramètres réels de `turn/start` | `CodexAdapterV2.ts:6220` (`buildCodexTurnStartParams`) | Confirmé. |
| Source Claude = queryOptions finales, mode mutable | `ClaudeAdapterV2.ts:7290-7395` (`makeClaudeQueryOptions`, `setPermissionMode`, réutilisation du contexte vivant) | Confirmé. `plan.md:64` décrit le vrai code. |
| Champs d'approbation granulaire Codex réels | `packages/effect-codex-app-server/src/_generated/schema.gen.ts:129-146` | Confirmé : `mcp_elicitations`, `request_permissions?`, `sandbox_approval`, `skill_approval?`. Union fidèle, rien d'inventé. |
| `approvals_reviewer` user/auto_review/guardian_subagent réels | `schema.gen.ts:153-160`, `CodexAdapterV2.ts:710` | Confirmé. T3 pose `auto_review` aujourd'hui. |
| Défaut `settingSources` du SDK | SDK `@anthropic-ai/claude-agent-sdk@0.3.276`, `sdk.mjs` : `ist=["user","project","local"]` | Confirmé : sans option, le CLI charge user+project+local. Le champ `settings_sources:"provider_default"` du contrat est honnête, ni plus ni moins. T3 ne passe pas `settingSources` sur le chemin principal (seule la sonde de capacités en passe, `ClaudeProvider.ts:231`). |

## 2. Preuves d'environnement local (conception, pas preuve fournisseur)

Ces constats décrivent cette machine. Ils ne prouvent pas un comportement
fournisseur réel. Ils ne couvrent pas tous les contextes futurs.

| Constat | Résultat |
|---|---|
| `/Users/moi/.claude-glm/settings.json` | Clés présentes : `enabledPlugins`, `extraKnownMarketplaces` seulement. Aucune clé `permissions`, `sandbox`, `hooks`, `policyHelper`, `apiKeyHelper`, `env`. |
| `/Library/Application Support/ClaudeCode/managed-settings.json` | Clés présentes : `modelPricing` seulement. |
| Dropins et remotes | `managed-settings.d` absent. `remote-settings.json` absent dans `~/.claude`, `~/.claude-glm` et le dossier application. |
| Cible T3 `claude_glm` | `/Users/moi/.t3/userdata/settings.json` : `binaryPath=/Users/moi/.local/bin/gclaude`, `homePath=/Users/moi/.claude-glm`. |
| Wrapper `gclaude` | Exporte l'auth Z.AI et des variables `ANTHROPIC_*`. Aucun override permission, settings ou tools. Ligne finale : `exec claude "$@"` (résolution PATH, voir G-P-04). Empreintes relevées : `gclaude` = `dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e` ; `claude` 2.1.296 = `c9b5341637becbd423ddffc5b254afb645682a3868cb708bbc6cc0e7bb419937`. |
| Registre Bridget vivant | `/Users/moi/.config/bridget/agents.json` contient `cursor` et `claude` seulement. Aucune entrée `glm` (voir G-P-05). |

Le couple launcher/profil est bien identique des deux côtés T3 (`claude_glm`)
et contrat 149 (`permissions.md:217-223`). Le chemin positif same-family est
donc ancré sur des inputs réels et vérifiables par digest.

## 3. Findings

Sévérités : majeur = bloque la gate concernée ; moyen = corriger avant code du
volet ; mineur = à réconcilier avant clôture T001.

### G-P-01 — majeur — Aucun oracle pour le refus corrélé `can_use_tool` et l'échec explicite `permission_denials`

- Contrat : `contracts/permissions.md:313-321` exige un refus automatique corrélé
  de toute trame `can_use_tool` non couverte, et un échec terminal nommé
  (`permission_not_inherited` ou `provider_permission_denied`). La fin de tour
  seule ne prouve jamais un succès.
- État réel : le pilote natif ignore ces trames (`claude_stream_json.rs`, zéro
  occurrence). C'est un comportement nouveau, prévu par T012 (`tasks.md:28`).
- Trou : la stratégie n'a aucun scénario pour ce chemin. S149-14 écrit un fichier
  autorisé. S149-15 refuse avant spawn. Rien ne prouve le refus en cours de
  mission ni l'impossibilité d'un faux succès.
- Risque : une mission bloquée par une permission se termine « normale ».
  SC003 « une action hors politique est refusée côté enfant » reste sans preuve.
- Fix concret : ajouter un scénario fixture E2E (S149-28). Le fournisseur factice
  émet une trame `can_use_tool` pour une action hors politique. Oracle : refus
  corrélé au `request_id` de la trame, état tâche `failed` avec code nommé,
  zéro résultat de succès, compteur de lancements inchangé. Mapper FR008/SC003.
  Une variante réelle reste optionnelle si la trame n'est pas déclenchable chez
  le fournisseur réel ; la limite serait consignée.

### G-P-02 — majeur — Aucun oracle pour `settings_revision_changed`

- Contrat : `contracts/permissions.md:211-216` fige launcher, profil, cwd,
  sources et digests. Toute divergence avant premier lancement ou à une reprise
  rend `settings_revision_changed`, sans adopter le nouveau fichier.
- État de la stratégie : S149-09 vérifie le snapshot inchangé au rejeu.
  S149-26 vérifie la revision et le mode Claude. Aucun scénario ne modifie un
  digest (launcher ou fichier source) entre admission et lancement ou reprise,
  puis n'attend le refus nommé.
- Risque : la garde centrale de FR019 (« un rejeu retourne la tâche initiale et
  n'élargit pas les droits » côté sources) n'a pas de preuve négative.
- Fix concret : deux oracles. (a) Unitaire : fonction de recontrôle des digests
  testée avec un fichier muté, un fichier supprimé et un fichier ajouté →
  `settings_revision_changed` dans les trois cas, aucune re-résolution.
  (b) Fixture E2E : mutation du fichier source de la fixture entre admission et
  premier lancement, puis à une reprise → refus nommé, compteur de lancements
  inchangé. Mapper FR019/SC003.

### G-P-03 — moyen — Code de refus `permission_source_unavailable` hors inventaire

- `contracts/permissions.md:229` nomme `permission_source_unavailable` pour les
  sources managed opaques. Le tableau de mappage (`permissions.md:301`) rend les
  mêmes cas avec `permission_attestation_unavailable` ou
  `permission_mapping_unavailable`. La stratégie (§9.5) liste quatre codes et
  aucun scénario ne nomme `permission_source_unavailable`.
- Risque : un implémenteur rend un code que les tests n'attendent pas, ou
  l'inverse. La divergence entre deux sections du même contrat est déjà réelle.
- Fix concret : soit définir `permission_source_unavailable` dans l'inventaire
  des codes du contrat, l'ajouter à la ligne 301 du tableau et à un oracle
  dédié ; soit supprimer ce nom et unifier sur `permission_mapping_unavailable`
  limité au contexte managed opaque. Une seule source de vérité pour les codes.

### G-P-04 — moyen — `cli_revision` ne couvre pas le CLI résolu

- `contracts/permissions.md:189` fige « SHA256 des octets du launcher choisi ».
  Le launcher réel `gclaude` termine par `exec claude "$@"` (ligne 30), résolu
  via PATH. Le binaire réellement exécuté (`/Users/moi/.local/bin/claude`,
  empreinte relevée en §2) n'est pas dans la preuve.
- Risque : une mise à jour légitime du CLI entre admission et lancement, ou
  entre deux reprises, change l'application des règles sans déclencher
  `settings_revision_changed`. La promesse « mêmes inputs » perd un input.
- Fix concret : ajouter dans `launch_context` le binaire résolu avec son
  `sha256`, capturé par l'adaptateur propriétaire, recontrôlé comme les autres
  digests. Alternative minimale : épingler le chemin absolu du CLI dans la
  définition native (le modèle existe : l'entrée `claude` du registre vivant
  utilise déjà un chemin absolu). Le critère « hash de commande ET hash du CLI
  réel » est alors couvert sur les deux contextes.

### G-P-05 — moyen — Aucune cible `glm` dans le registre natif vivant

- Le contrat (`permissions.md:208-223`) rapproche `cli_path` et `config_dir` de
  « la définition native autorisée ». Le registre vivant
  `/Users/moi/.config/bridget/agents.json` ne contient pas d'entrée `glm`.
  La cible glm n'existe que dans des registres privés de recette passés par
  variable d'environnement 148.
- Risque : la recette réelle (registre privé) et l'admission (définition native)
  peuvent porter des couples launcher/profil différents. Le contrat exige le
  même couple.
- Fix concret : T003 (`tasks.md:16`) consigne le registre source réel de la
  recette : chemin absolu, entrée exacte `glm` (`command=gclaude`,
  `claude_config_dir=/Users/moi/.claude-glm`, empreinte du launcher et du CLI).
  T038 fait de même pour le standalone. Les documents affirment que le registre
  vivant de production ne reçoit pas d'entrée `glm` sans décision humaine
  (aucune config de production modifiée).

### G-P-06 — mineur — Marqueurs de spec périmés et phrase de plan ambiguë

- La fiche synthèse de `spec.md:19-21` marque `tasks.md: ✗` et `plan.md: ✗`
  alors que les deux fichiers existent. `plan.md:39` écrit que le plan attend
  une « correction documentaire » de FR018, alors que le texte actuel de FR018
  (`spec.md:190-193`) porte déjà la forme corrigée demandée.
- Risque : une double lecture du statut de la spec à la gate.
- Fix concret : T001 (`tasks.md:14`) réconcilie la fiche et reformule
  `plan.md:39` au passé ou au conditionnel exact. Aucun changement de fond.

## 4. Couverture des critères G-P

| Critère | État | Preuve ou trou |
|---|---|---|
| Schéma `bridget_session` v2 = identité 148 + `permissions` v1, arguments vides fermés | Couvert | `permissions.md:38-76` ; parseur fermé 148 vérifié ; corrélations driver/kind, bornes 256 o, 64 KiB. |
| Callback de confiance, lien credential/run, cwd fournisseur, politique finale | Couvert | `permission_callback` `t3_runtime` (`permissions.md:130-134`) ; fait lié au credential et au `run_id` (`permissions.md:234-243`) ; source = paramètres validés de `turn/start` et queryOptions finales. Vérifié contre le code T3 réel. |
| Mise à jour Claude init/status/mode ; settings réellement appliqués après montage MCP | Couvert | `plan.md:64` ; T007 (`tasks.md:23`) ; réutilisation et `openedPermissionMode` confirmés dans `ClaudeAdapterV2.ts` ; les options finales incluent la fusion des outils MCP montés. Défaut SDK `["user","project","local"]` vérifié : `provider_default` est exact. |
| Source Codex sandbox/approval exacte | Couvert | `permissions.md:79-114` ; champs granulaires et reviewer fidèles au schéma généré ; flags non représentés → attestation indisponible. |
| Source Claude same-family launcher/profil/cwd/sources/overrides ; digests fichier entier ou `absent` ; jamais auth/env/MCP bruts | Couvert | `permissions.md:136-199` ; `permission_sources` kind+path+sha256 ; interdiction explicite de copier le brut. Trous de preuve : G-P-04. |
| Snapshots natifs figés avant effet ; retry sans élargissement ; preuve hors verrou puis recontrôle binding | Couvert | `permissions.md:26-30`, `:245-250` ; invariant 5 du plan (`plan.md:49`) ; data-model (`data-model.md:59-70`). Oracle de divergence : G-P-02. |
| Aucun credential T3 persisté, transmis à l'enfant ou révélé par Debug | Couvert | `permissions.md:252-255` ; T009/T013/T015 ; S149-11. |
| Standalone natif : Codex `config/read` transport ; Claude PTY hook propriétaire (mode/session + inputs choisis, jamais prompt/agent) ; propriété de la connexion ; révocation | Couvert | Table `permissions.md:264-269` ; hook PreToolUse observateur sans décision ; fait lié à la connexion primaire (`permissions.md:259-263`). |
| Voie positive même profil sans fusion SDK inventée ; callbacks, règles deny, précédence, sources claires | Couvert | `permissions.md:201-230` ; la précédence reste déléguée au fournisseur, Bridget ne refusionne pas ; `settings_overrides` limité à `permissions` ; clés de sécurité non représentées → chemin indisponible. |
| Refus managed opaque limité à son contexte | Couvert avec défaut de nommage | Voir G-P-03. |
| Changement de source avant lancement/reprise → `settings_revision_changed`, sans re-résolution élargie | Contrat couvert, oracle manquant | Voir G-P-02. |
| Table de mappage : same-family ; full Codex→GLM positif ; readonly explicite ; workspace-write Codex→GLM refus de confinement OS | Couvert | `permissions.md:291-301` ; `provider_confinement_unavailable` sans équivalent attesté ; « jamais flag --yolo global » ligne 293 ; S149-24. |
| Remplacement du refus development 148 sans yolo global | Couvert | Refus 148 confirmé au code ; S149-24(3) « aucun réglage global modifié » ; wrapper limité à la définition enfant (`plan.md:74`). |
| Parent ordinaire Claude SDK full-access → GLM autonome ; premier parent Claude PTY hors T3 ; pas de substitution descendant managed → parent externe | Couvert | Table des sources distincte (`permissions.md:265-266`) ; risque gate bloquant assumé (`plan.md:146`) ; règle correction 3 de la stratégie. |
| Admission hors projet = logique, pas confinement OS pour tout full-access | Couvert | `permissions.md:21-24` ; correction 2 de la stratégie ; S149-10/S149-15(1). |
| `--permission-prompts none` / `can_use_tool` corrélé ; faux succès impossible | Contrat couvert, oracle manquant | Voir G-P-01. |
| Aucun nouveau bypass par une politique 149 globale | Couvert | Wrapper = définition enfant seule ; S149-24(3) ; opt-out : réutilisation des chemins existants, aucun toggle nouveau (`permissions.md:32-34`, `plan.md:153`). |
| Pas de restart, pas de config de production | Couvert | Phases 5, T043-T045, non-buts de la stratégie ; managed/profile locaux inchangés requis par S149-14. |

## 5. Couverture FR / SC demandée

| Exigence | Couverture | Remarque |
|---|---|---|
| FR007 (bornes parent, aucun gain) | Couverte | S149-01, S149-02, S149-06, S149-07, S149-15, S149-24, S149-25. |
| FR008 (lecture/écriture GLM+Codex, sans grant, héritage, refus nommés) | Couverte avec trou | Le chemin positif réel est S149-14 et T037. Trou : refus en cours de mission (G-P-01). |
| FR009 (identique sans T3) | Couverte | S149-12, S149-14, T038. |
| FR011 (opt-outs et révocation, aucune projection sans preuve) | Couverte | S149-03, S149-20, S149-21, S149-25, S149-26. |
| FR012 (choix humains MCP conservés, grants hors MCP) | Couverte | S149-21, T005-T009, T013. |
| FR018 (permissions effectives préservées, retrait de garde limité à l'héritage) | Couverte | S149-02, S149-06, S149-15, S149-20, S149-24 ; texte FR018 actuel conforme. |
| FR019 (figeage avant effet, retry sans élargissement, mode UI sans effet) | Couverte avec trou | S149-09, S149-26. Trou : divergence de digest (G-P-02). |
| SC003 (écriture permise réussit, hors politique refusée, découverte = lecture) | Couverte avec trous | G-P-01 et G-P-02 portent tous deux sur SC003. |
| SC004 (même délégation sans T3) | Couverte | S149-12, T038, avec séparation premier parent externe / descendant managed. |
| SC005 (opt-outs masquent la projection, preuve révoquée ferme, moteur survit) | Couverte | S149-20, S149-21, S149-25, S149-26, S149-18. |

## 6. Axes constitution

### Complexité (article XVIII)

Conforme. Aucun finding. Index `root_owner_agent_id`/`parent_task_id` et
`child_agent_id` unique (`data-model.md:33-39`). Pagination `O(log N + P)`,
P≤100. Réconciliation `O(n)` une fois. Séquence incrémentée dans la transaction
d'état. Registre `O(log n + a)` documenté (`registry.rs:196`). Aucune double
boucle ni N+1 proposé.

### Minimalisme et Frugalité (article XIX)

Potentiel minimalisme : ~70 lignes de redondance documentaire, aucun code
encore écrit. Les invariants de `permissions.md`, `research-native.md`,
`plan.md` et l'ADR149 répètent les mêmes règles trois à quatre fois. Ce coût est
accepté comme checkpoint de gate, mais il produit déjà G-P-03 (un code nommé
deux fois différemment). Fix structurel : une table unique des codes de refus et
des sources dans `permissions.md`, citée par les autres documents sans
reformulation. Les unions du contrat ne sont pas spéculatives : chaque valeur
est vérifiée dans le wire réel (schéma généré Codex, défauts SDK). Rien à
retirer du contrat sur ce plan. Le callback `permission_callback` ne transporte
aucune fonction : c'est un fait, pas un wrapper. Bien.

### Vertus LLM et Responsabilité future (article XX)

Charge future réduite si G-P-03 et G-P-04 sont appliqués : les refus nommés, le
refus d'inventer une fusion SDK et le refus d'assertions de confinement
suppriment des classes entières de débogage futur. Le contrat reste explicable
en intention, invariants et limites : la preuve porte sur des inputs, le
fournisseur applique ses règles, un manque de preuve est un refus nommé. Les
limites sont déclarées (recette PTY exigée avant validation, contrats non figés
jusqu'à la gate). Le point de vigilance est la multiplication des documents qui
répètent les refus : sans source unique, la prochaine évolution risque une
divergence silencieuse.

## 7. Demandes réalistes pour les tests réels

La revue valide ces exigences et demande qu'elles restent non négociables :

1. Écrivain réel GLM `glm-5.3-flash`, modèle exact. Toute autre valeur, y
   compris un repli `glm-5.3`, échoue le test (stratégie correction 8, S149-14,
   §6 des commandes). Aucun repli de modèle.
2. Écrivain réel Codex, un enfant, une écriture, un refus hors politique
   (T037).
3. Premier parent GLM réel hors T3, voie PTY par sa source propriétaire,
   avec T3 absent (S149-14 variante (a), T038). Ce cas ne peut être remplacé ni
   par un descendant managed ni par une fixture (correction 3).
4. Registre privé de recette cohérent avec le couple attesté : `gclaude` +
   `/Users/moi/.claude-glm`, consigné par chemin absolu (G-P-05).
5. Empreintes gelées et recontrôlées : launcher ET CLI résolu (G-P-04), settings
   et commandes sources inchangés en fin de recette.

Ces demandes sont réalisables avec les harnais existants (`idempotent.rs`,
recette 148 étendue) et sans aucun redémarrage ni config de production.

## 8. Conditions de levée du verdict

REQUEST_CHANGES levé quand :

1. G-P-01 : un scénario fixture prouve le refus corrélé `can_use_tool` et
   l'échec explicite, avec l'oracle exact ci-dessus.
2. G-P-02 : deux oracles prouvent `settings_revision_changed` (unitaire +
   fixture, avant lancement et à la reprise).
3. G-P-03 : un seul nom par cas de refus managed opaque, présent dans le
   contrat, le tableau de mappage et un oracle.
4. G-P-04 et G-P-05 : digests du CLI résolu et registre glm de recette consignés
   dans le contrat ou les preuves T003/T038.
5. G-P-06 : réconciliation documentaire T001.

La stratégie doit aligner ses oracles sur le contrat final avant la gate, comme
le prévoit déjà `plan.md:13`. Aucun code de permission ou d'héritage ne démarre
avant la fermeture de ces points et le GO principal.

## 9. Limites de cette revue

- Revue documentaire et de lecture de code. Aucun test, aucun build, aucune
  recette réelle exécutés.
- Les constats de §2 décrivent cette machine à cette date. Ils ne sont pas une
  preuve de comportement fournisseur et ne couvrent pas tous les contextes.
- Le volet Lineage (G-L, `contracts/lineage.md`) n'est pas jugé ici.
- Les numéros de ligne valent pour les versions des fichiers à la date de cette
  revue (base 148 `6807c22b`, worktrees 149 inchangés).
