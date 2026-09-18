# Tâches101 — Implemented

Base des chemins : racine du worktree101. Gate reuse-audit PASS lu avant création.
Toutes les tâches sont nécessaires ; pas d'arrêt après une tranche MVP.

## Préparation

- [x] T001 Écrire les scénarios métier dans tests/features/101-abonnements-t3.feature et documenter les recettes isolées ; vérifier leur correspondance FR001–010.

## US1 — Identité sûre

- [x] T002 [US1] Ajouter tests avant correctif dans crates/bridget-daemon/src/t3code_identity.rs : deux fils même cwd, sous-agent plus récent, curseur nul, plusieurs correspondances, PID recyclé, SQLite absent ; aucun rattachement deviné (FR001–002).
- [x] T003 [US1] Implémenter le rattachement dans crates/bridget-daemon/src/t3code_identity.rs en réutilisant crates/bridget-daemon/src/{runtime.rs,mcp_identity.rs,managed_process.rs}, l'appeler depuis t3code.rs ; marqueur privé atomique possédé, retiré sur perte/ambiguïté ; aucun changement de T3, aucune nouvelle dépendance (FR001–002).

## US2 — Faits T3

- [x] T004 [US2] Ajouter tests de contrats/corrélation dans crates/bridget-daemon/src/{t3code_contract.rs,t3code.rs} : fin sans texte, état inconnu, historique, tour de notification, permission, écriture réussie/échouée/lecture ; vérifier l'absence de faux faits (FR003–005).
- [x] T005 [US2] Raccorder les faits structurés au journal100 dans crates/bridget-daemon/src/{t3code.rs,t3code_contract.rs}, conserver déduplication après flush et origine système durable ; ne pas répondre automatiquement aux notifications système (FR003–005).

## US3 — Capacité et interruptions

- [x] T006 [US3] Étendre crates/bridget-transport/src/protocol.rs et crates/bridget-daemon/src/{daemon.rs,wrapper.rs,t3code.rs,observation.rs} avec annonce primaire de capacités, catalogue et validation de souscription ; tester agent inexistant, source incompatible, auxiliaire refusé et wrappers compatibles (FR006,FR008).
- [x] T007 [US3] Persister la trace bornée des abonnements dans Store et rétablir état interrompu après restart via crates/bridget-daemon/src/{store.rs,observation.rs,daemon.rs} ; TTL absolu et création/once/unsub cohérents, aucune sauvegarde pour fait non pertinent ; tests SQLite/restart/erreur de sauvegarde (FR007–008).
- [x] T008 [US3] Signaler perte/reprise de source et retour du propriétaire avec observation_output existant dans crates/bridget-daemon/src/{daemon.rs,t3code.rs,observation.rs} ; aucune E/S de notification sous verrou, DND respecté, pas de boucle (FR004,FR007–008).

## US4 — Parcours quotidiens et livraison vérifiable

- [x] T009 [US4] Ajouter une recette isolée bout en bout dans crates/bridget-daemon/tests/spec101_observation_test.rs réutilisant le support sûr100 : source → journal → notification, partage journal, collision, restart ; pas de SIGKILL ni service réel (FR009–010).
- [x] T010 [US4] Exécuter fmt, clippy, tests101/100/099/MCP/attach/protocole et documenter commandes/résultats dans specs/101-abonnements-t3/implementation.md ; adapter seulement fixtures affectées, ne pas masquer les erreurs (FR001–010).
- [x] T011 [US4] Mettre à jour README.md, skills/bridget/references/commandes.md et docs/decisions/037-observations-t3-attestees.md ; vérifier identité et notification dans un vrai fil T3 après accord d'adoption et sauvegarde, tracer la preuve dans implementation.md (FR009–010).

## Dépendances et parallélisme

T001 → T002 → T003 ; T004 → T005 ; T003/T005 → T006 → T007 → T008 → T009 → T010 → T011.
La préparation identité et les tests de projection sont indépendants, mais t3code.rs
reste la propriété du responsable d'intégration : aucun autre agent ne l'édite.
T002/T003 peuvent être confiées ensemble à un exécutant, tests en premier ; le
responsable intègre l'appel au module. T004/T005 restent locaux.

## Validation et limites de recette

Commandes prévues : cargo fmt --all -- --check ; cargo clippy --workspace
--all-targets -- -D warnings ; cargo test -p bridget-daemon --lib spec101 ;
cargo test -p bridget-daemon --test spec101_observation_test ; régressions
spec100, spec099, mcp::tests, attach::tests, communication::client::security_tests,
journal::tests et protocole. HOME/socket privés pour recettes ; variables du
daemon réel retirées des tests utilisant leurs propres fixtures.
Pas de cargo test --workspace aveugle : plusieurs anciens harnais SIGKILL sont
incompatibles avec la règle processus utilisateur. Déclarer cette exclusion.
T011 exige une preuve réelle et l'autorisation d'adoption101 ; sans celle-ci,
laisser cette tâche bloquée, pas de statut Implemented.

## Convergence

- [x] T012 [US1] Corriger deux écarts de rattachement révélés par la relecture indépendante dans crates/bridget-daemon/src/t3code_identity.rs : normaliser le provider_name réel claudeAgent vers claude via agent_type_for existant ; refuser toute collecte de rollout incomplète au lieu de masquer un processus potentiellement ambigu. Ajouter régressions avec alias réel et second processus partiellement illisible ; relancer tests identité (FR001–002).

- [x] T013 [US3] Rendre les lacunes de la file de faits consultables et notifiées via protocole primaire attesté et relais existant (wrapper.rs, protocol.rs, daemon.rs, observation.rs), sans fausse observation de fin. Réduire l'attente de verrou SQLite des seules sauvegardes d'abonnements (store.rs) en conservant refus/rollback et comportement des autres opérations. Tester saturation amont réelle, refus auxiliaire et contention réelle à deux connexions (FR005,FR007–008).
