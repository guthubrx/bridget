# Archives techniques de l’adaptateur T3

Document d’origine : [specs/138-priorite-projet/tasks.md](../../../../../../specs/138-priorite-projet/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Noyau: protocol/message/communication/daemon/threads, leurs tests et le harnais
Rust existant. Façades: cli/mcp/communication-client/wrapper/T3 et leurs tests.
Agent Loop: sources, tests et skill dans le worktree dotfiles isolé. Principal:
Gherkin138, documentation, journal, preuves, Analyze, Converge et audit.

<a id="extrait-2"></a>

## Extrait 2

- [x] T003 [US1] Noyau — Ajouter d'abord les tests RED de fait projet, annonce propriétaire, auxiliaire, identité forgée et annuaire same/global/unknown dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/spec102_threads_test.rs ; conserver ListAgents global. Preuve: échanges avec le vrai daemon, zéro candidat local si l'émetteur est inconnu ; FR-13801–04, SC-13801/05/06.
- [x] T004 [US1] Noyau — Étendre /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-transport/src/protocol.rs, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/communication.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs : CommunicationProjectFact distinct de Register, fait de connexion vivant, comparaison hôte/racine commune Git, DirectoryScoped et réponse ScopedAgentInfo aplatie distincte du diagnostic. Inclure le contexte Client autonome validé depuis racine/hôte, sans Register d'agent ni identité T3 empruntée ; il ne remplace jamais un fait auxiliaire parent. Preuve: T003 GREEN, annonce forgée refusée, fait supprimé à la fin de connexion et contexte background exploitable ; FR-13801–04/14, SC-13801/05.
- [x] T005 [US1] Façades — Ajouter les tests RED des annonces adaptateur/wrapper et de who/agents local ou --global dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/cli.rs, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/mcp.rs, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/wrapper.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/t3code.rs. Preuve: contrats caller vérifiés, reconnexion et workspace attesté couverts ; FR-13801–04/13, SC-13801/05/06.
- [x] T006 [US1] Façades — Étendre les chemins de T005 ainsi que /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/t3code_contract.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/communication/client.rs pour annoncer le fait, consommer l'annuaire scoped et rendre l'inconnu. Preuve: T005 GREEN et matrice réelle CLI/MCP du daemon ; aucun repli extérieur, domain reste cosmétique ; FR-13801–04/13, SC-13801/05/06.
