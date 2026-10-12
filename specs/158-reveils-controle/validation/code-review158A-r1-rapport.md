# Revue adverse READONLY - SPEC158A (r1)

Verdict : **APPROVE_WITH_CHANGES**

Aucun défaut bloquant le contrat 158A n'a été trouvé à la lecture.
Deux défauts moyens et un défaut faible touchent la fidélité des réponses
et une preuve. Ils doivent être corrigés ou documentés avant le GO.

Aucun Cargo, test, build, scanner, provider ou service n'a été lancé.
Aucune source n'a été modifiée. Ce rapport et le manifeste sont les seuls fichiers écrits.

## Vérifications d'intégrité

- `scope-files.json` : SHA256 `5081979d...e4ea43cb25e` vérifié.
- `diff158A-final.patch` : SHA256 `590c8bbb...d87399b` vérifié (4612 lignes, 137 hunks).
- Fichiers du worktree lus : SHA identiques à `scope-files.json` (daemon.rs, native_delegation.rs, mcp.rs, mcp_scopes.rs, contrat, ADR).

## Findings

### F1 - MOYEN : arrêt committé, mais réponse de refus

- Fichier : `crates/bridget-daemon/src/daemon.rs:478-506` (`handle_delegation_halt_set`).
- Constat : l'état d'arrêt et le reçu sont committés dans
  `set_delegation_halt`. Ensuite, la boucle `human_cancel` peut échouer
  (par exemple à cause du plafond de 4096 reçus human_cancel). Dans ce cas,
  le code fait `failed = true; break;` puis répond
  `DelegationControlRejected { store_unavailable }`.
- Scénario : un humain lance `halt-delegations --cancel-active`. Une cible
  échoue. L'arrêt est posé (`halted=true`). La CLI affiche « refusé » et sort
  avec le code 1. Si l'humain relance la même commande avec le même
  `command_id`, il reçoit `replayed: true` et aucune annulation. Il ne lui reste
  que `delegations cancel <id>`, cible par cible.
- Oracle : test qui force une erreur sur `human_cancel`. Attendu : `halted=true`
  est confirmé, et la réponse dit qu'une annulation est incomplète (avec le
  compte `cancelled`). Le refus ne doit pas être affiché.
- Correctif minimal : renvoyer `DelegationHaltState` avec un indicateur
  `cancel_incomplete`, ou afficher « arrêt posé, annulation partielle :
  `delegations cancel` » dans la CLI. Documenter le choix dans le contrat.

### F2 - FAIBLE à vérifier : un ancêtre absent bloque toute la commande

- Fichier : `crates/bridget-daemon/src/daemon.rs:442-456`.
- Constat : pour `--cancel-active`, le calcul des racines retourne
  `StoreUnavailable` si un parent est absent de `tasks()`, ou si la profondeur
  atteint 8. La transaction est alors annulée, arrêt compris.
- Limite : la règle de purge (`crates/bridget-daemon/src/delegation_lineage.rs:253`)
  n'a pas été lue. Je ne sais pas si un parent peut disparaître alors qu'un
  enfant est actif.
- Oracle : un parent terminé et purgé, un enfant actif, puis `halt --cancel-active`.
  Attendu : arrêt posé ou refus explicite. Dans les deux cas, l'état doit être cohérent.
- Impact : le simple `halt` (sans annulation) n'est pas touché. Seul
  `--cancel-active` est bloqué.

### F3 - FAIBLE : code de sortie de `control status` modifié

- Fichier : `crates/bridget-daemon/src/cli.rs` (bloc `Some("status")`, hunk
  `@@ -6102,6 +6197,33`).
- Constat : après l'affichage de l'état, une réponse refusée à la lecture
  de la vue d'arrêt fait `exit(1)`. Le contrat ne mentionne pas ce changement.
- Correctif : soit documenter le code 1, soit n'afficher qu'un avertissement
  sur stderr.

### F4 - FAIBLE : oracle ajusté au comportement

- Fichier : `crates/bridget-daemon/src/daemon/native_delegation.rs`, test
  `control158_halt_et_pause_4_combinaisons_a_l_admission`, ligne 1336 du patch :
  `expected["suspended"] = json!(true);`.
- Constat : le rejeu de `Continue` ne renvoie pas la même réponse que la
  première admission. Le test change la valeur attendue pour l'aligner.
- Le contrat dit seulement que le rejeu « rend la tâche existante ». Il ne dit
  pas si le champ `suspended` est figé ou dynamique.
- Correctif : décider le contrat (vue dynamique documentée, ou réponse figée),
  puis tester exactement ce choix.

## Points vérifiés par lecture (conformes)

- Ordre de `Continue` : le rejeu (`saved.rounds`, `native_delegation.rs` ~1775)
  précède la garde d'arrêt. `Delegate` lit `by_agent_request` avant la garde
  (~1063-1080). Pause puis arrêt sont composés dans `referent_control.rs:145-155`.
  Priorité de l'arrêt confirmée par le test à quatre combinaisons.
- Clé neuve refusée pendant l'arrêt : `delegations_halted`, sans création de tâche.
- Plafond discovery : `CallerScope` lit l'index UNIQUE `native_delegations_child`
  (`delegation_lineage.rs:126`) et filtre `!cleanup_done` et posture effective
  puis demandée. Un échec RPC, une réponse invalide ou un `caller_read_only`
  non booléen ferment l'appel avant tout effet (`mcp.rs:2172-2194`).
- Portées MCP : 30 entrées (`mcp_scopes.rs:3310-3339`). `readOnlyHint` dérive
  de la base (`mcp.rs:2216-2229`). `bridget_journal` a une base `sends`, donc
  `readOnlyHint` faux. `to` absent ou `null` abaisse à `reads`, chaîne vide
  garde `sends`.
- Vue globale : limite 1..100 et filtre `active|all` contrôlés dans
  `overview_page` (`delegation.rs` ~1786). `total_active` compté avant la
  limite. Projection à 7 champs (`overview_row`).
- Reçus : `native_delegation_halt_commands` gardé à 256 (`ORDER BY ... OFFSET 256`).
  Rejeu typé, sans comptage ni annulation. Test de redémarrage présent.
- CLI : mutations exigent un terminal (`require_interactive_terminal`).
  Usage invalide : code 2. Limite 1..100 validée. Test de parse présent.
- Aucun outil MCP ajouté pour le contrôle humain (`control158_halt_aucun_outil_mcp`).

## Limites de cette revue

- Compilation non vérifiée (interdit par la consigne). Les conversions `?`
  s'accordent par lecture avec les signatures, mais rien n'est prouvé.
- Tests non exécutés. Les 47 tests 158 ne sont pas une preuve de PASS.
- Lecture partielle du diff. Hunks non lus :
  - `daemon.rs` patch 584-1097 (grand bloc `presence_tests`, ~513 lignes) ;
  - `native_delegation.rs` patch 1356-1608 (tests Continue/pause) ;
  - `delegation.rs` patch 1920-2145 (tests store) ;
  - `mcp.rs` patch 2300-3281 (tests), vus seulement par grep des mocks ;
  - `delegation-native.md` patch 3742-3834 ;
  - `skills/bridget/SKILL.md` patch 3835-3882 ;
  - `commandes.md` patch 3883-3984 ;
  - docs de spec patch 3985-4612 (sauf contrat et ADR, lus dans leur version finale).
- Comportement de la purge amont (F2) non lu.
- Le fond du contrat (R1 à R8) est vérifié dans le code à la lecture, sans exécution.
