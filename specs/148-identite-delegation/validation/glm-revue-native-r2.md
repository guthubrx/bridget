# Revue R2 indépendante — Session 148, moteur natif + intégration T3

Lecture seule respectée. Aucun test, Git, build, daemon ni modèle lancé. Numéros de ligne = état du worktree lu le 2026-10-10.

## Objections R1 — toutes closes

| Objection | Verdict | Preuve lue |
|---|---|---|
| D1 transfert de tâche (majeur) | **Close** | `can_own_task` (`native_delegation.rs:105-139`) : présence `connected`/`busy` **ou** connexion primaire non-auxiliaire vivante de l'instance d'origine bloque rejeu, `Status` et `Cancel`. `Status` ne mute plus. Le tick passe par `recover_owner`. Test `native148_live_original_owner_blocks_replay_status_and_cancel` : les trois chemins refusés, puis connexion primaire seule suffit, puis reprise après départ réel. `owner_instance` inchangé pendant les refus. |
| D2 révocation perdure (moyen) | **Close** | Tombstone `native_delegation_revocations(agent_id)` consulté en tête de `root_permission` (`native_delegation.rs:62`), avant grant d'instance, héritage managed et attestation projet. `grant_agent` (`delegation.rs:214-241`) ferme d'abord le droit stable, puis l'instance. Test : réouverture du store (durable), nouvelle instance + attestation projet + grant d'instance restent fermés ; seul `grant_agent` humain lève. Le puits Grant exige rôle Client + acteur contrôle humain + ControlStateV1. |
| D3 lecture inter-conversations | **Close** | Même garde ; `Status` devient lecture pure (`native_delegation.rs` commentaire explicite). |
| D4 zombie si capture échoue | **Close** | `capture_reply` retourne `Result`. `Err` → NACK `native_result_not_persisted` dans `SendIdempotent` (`daemon.rs:~8647`) et `Send` (`daemon.rs:~17415`), tous deux **avant** `prepare_dispatch` : aucune livraison directe. Test `native148_capture_database_failure_never_acks_or_delivers_directly` : trigger SQLite d'échec d'UPDATE, puis DROP TABLE ; aucun ACK, aucun octet émis sur le socket, tâche `working`, résultat absent. |
| Observation « completed sans reply → working à vie » | **Close** | `MISSION_REPLY_LIMIT_SECS = 3600`. Échéance figée à l'admission via `get_or_insert` (jamais renouvelée au rejeu, lignes ~698/827), recalculée durablement au tick si absente, expiration → `mission_reply_timeout` + cleanup. Visible au catalogue (`mission_reply_limit_secs`) et au reçu (`mission_deadline_at`). Test horloge dédié. |

## Intégrations demandées — vérifiées

- **Wrapper 20 outils** : `BRIDGET_SAFE_MCP_TOOLS` = 20 entrées exactes (`wrapper.rs:4750-4771`), sans wildcard, `guichet_*` exclu. Deux usages cohérents : Claude `--allowedTools` préfixé `mcp__bridget__`, Codex `tools={...approve}` + `default_tools_approval_mode="prompt"`. Tests : spec091 octet-pour-octet, spec094, `native148_claude_allows...` (20, 4 nouveaux présents, pas de `*`). Restriction sous-agent interne who/send conservée (`mcp.rs:365-377` via `delegated_origin`) ; l'enfant natif n'est pas marqué `delegated_origin`, donc il garde `bridget_delegate` pour la profondeur 8.
- **Autres durcissements** : résolution Git hors verrou avant `state.lock`, comparaison au puits dans `authorize_cwd` — `SameProject` exige maintenant l'égalité exacte de projet (plus strict qu'un `starts_with`) ; cleanup/annulation sans preuve retour parent (test dédié) ; notices failed indexées sur `$.owner` (index SQL dédié) ; `parent_execution_id` depuis les exécutions actives seulement, `parent_execution_ambiguous` si >1 (`execution_store.rs:1456`, LIMIT 2).
- **T3 `applyFlagSettings`** : appliqué seulement après montage, statut `connected` relu, et présence vérifiée de tous les outils y compris les 2 lectures. Vieux catalogue sans les 2 outils → `catalogue_unavailable` → candidat fermé (`ClaudeAdapterV2.open`, onError → close) : pas de règles sur outils absents, pas de montage dégradé silencieux. Fusion `claudeBridgetReadOnlySettings` : préserve deny/ask/defaultMode/allow, ajoute seulement `capabilities`+`task_status`, aucun mutant ni wildcard ; settings en chemin de fichier → `undefined` → aucune application. Opt-out disabled/custom/revoked/explicit → zéro application, zéro appel. Aucun symbole nouveau côté Codex (grep).
- **Docs** : corrigées. `delegate-grant` référencé (`commandes.md:51`, `delegation-native.md`) ; exemple `glm` avec note « `claude_glm` n'est pas un type du registre » ; motif development distingué (`development_protocol_unavailable` au catalogue vs message registre à l'appel) ; échéance et révocation stable documentées (`contracts/delegation.md`) ; plus aucun « 16 outils » dans les guides.

## Nouveaux défauts

Aucun bloquant ni majeur. Deux notes :

1. **INFO** — Un résultat enfant invalide (corps vide ou >256 Kio) produit le NACK `native_result_not_persisted` alors que la cause réelle est `native_result_invalid` (logguée seulement). Fail-closed, échéance bornée. Libellé trompeur pour l'enfant, sans effet de sécurité.
2. **NOTE** — Le worktree Bridget évoluait pendant ma lecture (`native_delegation.rs` passé de ~2100 à 2324 lignes ; introduction de `ParentPermission` et d'un test `attested_git_discovery`). J'ai relu les sections critiques dans l'état final : toutes les gardes validées y sont inchangées ou renforcées.

## Manques de preuve (distincts des défauts)

1. **Recette GLM réelle non exécutée** : `native_delegation_real_glm.rs` existe, `#[ignore]` + `BRIDGET_148_REAL_GLM=1` requis, parent fixture Codex synthétique annoncé dans l'en-tête. Aucun résultat d'exécution réel n'est disponible dans ce que j'ai lu. Aucune preuve d'un modèle réel de bout en bout n'existe donc — je n'en fabrique pas.
2. **Tests non exécutés par moi** (contrainte lecture seule) : mes conclusions comportementales viennent du code et des tests lus, pas d'un passage réel.
3. **« Codex approval/sandbox inchangés »** : prouvé par absence des nouveaux symboles dans les chemins Codex et lecture du montage existant. Pas de diff Git (interdit). Preuve partielle.
4. **Test CLI T3 réel** (`BRIDGET_147_CLAUDE_METADATA_COMMAND`) : skippé sans la variable, non exécuté.
5. **Interop réseau identité HTTP** : annoncé en cours, hors de ce périmètre.

## Verdict : APPROVE

Les quatre objections R1 sont closes, chacune avec garde en code et test ciblé lu. L'intégration wrapper/T3 est conforme au brief. Les manques ci-dessus relèvent de l'exécution, pas du code lu.
