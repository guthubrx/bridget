# Archives techniques de l’adaptateur T3

Document d’origine : [specs/138-priorite-projet/reuse-audit.md](../../../../../../specs/138-priorite-projet/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| 1. Fait projet et racine commune | types partagés et racine T3 existante | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/t3code_contract.rs:243 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/wrapper.rs:926 | Nouveau fait de communication : aucun équivalent fiable à domain ; résoudre Git au rattachement |
| 2. Annonce et autorité de connexion | live_connection_identity/register_auxiliary | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs:7833 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs:7891 | Annonce séparée de Register ; auxiliaire hérite de la preuve vivante |
| 3. Annuaire scoped | projection ListAgents et AgentInfo | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-transport/src/protocol.rs:2748 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs:13667 | Nouvelle variante justifiée ; diagnostic global et type historique conservés |
| 4. Garde directe | prepare_dispatch | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs:7977 | Prévalidation commune O(1), avant dépôt ou sollicitation |
| 5. Motif et canon | BridgetMessage/canonical_send/ThreadRequest | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-core/src/message.rs:80 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/communication.rs:64 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-transport/src/protocol.rs:2149 | Champ optionnel et suffixe canonique ; None historique exact |
| 6. Réponse et mandat corrélé | valid_tracked_reply dans prepare_dispatch | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs:7999 | Demande OPEN acceptée et participants inversés ; aucune table de consentement |
| 7. Audience de fil et canon | create/post/load_members/thread_post | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/threads.rs:348 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/threads.rs:485 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/store/threads.rs:915 | Contrôler les lecteurs, pas seulement notify ; motif par opération sans migration SQL |
| 8. Warning résultat | DaemonToWrapper/ThreadResult | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-transport/src/protocol.rs:3186 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-transport/src/protocol.rs:2284 | Métadonnée caller distincte du corps ; résultat durable de rejeu conservé |
| 9. Façade CLI | cmd_who/cmd_agents/résolveur/send | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/cli.rs:5252 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/cli.rs:4859 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/cli.rs:961 | Options du contrat commun ; cible explicite ne vaut pas suggestion automatique |
| 10. Façade MCP | execute_who/send/thread | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/mcp.rs:993 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/mcp.rs:654 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/mcp.rs:484 | Même motif et même verdict ; droits du sous-agent133 préservés |
| 11. Client et fédération | client Unix et contrats de communication existants | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/communication/client.rs:1 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/scripts/federate-ssh.sh:1 | Réutiliser échange et capacités ; pair sans preuve reste inconnu |
| 12. Projet du run et recrutement | bridget_agents/resolve_bridget_target/cmd_attach_agent | /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py:987 ; /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py:1025 ; /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py:1294 | Modifier la source dans le worktree dotfiles isolé ; ne pas utiliser run.domain comme preuve |
| 13. Mandats et rôles attribués | validate_authorized_recipient/send_bridget_message/cmd_dispatch | /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py:1136 ; /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py:1059 ; /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py:1875 | Ajouter motif borné par attribution aux contrôles existants ; pas d'orchestrateur parallèle |
| 14. Règle skill et aides | skill Bridget existante | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/skills/bridget/SKILL.md:1 | Compléter la règle de communication ; aucun nouveau paquet skill |
| 15. Harnais Rust/fils | tests102/136 et tests canoniques | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/communication.rs:107 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/store/threads.rs:1605 | Étendre le vrai store et les clients locaux, sans second harnais |
| 16. Acceptation Gherkin | conventions de features102/136 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/tests/features/102-fils-inter-agents.feature:1 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/tests/features/136-messages-utiles.feature:1 | Nouvelle feature138 nécessaire pour son comportement distinct |
| 17. Tests de skill | unittest Agent Loop et contrôle135 | /Users/moi/.codex/skills/agent-loop/tests/test_agent_loop.py:1 ; /Users/moi/.codex/skills/agent-loop/tests/test_135_systemic_notifications.py:1 | Étendre dans /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/tests ; aucune exécution fournisseur |
| 18. Consigne injectée et warning durable | handle_execution_control, reserve_control_command et init_schema | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs:8656 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/execution_store.rs:1084 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/execution_store.rs:1531 | Garde partagée avant SteerCurrent.message ; une colonne JSON project_warnings dans la table existante ; reçu accepté rendu avant garde mutable, canon/refusal_reason inchangés |

<a id="extrait-2"></a>

## Extrait 2

| Requete | Portee | Resultat |
|---|---|---|
| rg live_connection_identity/register_auxiliary/prepare_dispatch | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates | Autorité vivante et garde centrale retrouvées |
| rg ListAgents/Register/ThreadRequest/ThreadAction | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-transport | Diagnostic global séparé, points additifs du contrat |
| rg canonical_send/canonical_hash/load_members | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon | Rejeu et membres déjà centralisés |
| rg workspace_root/derive_domain_at | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon | Racine T3 disponible ; domaine mutable non probant |
| rg project: None et lecture refus runtime | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon | Ancien ProjectReference/runtime retiré confirmé |
| rg who/agents/resolve/send et client Unix | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon | Façades communes identifiées, pas nouveau client |
| rg fonctions agents/authorized/attach/dispatch/background | /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py | Attributions et envois de fond réutilisables |
| rg --files --hidden agent-loop | /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project | Sources Codex/Claude et tests isolés retrouvés |
| Lecture Cargo.toml et inventaire rg --files | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet | serde/rusqlite/SHA-256 existants ; pas nouvelle dépendance |
| rg --files specs/docs/tests | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet | Specs proches et tests102/136 ; ADR046 libre |
