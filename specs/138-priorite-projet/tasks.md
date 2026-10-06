# Tâches 138 — Priorité au projet, périmètre complet

Date: 2026-10-06
Statut: Implemented ; 20/20 tâches terminées. Vérification, Converge pass1, audit validé et remise documentaire terminés.
Prérequis: audit de réutilisation PASS, 18/18 items et cinq critères cochés.
Le principal exécute Analyze avant l'implémentation.

## Propriétaires et règles communes

Noyau: protocol/message/communication/daemon/threads, leurs tests et le harnais
Rust existant. Façades: cli/mcp/communication-client/wrapper/T3 et leurs tests.
Agent Loop: sources, tests et skill dans le worktree dotfiles isolé. Principal:
Gherkin138, documentation, journal, preuves, Analyze, Converge et audit.

Les propriétaires ne modifient pas les fichiers d'un autre groupe sans transfert
explicite. Ils conservent les éditions existantes. Le noyau définit le contrat.
Les façades le consomment sans recopier la règle de portée.

Tous les essais utilisent un BRIDGET_HOME, une socket et un target Cargo isolés.
Le harnais parle à un vrai daemon temporaire avec des transports locaux. Les
assertions portent sur réponses publiques, contenu exact, store et effets remis.
Aucun fournisseur, commit, installation globale ou restart production.

Preuves communes: sortie RED avant code, sortie GREEN après correction, commande
exacte, code de sortie et assertions. Les conserver dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence.

## Préparation

- [x] T001 Principal — Écrire les quatre scénarios utilisateur et leurs cas limites dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/tests/features/138-priorite-projet.feature avant le code ; relier chaque scénario aux FR-13801–14 et SC-13801–06. Preuve: feature lisible avec local/global/inconnu, exception volontaire, boucles et compatibilité.
- [x] T002 Principal — Vérifier la sélection138, le PASS de réutilisation et les sorties isolées ; démarrer le journal dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/implementation.md et consigner Analyze initial dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/analysis-report.md. Preuve: aucun CRITICAL ouvert, chemins absolus et baseline de tests conservés ; FR-13814.

## US1 — Annuaire et suggestions par projet

But: proposer les agents locaux, permettre la vue globale volontaire et rendre
l'inconnu visible. Test indépendant: deux projets et un agent inconnu ; vérifier
les UUID réellement rendus, puis le diagnostic global historique.

- [x] T003 [US1] Noyau — Ajouter d'abord les tests RED de fait projet, annonce propriétaire, auxiliaire, identité forgée et annuaire same/global/unknown dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/spec102_threads_test.rs ; conserver ListAgents global. Preuve: échanges avec le vrai daemon, zéro candidat local si l'émetteur est inconnu ; FR-13801–04, SC-13801/05/06.
- [x] T004 [US1] Noyau — Étendre /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-transport/src/protocol.rs, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/communication.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs : CommunicationProjectFact distinct de Register, fait de connexion vivant, comparaison hôte/racine commune Git, DirectoryScoped et réponse ScopedAgentInfo aplatie distincte du diagnostic. Inclure le contexte Client autonome validé depuis racine/hôte, sans Register d'agent ni identité T3 empruntée ; il ne remplace jamais un fait auxiliaire parent. Preuve: T003 GREEN, annonce forgée refusée, fait supprimé à la fin de connexion et contexte background exploitable ; FR-13801–04/14, SC-13801/05.
- [x] T005 [US1] Façades — Ajouter les tests RED des annonces T3/wrapper et de who/agents local ou --global dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/cli.rs, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/mcp.rs, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/wrapper.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/t3code.rs. Preuve: contrats caller vérifiés, reconnexion et workspace attesté couverts ; FR-13801–04/13, SC-13801/05/06.
- [x] T006 [US1] Façades — Étendre les chemins de T005 ainsi que /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/t3code_contract.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/communication/client.rs pour annoncer le fait, consommer l'annuaire scoped et rendre l'inconnu. Preuve: T005 GREEN et matrice réelle CLI/MCP du daemon ; aucun repli extérieur, domain reste cosmétique ; FR-13801–04/13, SC-13801/05/06.

## US2 — Échange interprojets volontaire

But: exiger un motif pour une audience extérieure connue. Test indépendant:
envoi et fil mixtes avec puis sans motif, dont notify=[] ; comparer dépôt,
notification et corps. Une réponse corrélée réutilise la demande OPEN acceptée.

- [x] T007 [US2] Noyau — Ajouter les tests RED du motif borné UTF-8, None legacy, null invalide, canon de message et d'opération de fil, réponse OPEN inversée et in_reply_to forgée dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/communication.rs, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/threads.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/spec102_threads_test.rs. Preuve: fil mixte silencieux refusé sans motif avant toute écriture ; unknown legacy accepté avec warning ; FR-13805–08/11–13, SC-13802/03/06.
- [x] T008 [US2] Noyau — Ajouter cross_project_reason et son canon dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-core/src/message.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-transport/src/protocol.rs ; étendre la garde commune, prepare_dispatch et create/post dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/communication.rs, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/threads.rs. Preuve: T007 GREEN, motif par opération sur tous les lecteurs, warning caller établi avant effet hors corps, zéro table ou colonne de fil ; FR-13805–08/14, SC-13802/03/06.
- [x] T009 [US2] Façades — Ajouter les tests RED de --cross-project-reason, champ MCP, warning émetteur et serveur ancien sans capacité dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/cli.rs, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/mcp.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/communication/client.rs. Preuve: refus explicite de capacité sans downgrade ni envoi ; valeur vide et trop longue refusées ; FR-13805/06/13, SC-13802/03/06.
- [x] T010 [US2] Façades — Étendre les fichiers de T009 pour porter le motif, les avertissements caller et la négociation compatible dans les envois, fils et clients fédérés ; ajouter les aides des options. Preuve: T009 GREEN et vrai daemon temporaire confirmant parité CLI/MCP, réponse autorisée sans dialogue humain supplémentaire et corps byte-identique ; FR-13805–08/13, SC-13802/03/06.

## US3 — Boucles dans leur mandat

Complément des façades T009/T010: tester puis étendre
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/handoff.rs
et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/attach.rs.
parse_request/HandoffTransport et JournalRequest portent le motif structuré
jusqu'à la garde daemon commune. Vérifier le refus sans motif, la transmission
du motif valide et le warning émetteur hors corps. Aucun second moteur de portée.
Ces assertions appartiennent à US2, FR-13805/06/13 et SC-13802/03/06.

But: aucun recrutement extérieur automatique ; conserver les rappels des agents
déjà mandatés. Test indépendant: absence locale, local busy, ROOT extérieur sans
mandat, puis worker/coordinator/ROOT explicitement mandatés sur plusieurs ticks.

- [x] T011 [P] [US3] Agent Loop — Ajouter avant code les tests RED dans /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/tests/test_agent_loop.py et /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/tests/test_135_systemic_notifications.py : projet du run distinct de domain, absence/busy sans repli, ROOT sans mandat produit une décision, trois rôles mandatés gardent leurs rappels. Simuler un échec d'envoi, modifier le mandat du run, puis vérifier que le rejeu transporte le motif/source/destinataire anciens figés. Preuve: commandes de transport structurées et outbox inchangée lors de cette reprise ; FR-13807/09/10/12/14, SC-13804/06.
- [x] T012 [US3] Agent Loop — Étendre /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/scripts/agent_loop.py : annuaire scoped, preuve du projet à attach/dispatch, mandat motif par agent/rôle, envoi background et outbox avec motif/source figés. Preuve: T011 GREEN ; renommer domain ou changer une configuration après mise en file ne modifie pas le mandat de cette émission ; aucun nouveau service ou migration de mission ; FR-13807/09/10/12/14, SC-13804/06.
- [x] T013 [US3] Agent Loop — Mettre à jour /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/SKILL.md ; vérifier que /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/claude/.claude/skills/agent-loop/scripts/agent_loop.py délègue au canon existant plutôt que dupliquer la règle. Preuve: copie Claude validée, 124 tests existants plus nouveaux exécutés avec total réel consigné, skill vérifiée par /Users/moi/.codex/skills/.system/skill-creator/scripts/quick_validate.py ; skill vivante non installée ; FR-13809/10/13/14, SC-13804/06.

## US4 — Identités robustes et compatibilité historique

But: préserver les reçus, envois acceptés et historiques lorsque les faits de
projet changent. Test indépendant: worktree/symlink/homonymes, ancienne opération
acceptée puis changement de fait et restart, fil136 silencieux et client legacy.

- [x] T014 [US4] Noyau — Ajouter d'abord les tests RED de reçu antérieur après changement de faits et redémarrage, opération ancienne acceptée encore en remise, canon legacy, message/fils à motif différent sous même clé et historique silencieux136 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/spec102_threads_test.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/idempotency_test.rs. Inclure un fil A/B/U : U inconnu ne supprime jamais l'exigence de motif pour B connu extérieur. Preuve: vrai daemon temporaire repris, zéro nouveau dépôt/notification et ancien résultat exact ; FR-13803/04/05/08/11–14, SC-13802/05/06.
- [x] T015 [US4] Noyau — Ajuster uniquement les chemins existants de reprise, reçu et remise dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/threads.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/store/threads.rs afin de rendre le résultat déjà durable avant une nouvelle garde. Preuve: T014 GREEN, messages et historique exacts, opérations acceptées conservées, aucun balayage de file ; FR-13811/12/14, SC-13806.

## Vérification complète et remise au principal

- [x] T020 [US3] Agent Loop — Corriger le finding P1 de publication entre la réservation CAS et l'archivage dans /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/scripts/agent_loop.py et ses tests. Ajouter d'abord le test RED qui impose cet interleave réel ; protéger publication et archive par le verrou commun existant afin que le résultat publié conserve son chemin canonique. Preuve: RED puis GREEN avec contrôle du fichier, du result_path durable et du verdict, aucune perte ni déplacement de la publication concurrente. Inclure une panne E/S ordinaire après archive et avant écriture de tâche, ainsi qu'une publication échouée avec ancien document présent ou absent : vérifier restauration exacte avant commit et warning fermé du journal après commit. La borne couvre la remise initiale existing_bridget et write_task_result, pas les writers externes sans verrou ni un crash disque ou rollback lui-même défaillant. FR-13812/14, SC-13806. Dépend de T012, bloque T017 ; T012 garde sa clôture historique validée, T020 clôturée sur autorisation du principal : 152 tests PASS exit0 et revue corrective indépendante APPROVE ; preuve dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/python-final-validation.log.
- [x] T019 [US2] Noyau daemon/protocol et propriétaire execution_store — Ajouter d'abord les tests RED de SteerCurrent.message interprojets dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/execution_store.rs. Puis réutiliser la garde de portée commune avant injection et étendre le résultat dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-transport/src/protocol.rs. Conserver les warnings dans une seule colonne JSON project_warnings de execution_control_commands, avec migration compatible selon init_schema et les helpers d'introspection existants. Preuve: sans motif aucun contenu extérieur injecté ; warning caller hors corps établi avant effet ; reprise exacte après changement des faits et restart rend le résultat accepté avant la garde mutable sans réinjecter. Les anciennes lignes rendent [], le canon exact et refusal_reason ne portent jamais les warnings ; Interrupt sans message et canon legacy inchangés. Vrai daemon temporaire et store réel, RED puis GREEN consignés ; FR-13805/06/12/13/14, SC-13802/03/06. Dépend de T008 ; clôturée sur autorisation du principal après lecture des sources/tests, des RED garde/store et des GREEN bibliothèque25 PASS, store17 PASS et contrôle processus réel1 PASS avec reprise durable, sans validation globale implicite.
- [x] T016 [P] Principal — Compléter /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/skills/bridget/SKILL.md et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/docs/reference-communication.md ; rapprocher exemples et contrat dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/contracts/communication.md. Preuve: priorité locale, vue globale distincte du mandat, motif volontaire, inconnus avertis, tous les membres lecteurs et absence de seconde confirmation humaine clairement expliqués ; FR-13801–14, SC-13801–06.
- [x] T017 Principal — Exécuter les tests ciblés138 puis cargo test --workspace, cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings et cargo build --release --workspace dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet avec target isolé ; lancer unittest complet et validation de la skill dans /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop. Preuve: logs complets, comptes et codes de sortie dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence ; aucun test fournisseur ; SC-13801–06, FR-13814. Clôturée par le principal : workspace V5 exit0, 1633 PASS/0 FAIL/55 ignorés ; Python152 PASS ; recette opt-in réelle138 PASS ; fmt/clippy/release après dernières fixtures exit0 ; deux skills valides. Les captures rouges antérieures restent conservées.
- [x] T018 Principal — Demander la lecture adverse disponible, relier chaque FR/SC au code et aux tests réels, exécuter Analyze final, Converge et audit du périmètre dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet ; compléter /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/implementation.md et les preuves. Preuve: findings traités ou blocages nommés, charge future et complexité examinées, statut fidèle aux tâches restantes, aucun commit/install/restart ; FR-13801–14, SC-13801–06. Clôturée après Analyze final PASS, Converge pass1 CONVERGED (18:52:52–18:56:19 UTC, SHA tasks inchangé), audit A98,333 borné au diff avec 0C/H et5MED ouverts non bloquants, validator officiel exit0/0erreur/0warning et journal/résultats à jour. Aucun Converge2 ni déploiement revendiqué.

## Dépendances et possibilités de travail parallèle

T001 → T002 ouvre les travaux. US1: T003 → T004 → T005 → T006.
US2: T007 → T008 → T009 → T010 après T004. US3: T011 → T012 → T013 ; T011
peut commencer après T002, T012 attend le contrat noyau T008 et le client T010.
US4: T014 → T015 après T008. T016 attend le contrat stabilisé et peut se dérouler
pendant T014/T015. T019 suit T008 et son Analyze complémentaire PASS précède le code de contrôle. T020 suit T012 et sa preuve corrective. T017 attend T006/T010/T013/T015/T016/T019/T020. T018 termine la session.

Exemples sans conflit: pendant les tests noyau T003 ou T007, Agent Loop écrit
T011 dans son dépôt isolé. Pendant T014/T015, le principal écrit T016. Les façades
peuvent avancer leurs propres tests après publication du contrat noyau. Ne pas
faire écrire simultanément deux propriétaires dans le même fichier ou target.

## Stratégie et charge future

Livrer les quatre histoires avant de déclarer le périmètre terminé. US1 est une
tranche vérifiable, pas un arrêt autorisé du pipeline. Chaque correction reçoit
son test de comportement avant de passer à la suite. Les tests RED puis GREEN
sont consignés ; aucune case ne se coche sur une intention seule.

T004/T008 réduisent la duplication grâce au fait vivant et à la garde commune.
T006/T010 rendent les façades explicables. T012 conserve les décisions d'émission
de façon stable pour les relances. T015/T017 protègent la compatibilité. Les
nouveaux contrats sont justifiés par l'autorité et les diagnostics à préserver.
Aucune dépendance ou abstraction de politique générale n'est ajoutée.
