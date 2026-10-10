# Revue code natif r2 — session149 (GLM 5.3 Flash, indépendante)

Date : 2026-10-10. Base : `6807c22b7ada683f757486a6382aeda170ec68ab`, worktree `149-sous-agents-lineage`, diff production non commité.
Revueuse : GLM 5.3 Flash. Lecture seule des deltas corrigés et de leurs appelants directs. Aucune exécution, aucun build, aucun test lancé, aucun modèle réel, aucune édition.
Périmètre : les cinq objections r1 (`native-code-review-r1.md`) et les deltas annoncés par Sol. Pas de nouvelle exploration générale des 48 fichiers.

## Verdict : APPROVE

Les cinq findings r1 sont fermés. F1, F2 et F5 par correction vérifiée. F3 et F4 par décision Sol explicite, justification vérifiée, sans escalade démontrable. Les deltas complémentaires (source readonly, MCP, alias, observer, réexport) sont conformes aux annonces. Cette approbation porte sur les deltas de code natif. Elle ne vaut aucune preuve moteur : voir Limites.

## Findings r1 — état

### F1 — Medium — CLOSED — `allowed_tools` Claude dans le mappage vers Codex

- Preuve : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/native_permissions.rs` lignes 261-263.
- Réel : la branche `("claude_stream_json","codex_app_server")` refuse maintenant `permission_mapping_unavailable` si `allowed_tools` est un tableau non vide, ou si `tools` est un tableau vide. Le contrôle est en tête de branche, avant les calculs `full` (lignes 265-266) et `read` (ligne 267). Il s'applique donc en development comme en discovery. Le scénario r1 (parent limité à Read/Grep → enfant Codex `dangerFullAccess`) est mort.
- Réponse Sol vérifiée : `allowed_tools` est un champ de règles d'approbation Claude, pas un inventaire d'outils. Une allowlist non vide n'a pas de traduction exacte en bac à sable Codex. Le refus est la seule direction fidèle. `validate_permissions` (transport, ligne 78) continue d'accepter le champ comme fait du fournisseur. Cette séparation fait/ mappage est cohérente.
- Remarque de lecture : un parent Claude `bypassPermissions` avec `tools:["Read","Glob","Grep"]` est aussi refusé (`full` faux sans Bash, `read` faux hors `plan`). Fail-closed, aucune direction permissive ajoutée.

### F2 — Low — CLOSED — révision insensible au `cwd`

- Preuve : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/src/claude_stream_json.rs` lignes 999-1000.
- Réel : l'incrément porte sur `o.0!=session||o.1!=policy||o.3!=cwd`. Le `cwd` est capturé depuis l'événement, avec repli sur la valeur précédente si l'événement n'en porte pas. Même forme que le pilote Codex. Le contrat « la révision augmente quand le fait change » est respecté.

### F3 — Low — CLOSED — `tools: []` accepté comme fait, refusé au mappage

- Preuves : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/src/native_permissions.rs` ligne 77 ; `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/native_permissions.rs` lignes 261-263, 273-275.
- Réel : `validate_permissions` garde `[]` comme fait valide. C'est la décision Sol : un tableau vide décrit un état réel du CLI (builtins désactivés ; MCP distinct). L'escalade r1 est fermée des deux côtés :
  - Claude→Codex : `tools:[]` est refusé au mappage (ligne 261), y compris en discovery. L'enfant Codex ne reçoit plus de bac à sable `readOnly` plus capable qu'un parent sans outils.
  - Claude→Claude : la réduction discovery d'un parent `tools:[]` produit `tools:[]` (lignes 236-240). `only_read_tools` est vrai sur tableau vide (ligne 273), donc la posture effective est Discovery (ligne 274), et un development demandé est refusé `permission_not_inherited` (ligne 275). Aucune écriture ne devient possible.
- La décision est cohérente : le fait décrit le fournisseur, le mappage décrit la délégation. C'est le refus qui protège, pas la validation.

### F4 — Low — CLOSED — refus atomique de la migration assumé par Sol

- Preuves : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/delegation_lineage.rs` lignes 80, 87-92.
- Réel : `records.len()>4096`, un cycle, et `visited.len()>8` retournent `Err` depuis `initialize_lineage`. Sol assume le fail-closed et refuse la troncature proposée en r1.
- Justification vérifiée : le `break` existant (ligne 91, lien `child_instance` rompu) racine une sous-chaîne qui est réellement autonome. Tronquer une chaîne de 9 à 8 créerait un `root_owner_agent_id` faux. Les gardes d'autorité et les reçus d'annulation `(root,request_id)` seraient attachés au mauvais root. Les deux cas ne sont pas symétriques ; la direction Sol est correcte.
- Borne documentée : les nouvelles chaînes 149 ne peuvent pas dépasser 8 (admission bornée, vérifié en r1). Seul un héritage 148 pathologique refuse le démarrage du daemon. C'est une limite contractuelle assumée, pas un défaut de la migration. Elle doit rester visible dans la documentation d'exploitation.
- Tests : le réexport `delegation::task_entry` est retiré (`delegation.rs` ligne 9 ne réexporte que `ProjectionMutation`). `task_entry` reste `pub(crate)` dans `delegation_lineage.rs` ligne 31. Les tests l'atteignent par `use super::*` (`delegation_lineage_tests.rs` ligne 6, module câblé dans `delegation_lineage.rs`). La suppression ne casse pas cet usage.

### F5 — Low — CLOSED — lanceur réservé au CLI exact ou au hash épinglé

- Preuves : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/native_permissions.rs` lignes 10, 78-91 ; `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/src/native_permissions.rs` lignes 134-137.
- Réel : c'est le fix minimal demandé en r1, appliqué. `launcher` est résolu sur le PATH de l'environnement construit et canonicalisé. Si sa révision n'est pas le hash `gclaude` épinglé, alors `launcher != resolved("claude")` retourne `permission_source_unavailable`. Seule l'indirection `gclaude` épinglée passe, et elle résout le `claude` direct. Un script `#!` est refusé (lignes 89-91). Le recheck transport applique la même règle avec le même hash (ligne 136) et revalide les quatre champs `cli_path/cli_revision/resolved_cli_path/resolved_cli_revision`.
- Périmètre : les appelants sont la capture observer (`native_permission_observer.rs` ligne 38), la capture managed (`wrapper.rs` ligne 3971) et les rechecks (`wrapper.rs` lignes 3636 et 3873, `native_delegation.rs` ligne 552). Le chemin legacy 148 (`project_discovery_definition`, `registry.rs` lignes 1304-1327) n'emprunte pas `capture_context` et reste inchangé.
- La valeur du hash épinglé ne peut pas être vérifiée sans exécution. Sa mécanique (double révision, recheck des deux côtés) est correcte.

## Deltas complémentaires — état

### Source readonly — plus d'ajout automatique de `--restricted` sous 149

- Preuves : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/native_permissions.rs` lignes 298 et 312-314.
- Réel : `apply_child_arguments` retire `--restricted` des args enfant et n'en ajoute jamais. Le commentaire documente la raison : `--restricted` écarte les sources user/project/local figées. La réduction discovery est appliquée par `--permission-mode plan`, `--tools` et `--disallowedTools`. Le legacy 148 garde son `--restricted` (`registry.rs` ligne 1318), inchangé. CLOSED.

### Inventaire et deny — conservés

- Preuves : `native_permissions.rs` (daemon) lignes 225, 242-245, 273-274.
- Réel : les writers restent en deny, l'inventaire discovery reste en outils de lecture, les règles allow existantes restent sous l'inventaire et ne réintroduisent pas d'outil retiré (commentaire lignes 246-247). CLOSED.

### Alias `--allowed-tools` / `--disallowed-tools`

- Preuves : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/src/native_permissions.rs` ligne 162 ; `native_permissions.rs` (daemon) lignes 285-289 et 316-318.
- Réel : la capture des options CLI reconnaît les deux formes et les range dans `allowed_tools`/`disallowed_tools`. `apply_child_arguments` retire les deux formes des args enfant, puis réémet la politique figée en camelCase `--allowedTools`/`--disallowedTools`. Un alias passé par la définition enfant ne contourne plus la politique. CLOSED.

### MCP géré — strict-config Bridget, auto-allow limité au legacy

- Preuves : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/wrapper.rs` lignes 3959-3965, 4794-4798, 4801-4803 ; `native_delegation.rs` ligne 595.
- Réel : le montage reste `--strict-mcp-config` + `--mcp-config` vers le fichier éphémère Bridget. Les serveurs MCP du parent ne sont pas copiés. L'auto-allow `mcp__bridget__*` n'est ajouté que si `BRIDGET_NATIVE_CHILD_POLICY` est absent : legacy 148 inchangé, enfant 149 autorisé par le mode, les règles et les sources hérités. `apply_managed_permission_policy` est aussi sauté si une politique est héritée (ligne 3959) : aucun bypass de définition ne s'ajoute sous 149.
- Ordre vérifié : la variable est posée par le daemon au spawn (`native_delegation.rs` ligne 595) et lue depuis l'environnement du processus wrapper (lignes 3869 et 4801). Elle est donc présente avant `apply_managed_mcp` (ligne 3948). CLOSED.

### Observer — E0282 corrigé, mécanique inchangée

- Preuves : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/native_permission_observer.rs` lignes 13 et 54.
- Réel : le type d'acquittement est explicite (`Arc<(Mutex<Option<(String,bool)>>,Condvar)>` et `None::<(String,bool)>`). L'inférence ambiguë disparaît. Le reste est conforme r1 : socket et overlay 0600 `create_new`, nonce et pair descendant du PID provider, recheck du contexte à chaque événement, refus nommé avant tout effet, ACK daemon attendu avant réponse au CLI.
- Contrôle du chemin interactif : l'overlay `--settings` est ajouté après `Observer::start` (`wrapper.rs` lignes 2135-2146). L'observer ne voit jamais son propre overlay dans les args. Un échec de démarrage de l'observer émet un fait indisponible et laisse la session interactive démarrer : observation, jamais condition de lancement. CLOSED.

## Axes non rouverts

Le brief exclut de nouvelles règles MCP, droits, callback ou grant. Aucune promesse de confinement OS n'apparaît dans le code lu. Les refus restent nommés, sans repli de grant, sur les chemins vérifiés (catalogue `native_delegation.rs` lignes 382-388, spawn `?` ligne 436).

## Limites de cette revue

1. Aucune exécution. Cette approbation ne prouve ni les modèles réels ni le comportement du CLI. T037 et T038 restent à venir. Aucune gate ne peut être déclarée verte sur la seule absence de findings.
2. La composition réelle du `--settings` overlay par le CLI Claude et l'arbitrage exact entre `--tools`, `--allowedTools` et les règles de settings restent à prouver par recettes (T040 non faite).
3. La valeur du hash `gclaude` épinglé est un fait opérateur. Je n'ai pas pu vérifier qu'elle correspond au binaire réel.
4. Le fail-closed F4 a une conséquence d'exploitation : un héritage 148 pathologique (chaîne de 9, cycle, ou plus de 4096 missions) empêche le démarrage du daemon. Ce choix est assumé et justifié, mais il doit rester documenté.
5. Le chemin interactif garde l'auto-allow MCP inconditionnel (wrapper.rs ligne 2107). C'est le comportement humain préexistant, hors périmètre 149. Un fait interactif avec `allowed_tools` non vide sera refusé au mappage Claude→Codex, ce qui reste fail-closed.
6. Les tests natifs présents n'ont pas été compilés ni lancés par moi. La corrections E0282 et le retrait du réexport sont vérifiés par lecture de la résolution d'identifiants, pas par le compilateur.

## Synthèse

Les cinq objections r1 sont fermées avec preuve de chemin et de lignes. Les deux décisions assumées (F3 fait-vs-mappage, F4 fail-closed) sont justifiées et vérifiées sans escalade. Les deltas annoncés sont réels et localisés. Aucun nouveau bug réel n'a été trouvé dans les zones lues. Les objections actives sont nulles ; les limites restantes sont d'exécution et d'exploitation, pas de conception. Elles appartiennent aux recettes GLM en cours (T037, T038, T040) et à la documentation du fail-closed F4.
