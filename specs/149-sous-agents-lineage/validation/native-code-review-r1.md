# Revue code natif r1 — session149 (GLM 5.3 Flash, indépendante)

Date : 2026-10-10. Base : `6807c22b7ada683f757486a6382aeda170ec68ab`, worktree `149-sous-agents-lineage`, tout le diff production est non commité.
Revueuse : GLM 5.3 Flash. Aucune exécution, aucun build, aucun test lancé, aucun modèle réel. Lecture seule du diff natif et des fichiers nouveaux.

## Verdict : REQUEST_CHANGES

Un finding Medium bloque (F1, élargissement possible dans le mappage Claude→Codex). Quatre findings Low ne bloquent pas seuls. Le reste de l'implémentation examinée respecte les contrats149 sur les axes demandés.

## Findings

### F1 — Medium — mappage Claude→Codex : `allowed_tools` du parent ignoré

- Chemin : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/native_permissions.rs` lignes 252-257.
- Attendu (contrat permissions, table mappage) : « Claude bypassPermissions … Full-access seulement si les éventuelles restrictions et médiations parent sont représentables. Sinon permission_mapping_unavailable. »
- Réel : la branche `full` vérifie `permission_mode`, `tool_approval` et `tools`, mais jamais `allowed_tools`. `validate_permissions` (transport, ligne 78) accepte `allowed_tools` jusqu'à 128 entrées.
- Déclencheur : un fait T3 `provider_turn` avec `{kind:claude, permission_mode:"bypassPermissions", permission_callback.tool_approval:"allow", tools:{preset:"claude_code"}, allowed_tools:["Read","Grep"]}` délégué vers Codex produit `{"approval_policy":"never","sandbox_policy":{"type":"dangerFullAccess"}}`. Le parent ne pouvait employer que Read/Grep ; l'enfant Codex reçoit l'accès complet. Le miroir codex→claude traite bien la médiation (`approval_policy!="never"` → `permission_mapping_unavailable`, ligne 217) ; la direction Claude→Codex oublie la restriction outillage.
- Fix minimal pour Sol : dans le calcul de `full`, exiger `policy["allowed_tools"].as_array().is_none_or(|a|a.is_empty())`, sinon `permission_mapping_unavailable`. Une allowlist Claude n'est pas représentable comme bac à sable Codex.

### F2 — Low — révision du fait Claude insensible au changement de `cwd`

- Chemin : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/src/claude_stream_json.rs` ligne 999.
- Attendu (contrat) : « Revision augmente lorsque le fait courant change. » Le `cwd` fait partie du fait.
- Réel : l'incrément ne porte que sur `session` et `policy` (`o.0!=session||o.1!=policy`). Le pilote Codex inclut `cwd` (`codex_app_server.rs`, `observe_native_permissions`). Un `cwd` changé à session et politique identiques garde la révision stable.
- Impact borné : le fait stocké reste à jour (la valeur `cwd` est remplacée) ; seule la sémantique de révision faiblit. Aucun élargissement démontré.
- Fix minimal : ajouter `||o.3!=cwd` à l'incrément, sur le modèle Codex.

### F3 — Low — `tools: []` vide accepté puis lu comme « lecture seule »

- Chemin : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/src/native_permissions.rs` ligne 77 ; `crates/bridget-daemon/src/native_permissions.rs` ligne 255.
- Attendu : un tableau vide ne représente aucun outillage réel de T3 ; le contrat exige des règles non vides.
- Réel : `array(&policy["tools"],false)` accepte `[]` (`all()` sur vide = vrai). Dans le mappage Claude→Codex, `read` devient vrai pour `permission_mode=="plan"` et `tools:[]` ; l'enfant Codex reçoit un bac à sable `readOnly` avec Read/Glob/Grep implicites, plus capable qu'un parent sans aucun outil.
- Fix minimal : refuser un tableau `tools` vide dans `validate_permissions` (branche claude), ou exiger non-vide dans le calcul de `read`.

### F4 — Low — `initialize_lineage` fait échouer l'ouverture du magasin sur donnée héritée pathologique

- Chemin : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/delegation_lineage.rs` lignes 79-92.
- Attendu : la borne 8 missions est une règle d'admission et de migration ; rien n'impose d'arrêter le daemon pour une chaîne héritée trop profonde.
- Réel : `visited.len()>8` et `records.len()>4096` retournent `Err` depuis `DelegationStore::new` → le daemon ne démarre plus. Les liens rompus (`child_instance` différent) suivent déjà un `break` doux qui racine la sous-chaîne ; une chaîne de 9 ou un cycle, non.
- Fix minimal proposé (décision Sol) : traiter le dépassement de profondeur comme le `break` existant (raciner la sous-chaîne de 8), garder `Err` seulement pour un cycle vrai ou un magasin corrompu. La direction fail-closed reste défendable si Sol la assume.

### F5 — Low — lanceur binaire non prouvé transparent accepté

- Chemin : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/native_permissions.rs` lignes 78-85.
- Attendu (contrat) : « Une chaîne non observable … rend permission_source_unavailable pour cette voie seulement. » « Un nom de wrapper ne constitue jamais cette preuve. »
- Réel : un script (`#!`) est refusé sauf empreinte `gclaude` épinglée ; tout binaire non script passe avec `resolved_cli_path == cli_path`, sans preuve qu'il transmet les drapeaux à `claude`. Le registre est une entrée opérateur de confiance, donc l'escalade exige une définition hostile ou maladroite.
- Fix minimal : réserver l'indirection à l'empreinte épinglée — si `launcher != resolved("claude")` et `launcher_revision != TRANSPARENT_GCLAUDE`, retourner `permission_source_unavailable`.

## Axes permissions (T009–T014) — vérifié par lecture

- Identité et preuve privées : `NativeDelegationT3{request,proof}` ; `reattest` HTTP hors verrou, puis recontrôle identité/instance/garde147 sous verrou (`native_delegation.rs::handle_attested`, `handle_with_fact`). Le token reste masqué (`Debug`), `expose_for_attestation` a un seul appelant. Aucun argument MCP ne porte preuve, endpoint ou PATH.
- Fait v2 réel seulement : `t3code_mcp.rs::attest` accepte v1 sans `permissions` et v2 avec `permissions` validées (`validate_permissions(fact,true)` → source `provider_turn`, callback `t3_runtime`) et corrélées session/instance. Un fait invalide ne retombe jamais en v1.
- Retombée v1 : uniquement absence de fait jamais créé ; tombstone/refus stockés dans `native_permission_refusals`, jamais recréés.
- Freeze avant effet : snapshot à l'admission (`parent_fact` → `child_policy` → `snapshot`) ; `spawn_task` refigure `posture:None` pour les hérités, recontrôle `snapshot_contexts` et `launch_context`, puis réinjecte `CHILD_POLICY_ENV`/`CHILD_SOURCES_ENV`. Retry idempotent par `request_id` ne réadmets pas. Définition figée reconstruite par `AgentRegistry::from_resolved(task.definition)`.
- Révocations conservées : `spawn_task` revérifie `revoked_agent(owner/root_owner)` et `revoked(instance)` ; déconnexion et changement d'identité purgent faits et refus (`revoke_identity_authorizations` appelé aux six points de coupure).
- Postures : omission = inherit (`Option<SpawnPosture>`, schéma MCP `required` sans `posture`) ; discovery réduit et étend `disallowed_tools` sans retirer de deny ; development depuis parent lecteur → `permission_not_inherited` (`child_policy` ligne 263).
- Codex final : politique transportée sur `turn/start` (mêmes formes que le contrat T3 attesté) ; `thread/start` porte `cwd`+modèle seulement ; enfant sans politique → le fil ne démarre pas (test wire présent). Voir limites.
- FullCodex mission-local : workspace confinement absent → `provider_confinement_unavailable` (`child_policy` lignes 206 et 216) ; `externalSandbox` refusé.
- Résolution lanceur/CLI sous env final : `resolved()` sur le PATH de l'environnement construit, `canonicalize`, SHA256 deux couples, sources revisionnées (fichiers, répertoires triés ≤256, plugins inspectés sans exécution, MDM plist → refus). Mismatch → `settings_revision_changed` avant effets (`recheck_frozen_inputs` au spawn des deux transports ; recontrôle après chaque respawn transport).
- Refus nommés, zéro fallback grant : inventaire 149 respecté partout ; `development_refusal` rapporte la vraie cause, plus jamais `development_grant_required`.
- `can_use_tool` : refus corrélé `request_id` (≤256, sans contrôle), dédoublonné (borne 4096), journal `permission_denied`, `result` avec `permission_denials` ou trame refusée → `ManagedTerminal::Failed{provider_permission_denied}`. Aucun faux succès possible par cette voie.
- Observer PTY : overlay `PreToolUse` séparé des permissions ; socket et overlay 0600 `create_new` ; nonce double UUID jamais publié ; ACK daemon attendu avant de laisser passer (deadline 2s) ; pair filtré par PID+naissance+ascendance (LOCAL_PEERPID/SO_PEERCRED, profondeur 32) ; session/cwd/révision d'overlay recontrôlés ; fait lié au `request_id` observé ; Drop RAII + `OverlaySetup` armé pour les sorties précoces ; refus via JSON `permissionDecision:"deny"` officiel, code nommé. Aucune prétention de confinement OS dans le code.

## Axes lineage (T016–T022) — vérifié par lecture

- Fix Sol vérifié indépendamment : `initialize_lineage` compte la mission migrée (`visited` démarre avec la mission elle-même). Sept ancêtres → 8 missions admises ; le 8ᵉ ancêtre → `len 9 > 8` refusé ; cycle → refus. L'admission indépendante `parent_lineage` borne à 8 aussi (parent + 6 ancêtres + nouvelle mission). `descendants()` : CTE récursive `depth<8`, limite 4097 → refus.
- Store gen/seq : `increment` dans la transaction de la mutation ; rotation de génération à `HUMAN_LINEAGE_MAX_SEQ` (2^53−1, sûr JS) ; signal après commit, raciné par root (`committed` → `publish` filtre `context.root`). Lecture, rejeu, refus, ACK et `save` invisible ne signalent pas (`visible_changed`).
- Pagination : ≤100 lignes et ≤128 KiB par page (enveloppe 4 KiB réservée) ; curseur ≤2048, refusé si racine/génération/seq divergent → `snapshot_changed` ; ordre `(created_at,task_id)` indexé.
- Show/résultat : bornes 256 KiB, fenêtres 16 KiB sur frontières UTF8, `result_offset_invalid` hors bornes, aucune écriture à la lecture.
- Journal : 100 événements / 16 KiB, `JournalEntry` sérialisés entiers, lacunes explicites (`journal_gap`, `entry_too_large`), `journal_unavailable` sans invention ; chemin résolu par le daemon seulement, jamais retourné au client.
- Watch : `Ready{seq:0}` premier, préservé par l'élagage (non coalescible) ; événements fermés version/génération/seq (+tag `status`) ; bornes 128 suivis ; invalidation par garde, resync sur génération ou backlog ≥16 ; aucun corps, aucun polling.
- Journal follow : réutilise le vrai relai attach ; `SnapshotCaughtUp` poursuit la boucle (ne ferme pas) ; fin seulement sur `End` fournisseur ou invalidation ; vérification d'autorité avant chaque publication.
- Cancel humain : reçu idempotent `(root,request_id)`, `envelope_mismatch`, jamais `cancelled` avant preuve ; `tick` pilote l'arbre natif borné ; état terminal renvoyé tel quel.
- CLI : grammaire fermée (`cli_lineage.rs`), doublons et restes refusés, défauts 50/16 384, UUID canoniques, aucun champ d'autorité côté client ; autorité root dérivée de la garde147 (`human_authority_guard`, fil opaque ≤2048).
- Marqueur T3 : 7 champs fermés, UUID canoniques, 9 états natifs ; présent-invalide → fermeture sans repli, dans le parseur v1 ET v2, avant toute liaison/wrapper/présence ; exclusion complète de la session. Le préfixe UUID seul ne suffit nulle part.
- Pas de régression148 : chemin legacy discovery intact (`authorize_cwd` + `for_delegation(Discovery)`), `Task` avec `serde(default)`, `root_owner()` retombe sur `owner` pour les anciens payloads, `save_projected` ne suit jamais une instance ou un retry.

## Limites de cette revue

1. Aucune exécution. Les preuves G-P-01, G-P-02, G-P-07, G-P-08 et les oracles lineage restent à produire par les tests GLM (compilation/exécution en cours ailleurs). Cette lecture ne vaut aucune gate.
2. Comportement fournisseur Codex : le code envoie `approvalPolicy`/`approvalsReviewer`/`sandboxPolicy`/`cwd` sur `turn/start`, formes identiques au contrat T3 attesté (APPROVE). Que le fournisseur honore ces champs par tour n'est pas prouvé ici ; c'est l'objet des recettes T037/T038. Si un jour le fournisseur les ignorait, la politique figée resterait une déclaration ; le fait observé de l'enfant révélerait la politique réelle, mais les chaînes imbriquées mapperaient depuis la déclaration.
3. Recette T040 (composition `--settings` overlay) non faite : le code refuse un `--settings` préexistant et fige le contexte avant ajout ; la composition réelle par le CLI reste à prouver.
4. Aucun modèle réel T037/T038, aucune base de production, aucun redémarrage : hors périmètre et hors mes droits.
5. Les tests natifs présents n'ont pas été compilés ni lancés par moi ; leurs bugs de compilation appartiennent au compilateur GLM.
6. Les revues T3 APPROVE et les corrections C01/C02/F1/F2 ne constituent pas une preuve moteur ; cette revue n'en hérite pas.

## Synthèse

F1 est le seul refus : une ligne de garde manque dans le mappage Claude→Codex. F2–F5 sont des corrections locales que Sol peut prendre avec F1. Le fix Sol de `initialize_lineage` est correct et vérifié indépendamment de l'admission. Aucune recette n'est déclarée validée par cette lecture.
