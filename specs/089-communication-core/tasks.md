# Tâches 089 — extraction progressive du noyau

**Entrées :** spec.md, plan.md, research.md, data-model.md, contracts/communication.md.
**36 tâches de réalisation, 15 terminées.** P0 : référence mesurée et exclusions de sécurité consignées ; T005 ne signifie pas suite globale verte. Maicie/UI/runtime sont découplés ; stockage modularisé sans scinder les transactions, vérifié par fautes et crashs réels. Paquet indépendant T013 construit et exécuté (SC-08906), parité réelle CLI/MCP T014 et renommage CLI/identité T015 vérifiés ; la clôture du service reste à corriger dans T018. Pas de livraison globale ni de recette SSH déclarée.

Exécution séquentielle par tranches ; aucun lancement automatique de sous-agents. Un commit cohérent après tests, aucune capture d'un WIP tiers. Écrire l'oracle avant la modification qu'il doit protéger, vérifier son échec sur un mutant ciblé lorsqu'exigé, puis restaurer le vrai code. Les tests réels n'utilisent jamais les processus de production.

## P0 — Fondation vérifiable (bloque toute suppression)

- [x] T001 Inventorier les dépendances, tables, commandes, constantes de namespace et points d'appel à partir du commit source ; compléter specs/089-communication-core/baseline.md avec fichier:ligne et dispositions conserver/extraire/retirer. Inclure Cargo.toml, les trois manifests, daemon.rs/store.rs/wrapper.rs/cli.rs/mcp.rs et le protocole. Oracle : chaque module importé par lib.rs a une disposition, aucune entrée seulement supposée.
- [x] T002 Épingler les fixtures de toutes les familles de contracts/communication.md dans specs/089-communication-core/contracts/fixtures/ avec source Git et SHA-256 ; ajouter scripts/verify-089-contracts.sh. Oracle : octet altéré/source absente/commit non ancêtre refusés ; tester aussi dépôts/replies/événements, pas seulement les handshakes.
- [x] T003 Établir la liste des tests historiques retenus/retirés/déplacés dans specs/089-communication-core/test-map.md et les scénarios tests/features/089-communication-core.feature. Oracle : chaque SC-08901..12 a des tests nommés ; chaque retrait a une justification de périmètre, jamais « test rouge ».
- [x] T004 Compléter specs/089-communication-core/threat-model.md : UID/SSH/identité/capacités, limites, secrets, permissions fournisseur et sorties de connexion. Oracle : scénario hostile, autorité et refus attendu pour chaque frontière.
- [x] T005 Exécuter les tests retenus et mesurer la référence dans un environnement de test isolé, après audit des harnais ; consigner commandes/durées/échecs et ressources dans implementation.md. Aucun rouge préexistant caché, aucune exécution contre HOME/socket historiques. Référence collectée sur les suites auditées ; exclusions explicites reprises par les gates de réalisation et T034, pas comptées comme réussies.
- [x] T006 Relire spec/plan/contrats/test-map avant coupe ; proposer une revue indépendante et consigner le verdict dans checklists/extraction.md. Gate : absence de garantie perdue sans décision explicite.

## P1 — Découplage structurel (US4)

- [x] T007 [US4] Poser configuration indépendante de home/socket/instance dans crates/bridget-daemon/src/daemon.rs (DaemonConfig, dirs_cache, dirs_data), cli.rs et mcp_identity.rs, en réutilisant la résolution existante ; ajouter tests/core_089_isolation_test.rs. Oracle : défaut historique et namespace occupé refusés ; aucune connexion/écriture vers la flotte. runtime.rs représente les faits fournisseur et n'est pas le propriétaire de cette configuration.
- [x] T008 [US4] Sortir les helpers partagés de scope/canon/résolution des façades cli.rs/mcp.rs vers un module neutre crates/bridget-daemon/src/communication.rs, sans changer les octets ; garder un seul producteur. Oracle : tests du corpus et absence d'import de présentation par le store/daemon.
- [x] T009 [US4] Découpler maicie dans crates/bridget-daemon/Cargo.toml, src/lib.rs, mission_projection.rs et ses appelants ; conserver la frontière service publique sans import métier. Oracle : compilation sans plugins/maicie accessible et couture d'un consommateur externe.
- [x] T010 [US4] Retirer UI/desktop/rendering HTML du produit extrait via src/cli.rs, mcp.rs, lib.rs, ui.rs, artifact_* et apps/, uniquement après disposition T001/T003 ; préserver la publication/référence de contenu communicable, son contrôle d'accès et ses bytes sans renderer. Oracle : commandes retirées refusées, références utiles résolubles sans HTTP/UI, communication verte.
- [x] T011 [US4] Découpler runtime Docker/projet de src/daemon.rs, wrapper.rs, registry.rs et project_* ; retirer les implémentations hors périmètre, conserver droits/registre/capacités de session. Oracle : spawn/refus/stop sans Docker ; une permission absente reste refusée.
- [x] T012 [US4] Modulariser le stockage conservé dans src/store.rs et idempotency.rs sans scinder les transactions ACK/ledger/answered/événement ; isoler SQL hors périmètre selon T001. Oracle : atomicité par faute injectée et données historiques sur copies.
- [x] T013 [US4] Fermer Cargo.toml/Cargo.lock et exports des trois crates ; ajouter tests/core_089_dependency_test.rs. Oracle : métadonnées sans dépendance Maicie/UI/Docker, construction et exécution depuis un paquet ne contenant pas leurs sources. SC-08906.

## P2 — Usage local et garanties conservées (US1/US3)

- [x] T014 [US1] Ajouter tests/core_089_contract_test.rs : CLI et MCP réels → même daemon → même canon/record, refus de divergence body/cible/reply/deadline/in_reply_to sans mutation ; corriger uniquement la couture concernée. SC-08902.
- [x] T015 [US1] Ajouter tests/core_089_identity_test.rs : rename, reprise et instances distinctes de même binaire ; conserver issuer_scope stable et présence honnête dans src/mcp_identity.rs, identity_migration.rs et daemon.rs. SC-08901/09.
- [ ] T016 [US1] Ajouter tests/core_089_reply_test.rs : demande suivie → réponse liée → answered, ledger unique, zéro rappel après clôture, timeout/annulation attestés ; préserver helper transactionnel partagé. SC-08901.
- [ ] T017 [US1] Porter les crash-tests idempotence existants dans tests/core_089_crash_test.rs avec barrières, watchdog et comptage de prompts ; même retry/canon/issue, limites wrapper explicites. SC-08903, aucun SIGTERM présenté comme crash.
- [ ] T018 [US1] Ajouter tests/core_089_service_test.rs : consommateur public externe sans crate Maicie, dépôt/reprise/claim périmé/réponse/événement identique ; conserver capacité et quatuor du claim dans daemon.rs/store.rs. SC-08902/08.
- [ ] T019 [US3] Rejouer et compléter tests/core_089_attach_test.rs : raw inhabituel/champ inconnu intact, journal avant snapshot, SnapshotCaughtUp avant live, rotation et curseur périmé ; mutants de fraîcheur et de Gap refusés. SC-08905.
- [ ] T020 [US1] Rejouer les pilotes Codex/Claude natifs via src/wrapper.rs et bridget-transport/src/{codex_app_server,claude_stream_json,managed_session}.rs : échange réel, EOF en plein tour, groupe arrêté, attach et modèle/effort attestés. SC-08909.
- [ ] T021 [US1] Vérifier ACP et GLM via Claude Code sans tmux ni substitution vers API facturée ; tests/core_089_providers_test.rs et preuves expurgées dans implementation.md. Compte indisponible = gate non validé, jamais remplacement de fournisseur implicite. SC-08909.
- [ ] T022 [US3] Ajouter tests/core_089_ledger_test.rs : source unique src/ledger.rs et renderers CLI/MCP identiques au contrat ; golden binaire octet-pour-octet, données entrantes/sortantes et limites. SC-08902/10.
- [ ] T023 [US1] Livrer skills/bridget/SKILL.md, README.md et README.en.md centrés sur envoyer/répondre/consulter/retry ; tests/core_089_skill_test.rs pour in_reply_to et statuts distincts. Exécuter le scénario court sans Maicie ; ne pas modifier les skills globales. SC-08910.

## P3 — Fédération SSH obligatoire (US2)

- [ ] T024 [US2] Adapter scripts/federate-ssh.sh et deploy-remote.sh : chemins/labels explicites, dry-run et préflight sans écrasement de socket, client-only sans installation métier ; scripts/tests/federation_089_test.sh couvre cible occupée et options SSH sûres.
- [ ] T025 [US2] Ajouter tests/core_089_federation_test.rs avec serveur SSH de test isolé et transfert Unix réel : même protocole/annuaire/ledger, timeout et permissions ; ce test local ne remplace pas T026.
- [ ] T026 [US2] Exécuter la couture sur deux machines réelles et comptes autorisés, sockets indépendantes : demande/réponse/ledger/journal. Consigner commandes exactes et preuves dans implementation.md. SC-08904.
- [ ] T027 [US2] Couper/rétablir uniquement le tunnel de test : mêmes IDs et bytes, curseur préservé, Gap/Unavailable honnêtes, aucun double prompt ni redémarrage fournisseur. Compléter core_089_federation_test.rs et preuve distante. SC-08904/05.
- [ ] T028 [US2] Mesurer local et distant 10 événements/s sur 60 s ; p95/max/pertes/clock skew, budget d'append historique inchangé. Consigner méthode et résultats, puis nettoyer les seuls objets de test. SC-08912.

## P4 — Qualité, sécurité et adoption contrôlée (US4)

- [ ] T029 [US4] Ajouter tests/core_089_security_test.rs : permissions à la création, mauvais propriétaire, symlink, trame à la limite LF inclus, absence de capacité, scope usurpé et purge après BrokenPipe ; vérifier zéro effet interdit. SC-08908.
- [ ] T030 [US4] Ajouter tests/core_089_concurrency_test.rs : connexions réelles simultanées, saturation, budget avant connect, lenteur multi-phases, ouverture SQLite concurrente et retry même clé ; barrières bornées. SC-08903/08.
- [ ] T031 [US4] Ajouter tests/core_089_migration_test.rs : copies de schémas historiques, migration répétée et inconnue, IDs/bytes/terminaux inchangés, aucune écriture dans la source ; documenter import opt-in, sans l'exécuter sur données utilisateur. SC-08907.
- [ ] T032 [US4] Auditer Cargo.lock et scripts livrés : vulnérabilités, licences et dépendances réellement utilisées ; retirer les dépendances hors périmètre, documenter les alertes restantes dans implementation.md, aucune installation d'outil globale implicite. SC-08911.
- [ ] T033 [US4] Comparer graphe, modules de production, temps de build et mémoire/latence des scénarios de référence ; noter mesures reproductibles et régressions éventuelles dans baseline.md. SC-08912 ; la simple baisse du nombre de lignes n'est pas le verdict.
- [ ] T034 [US4] Exécuter fmt --check, clippy workspace/all-targets -D warnings et suite complète du workspace extrait sous watchdog ; rejouer les gates réels explicitement, contrôler toute disposition de test historique. SC-08911.
- [ ] T035 [US4] Finaliser README FR/EN, quickstart et guide d'installation indépendante/retour arrière ; corriger les limites de garantie et l'inventaire des commandes. Démontrer l'installation en espace de test neuf, pas dans la flotte.
- [ ] T036 [US4] Proposer revue adverse finale communication/sécurité et joindre son verdict à implementation.md ; vérifier SC-08901..12 avec preuves. Présenter livraison et procédure de bascule à l'utilisateur, sans déployer automatiquement.

## Dépendances et stratégie de livraison

P0 → P1 → P2 → P3 → P4. T007 précède tout essai de binaire extrait ; T002 et T003 précèdent toute suppression ; T009–T012 partagent les mêmes propriétaires et sont séquentiels. T026 nécessite l'accès réel à deux machines ; T020/T021 nécessitent les sessions autorisées. Aucun de ces gates n'est remplacé par un test simulé.

Le premier incrément démontrable est US1 + US3 après P2 ; il n'est pas la livraison complète, car SSH est obligatoire. La session ne démarre ni l'intégration T3 ni la greffière. Les lots sont des transactions Git petites, pas trente-six couches d'architecture.
