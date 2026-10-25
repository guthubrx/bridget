# Tâches SPEC-163 — Décision initiale configurable

**Statut courant au 14 octobre 2026 :** Implemented, 58/58 (100 %) ; **Pipeline : Livré** avec conservations justifiées. G8/G9 et documents de livraison sont acceptés ROOT ; T045 clôturée sur le commit documentaire c57cfd35efa1338cda5256a02c57d71258d8b8d4 poussé et vérifié. Source active 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5 inchangée ; les marqueurs de clôture 58/58 sont intégrés et poussés dans af8998a7d7494913d4420e2dca7354a73bd8a4f5, github/main égal HEAD propre vérifié par ROOT. La source active reste 3efa, distincte de ce commit documentaire.

## Autorités, chemins et preuves

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-1).

Les développeurs ne modifient ni main, ni worker actif 471321, ni socket/état/DB/credentials de production. ROOT possède Git, intégration/push, builds de livraison, installation/activation/recette et nettoyage. Aucun agent ne révoque les edits d’autrui. Les sources Linux/Mac ont le même commit SHA attesté.

Chaque tâche reste non cochée jusqu’à preuve observable de ses résultats et contrôles adaptés. T001–T003 sont cochées sur preuves préparatoires ROOT ; T004 sur gel wire et replay Linux transmis par ROOT. Aucune preuve de consommateur ou de code futur anticipée. Les tâches de tests explicites ci-dessous sont groupées par surface. Un même résultat peut valider plusieurs gardes prévues ; ne pas relancer une suite longue par tâche. Une compilation active à la fois dans toute la flotte, jobs 2/nice 10/tests 2. FULL finale unique seulement si gate applicable prévu. Les correctifs rejouent uniquement les contrôles affectés.

## Fallback Tasks et Analyze

/Users/moi/.agents/skills/speckit-tasks/SKILL.md et /Users/moi/.agents/skills/speckit-analyze/SKILL.md relus. setup-tasks.sh a été tenté par ROOT avec RC127 ; absence recontrôlée. Extensions et template tasks absents ; aucun hook présent ni outil installé. check-prerequisites.sh --json --require-tasks --include-tasks tenté une fois ici : RC127, absent. Fallback manuel documenté : documents requis présents, constitution projet et globale relues, génération structurée puis analyse en lecture seule, autofix explicitement autorisé par ROOT puis seconde relecture. Les hooks avant/après n’existent pas.

## Phase 1 — Préparation et isolation

ROOT atteste les espaces et responsabilités avant le code.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-2).

Wire et primitives de reçus/transaction figés avant consommateurs.

- [x] T004 **DEV_CARTAE163** — Définir et figer les types wire auto_delegate_v1, CandidateProbe, Hello/AcceptedWithCapabilities et Lineage v3 ; confirmer noms/champs/refus avec ROOT et transmettre exemples sérialisés aux consommateurs avant leur code. V1/v2 inchangés. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-transport/src/protocol.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-transport/src/lineage_protocol.rs ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/contracts/auto-admission.md. Couverture : FR-019, FR-020, SC-007, O4. Dépendance : T003.
- [x] T005 **DEV_CARTAE163** — Créer la migration additive et les types du reçu auto dans la SQLite existante ; conserver owner durable, unicités historiques, canon 512 Kio et métadonnées 16 Kio distincts, états/époque/révision. Ne créer aucune base ou service. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation.rs. Couverture : FR-013, FR-014, FR-021, SC-005. Dépendance : T004.
- [x] T006 **DEV_CARTAE163** — Extraire écriture de projection dans transaction fournie et publication après commit ; rendre disponible le puits Immediate Task/lien/collision sans transaction imbriquée ni notification sur rollback. Réemployer les primitives existantes. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation_lineage.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation.rs. Couverture : FR-013, FR-015, FR-016, SC-005, O5. Dépendance : T005.
## US1 — Régler les automatismes (P1)

Validation indépendante : les quatre combinaisons respectent les axes et zéro réseau absent/off.

- [x] T007 [US1] **DEV_CARTAE163** — Définir decision_services et admission_policy optionnels stricts ; axes false par défaut, profils1–8/références/bornes, hébergement distinct du placement et endpoint jamais issu de la mission. Conserver la racine historique tolérante. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/registry.rs. Couverture : FR-001, FR-002, FR-003, FR-006, FR-008, SC-001, SC-002. Dépendance : T006.
- [x] T008 [US1] **DEV_CARTAE163** — Brancher lecture fraîche de politique par admission puis snapshot figé ; filtrer les quatre combinaisons d’axes sans assouplir contraintes et sans watcher. Axe fixe sans profil suffisant refuse. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs. Couverture : FR-001, FR-003, FR-005, FR-014, SC-001, SC-003. Dépendance : T007.
- [x] T009 [US1] **DEV_CARTAE163** — Étendre les tests existants du registre et de l’admission pour les quatre combinaisons, champs inconnus, limites/références et politique modifiée après reçu ; observer refus et zéro réseau désactivé. Exécuter cette campagne regroupée. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/registry.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs. Couverture : FR-001, FR-002, FR-003, FR-005, FR-008, FR-014, SC-001, SC-002. Dépendance : T008.
- [x] T010 [US1] **ROOT** — Relire configuration et exemples sans secret ; rapprocher les résultats de T009 des axes utilisateur et du gate d’activation encore fermé. Documenter les limites sans choisir implicitement une configuration de production. Chemins : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/quickstart.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/implementation.md. Couverture : FR-001, FR-002, SC-001, SC-009. Dépendance : T009.
## US2 — Choix autorisé sans effets du probe (P1)

Validation indépendante : droits/mapping/ancien worker et zéro worktree/fournisseur/slot avant choix.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-3).

Validation indépendante : bornes/cooldown/2s/permis réel et repli autorisé sans multiplication de threads.

- [x] T017 [US3] **DEV_CARTAE163** — Implémenter module frontière minreq et garde MINREQ_TIMEOUT présent même vide/invalide au démarrage/préappel ; service seul non activable, daemon/Delegate explicite inchangés. URLconfig loopback/HTTPSrustls validé, redirects0, send_lazy corps8Kio+1/en-têtes8Kio/statut1Kio. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/decision_service.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/Cargo.toml. Couverture : FR-002, FR-003, FR-009, FR-010, FR-011, FR-012, FR-021, SC-002, SC-004, O1. Dépendance : T016.
- [x] T018 [US3] **DEV_CARTAE163** — Réemployer primitives de thread/synchronisation pour un seul travail réseau sans file, sans with_timeout ; deadline caller2s, permis jusqu’au vrai retour. DNS/connexion/TLS/écriture/lecture bloqués : suivants repli/refus immédiat et raison visible, aucun thread de remplacement. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/decision_service.rs. Couverture : FR-011, FR-012, FR-017, SC-004, O1. Dépendance : T017.
- [x] T019 [US3] **DEV_CARTAE163** — Ajouter santé à la demande et cache borné par service : positif30s, négatif30s configurable1–300 ; budget2s partagé. Aucun polling/découverte/démarrage/HTTP pour absent, désactivé ou candidat unique. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/decision_service.rs. Couverture : FR-003, FR-004, FR-008, FR-011, SC-002, SC-004. Dépendance : T018.
- [x] T020 [US3] **DEV_CARTAE163** — Implémenter adaptateur laya_systemone_v1 étroit : brief explicite/critères/candidats opaques seuls, réponse fermée et raison256octets ; bornes JSON4096/state1536/descriptions32/huit IDs/max_len1024/head512 et routing.model/usage contrôlés. Pas adaptateur Jev inventé. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/decision_service.rs ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/contracts/decision-service.md. Couverture : FR-009, FR-010, FR-017, SC-003, SC-008. Dépendance : T019.
- [x] T021 [US3] **DEV_CARTAE163** — Étendre les tests du module avec serveur HTTP contrôlé dans harness existant : service absent/off/unique, TTL de santé/délai de reprise, MINREQ_TIMEOUT, délai caller, permis encore occupé, réponse tardive avec CAS, corps/en-têtes/statut excessifs et choix invalide ; vrais compteurs, pas assertions sur mocks décoratifs. Exécuter regroupé. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/decision_service.rs. Couverture : FR-003, FR-004, FR-009, FR-010, FR-011, FR-012, FR-017, FR-021, SC-002, SC-004, SC-005, O1. Dépendance : T020.
- [x] T022 [US3] **ROOT** — Relire limite réseau et normalisation à partir des résultats ; exposer l’absence de garantie d’arrêt distant et la capacité occupée. Garder automatismes désactivés tant que recette depuis maître Mac et profils ne sont pas attestés. Chemins : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/research.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/implementation.md. Couverture : FR-010, FR-012, SC-004, SC-008. Dépendance : T021.
## US4 — Choix durable et protections (P1)

Validation indépendante : concurrence/crash/collision/purge/entretien autonome et un lancement au plus.

- [x] T023 [US4] **DEV_CARTAE163** — Implémenter réservation pending avec canon/politique figés, CAS epoch/revision et decision_started avant HTTP ; concurrent admission_pending sans deuxième appel/file. chosen persiste tuple/raison/mode/canon Delegate avant effets ; réponse tardive ne remplace aucun repli. Au point commun finalise_auto_choice, vérifier la règle de mode figée dans le contrat : axes tous deux désactivés et tuple fixe réel validé → Explicit ; repli → Fallback ; hors repli, au moins un axe automatique, même candidat unique → Automatic. Correction R3 mise en œuvre et contrôles ciblés attestés ; rapprochement final ROOT et gates de livraison ouverts. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs. Couverture : FR-013, FR-014, FR-015, FR-017, SC-005, O2. Dépendance : T022.
- [x] T024 [US4] **DEV_CARTAE163** — Raccorder choisi au puits162 sous autorité interne non forgeable. Natif externe vérifie reçu auto avant by_agent_request sous Immediate ; Task + lien atomiques/publication après commit. Refus chosen terminal auto même task_limit/pause/retention_saturated, sans native_refusal même clé ni élargissement des cinq codes. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs. Couverture : FR-013, FR-014, FR-015, FR-016, FR-017, SC-003, SC-005, O5. Dépendance : T023.
- [x] T025 [US4] **DEV_CARTAE163** — Étendre entretien existant : premier passage après réconciliation traite tous les reçus d’une époque précédente par lots/CAS/E/S hors mutex sans demandeur ni HTTP. pending probes figés→unique/repli/refus ; started→repli/refus ; chosen→admissionidempotente. Erreurs techniques/SQL visibles nonrésolues réentretenues, aucun lancement si droits/parent indisponibles. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation.rs. Couverture : FR-015, FR-016, FR-018, SC-003, SC-005, SC-006, O2. Dépendance : T024.
- [x] T026 [US4] **DEV_CARTAE163** — Adapter compteurs/pression/collecte à UNION4096 sans doublecompte reçu/Task. Purger seulement refus auto terminaux sans Task ancien→récent avec tombstone30j atomique, avant refus au plafond natif/auto ; replayexact avant purge puisrequest_expired. Garder pending/chosen/ambigu/vivant, refus natifs horscompteur et cyclearbre 7 j existants. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation_lineage.rs. Couverture : FR-014, FR-021, SC-005, O3. Dépendance : T025.
- [x] T027 [US4] **DEV_CARTAE163** — Étendre tests durables store/admission pour crashs aux frontières, concurrence/collisions, politique changée, entretien sans rejeu, erreur SQL visible, refus après chosen, purge/tombstone et reprise152/160 sans redécision. Observer HTTP/Task/lancement réels et rollback de publication ; exécuter campagne regroupée. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/tests/native_delegation_e2e.rs. Couverture : FR-013, FR-014, FR-015, FR-016, FR-017, FR-018, FR-021, SC-003, SC-005, SC-006, O2, O3, O5. Dépendance : T026.
- [x] T028 [US4] **DEV_CARTAE163** — Brancher outil bridget_auto_delegate et sous-commande native auto-delegate avec capacité auto_delegate_v1 préalable, identité/projet/fil/workspace et droits hérités. Maître ancien refuse capacité ; guichet legacy et Delegate explicite intacts. Rejeu montre dernier état confirmé/diagnostic sans fausse clôture. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation_mcp.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/cli.rs. Couverture : FR-005, FR-014, FR-019, FR-020, SC-003, SC-005, SC-007. Dépendance : T027.
- [x] T029 [US4] **DEV_CARTAE163** — Étendre et exécuter les tests CLI/MCP natifs existants : mêmecontrat strict/identité owner, ancienmaître, rejeux exact/différent, bornes mission/métadonnées/canon/cadrage et décision jamais une autorisation. Vérifier absence de gain de droits et résumé lisible natifBridget. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation_mcp.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/tests/native_delegation_e2e.rs. Couverture : FR-005, FR-009, FR-014, FR-019, FR-020, FR-021, SC-003, SC-005, SC-007. Dépendance : T028.
## US5 — Comprendre le choix et préserver les lecteurs (P2)

Validation indépendante : v1/v2 inchangés, repli exact, pages homogènes et résumé borné sans secret.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-4).

G8 et G9 sont des tâches séparées, sans preuve future anticipée.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-5).

## Dépendances et parallélisme autorisé

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-6).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-7).

## Stratégie et Article XIX/XX

Premier incrément utilisable : US1 avec politiques désactivées et parcours explicite inchangé. Il ne clôt pas le mandat : livrer les cinq histoires, G8 puis G9. T006/T012 réduisent la duplication des écritures et gardes ; T013/T031 préservent les anciens pairs/lecteurs. Table avant Task, probe sans effets et module HTTP isolé sont les seules nouvelles frontières, justifiées par ADR et audit. Aucun scheduler, runtime async, base, tokenizer, framework de mocks ou campagne optionnelle. T035/T037 vérifient la charge de maintenance et les suppressions possibles à comportement constant.

## Estimation initiale après découpage — 13 octobre 2026, historique

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-8).

## Préparation prouvée par ROOT — trois tâches soldées

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-9).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-10).

## T004 — wire figé et replay transmis par ROOT

T004 est validée sur preuve DEV_CARTAE transmise par ROOT : cargo test -p bridget-transport spec163 --lib, deux PASS/zéro FAIL, replay RC0 en 15,34 s à 20:33:45 CEST le 13 octobre 2026. Le premier essai a échoué à la compilation E0425, puis correctif et replay ; cet échec historique n’est pas effacé. Artefact local /Users/moi/.cache/bridget-delivery163-20261013/wire163.json, SHA-256 099acf8194bcecb26d71e08cd97e9096575503498188b135d24505cd2c0524c8 ; origine /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/wire163.json.

Le rédacteur n’a ni exécuté ce test ni vérifié les empreintes des sources compilées. La copie lineage après T004 diffère et reste une source en cours : aucun succès de test lui est attribué. Checkpoint historique du 13 octobre 2026 au gel T004 : la borne candidate_id attend encore une clarification interne ; aucune tâche de consommateur ou de test futur n’est cochée sur ce gel. Progression 4/45 In Progress ; première non cochée T005.

## Slot adaptateur de bureau préparatoire autorisé — distinction avec les gates finaux

ROOT autorise un groupe unique des cinq bancs existants Reader/contrats/infobulle/store sur le wire T004 figé, après copie/cache contrôlés. maxWorkers=2, nice=10, puis format/lint des sept fichiers et typechecks ciblés. Aucun nouveau banc ni suite supplémentaire ; après correction, rejouer seulement les contrôles affectés. DEV_CARTAE confirme suspension Cargo jusqu’à restitution du slot ; ROOT rapporte ps Linux à 21:02:13 CEST sans cargo/rustc. DEV_CARTAE poursuit l’édition Rust pendant ce créneau.

Ce permis est une autorisation d’essais autonomes de fixtures, pas un résultat. Il ne valide ni interopérabilité ni runtime T030, et ne coche aucune T031–T035. T030 reste leur gate final ; il ne bloque pas ces seuls essais unitaires préparatoires. Aucun compteur ou scope modifié.

## Transfert ciblé worker — autorité ROOT et preuve d’admission

Principal : child 97c3c553-2bc1-49c8-a16c-e0ab21f70be8, confirmation écrite qu’il n’éditera plus les trois fichiers transférés. Selon ROOT, ces fichiers étaient inchangés au contrôle Git avant transfert. Lecture initiale du second puis GO ownership ROOT seulement après cette confirmation.

Second : task 70aea08d-b411-4141-9f3e-2efc2df13df2, child 605c9b05-9e05-4baa-ba1a-a1be59185a1f, request aea964ae-b4c2-4a90-a24b-0c60dde3be05. Reçu /Users/moi/.cache/bridget-delivery163-20261013/delegation-worker163.json, SHA-256 90157c119ff6255b6e7d90de25875a1a49ac8f7b5f9c8a3bf49c3efe292dba29 vérifié localement. Admission ne vaut ni achèvement de T012/T013/T015 ni réussite future.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-11).

## Convergence — passage 1, manques vérifiés dans le code final

Lecture ROOT du 13 octobre 2026 et lecture croisée DEV_T3 sur sources finales (manifeste privé source-manifest163.json). Aucune source modifiée pendant cette phase. Les tâches initiales et leurs preuves historiques sont conservées. Ces corrections appartiennent au contrat existant ; elles n'ajoutent aucune fonctionnalité.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-12).

- [x] T050 **DEV_T3** — Aligner la validation des raisons v3 sur le texte affichable du producteur : refuser caractères de contrôle Cc/Cf et bidi selon le contrat Rust, conserver 1–256 octets UTF-8. Étendre les oracles de contrats existants sur ces valeurs et rejouer seulement les bancs affectés. Preuve : protocol.rs:2464 applique is_disallowed_control, bridgetLineage.ts:87 ne vérifie que la taille. Couverture : T031/T034, FR-020, SC-007.

- [x] T051 **DEV_CARTAE163** — Vérifier le schéma sanitaire du protocole Laya dans le corps borné ; une santé négative/malformée ou expirée doit alimenter le délai de reprise sans POST ni libération prématurée du permis. Compléter les seuls oracles de T021 déjà prévus pour santé/TTL/cooldown, budget partagé, MINREQ_TIMEOUT et bornes réseau/parser. Preuve : decision_service.rs:165 accepte tout2xx ; CallerDeadline retourne avant store_health. Couverture : T019–T021, FR-004/FR-010/FR-011/FR-012, SC-002/SC-004.

### Passage 2 — écarts observés sur la source R2

- [x] T052 **DEV_CARTAE163 + ROOT** — Accepter la réponse sanitaire réelle du protocole retenu, en conservant status obligatoire égal à ok et les bornes. GET maître8792 du 14 octobre à00:09:06CEST : HTTP200,271octets, SHA9e0a04a6e2b3a19664d304215ad9ab7dee2282ccb4ed6c225d6c37fedc3fc1a9 ; loaded/revisions/device font rejeter le DTO status-seul à decision_service.rs:296. Déclarer les champs sanitaires connus sans leur donner d'autorité, compléter l'oracle existant avec ce corps exact et les refus négatifs/malformés. Vérifier aussi la garde MINREQ_TIMEOUT au démarrage déjà prévue par T017, sans empêcher le daemon ni le parcours explicite. Couverture : T017/T019/T021/T039, FR-004/FR-012, SC-002/SC-004/SC-008.
- [x] T053 **DEV_CARTAE163** — Compléter le seul oracle worker/probe existant de T015 pour cwd Worktree futur sans création, invalidation à reconnexion et révocation entre verdict Eligible et admission. Réemployer fixtures/gardes existantes et observer les mêmes compteurs zéro effet ; ne pas créer de banc nouveau. Preuve : remote_worker_162.rs:1000–1090 exerce huit profils Shared et une révocation entre deux sondes ; Prepare/ParentProof162 ne prouvent pas ces nouveaux parcours. Couverture : T012–T015, FR-005/FR-006/FR-007/FR-016/FR-019, SC-003/SC-006/SC-007.
- [x] T054 **DEV_CARTAE163** — Compléter les oracles durables existants de T027 par fermeture/réouverture réelle des reçus auto aux frontières et deux admissions concurrentes corrélées ; observer compteurs HTTP, Task et lancement ainsi que l'absence de nouvelle décision après reprise. Réemployer le store sur disque et le serveur/provider contrôlés existants, sans framework ni suite globale. Preuve : delegation.rs:3544/3910 utilise mémoire/états semés ; native_delegation.rs:9044 traite les états dans le même DaemonState. Couverture : T023–T027, FR-013/FR-014/FR-015/FR-018, SC-005/SC-006.
- [x] T055 **DEV_CARTAE163** — Vérifier les contrôles déjà prévus T029 sur la nouvelle entrée auto CLI/MCP : limites mission/canon/cadrage et impossibilité de forger les droits parent. Étendre les bancs existants ou citer un oracle exécuté qui exerce réellement cette entrée et les gardes communes ; ne pas confondre une Task simulée avec une admission réelle. Preuve : lineage_cli_test.rs:287/459 et delegation_mcp.rs:452 couvrent forme/feature ; closed_arguments_reject_forged_parent_without_connecting:820 appelle seulement bridget_delegate. Couverture : T028/T029, FR-005/FR-009/FR-019/FR-021, SC-003/SC-005/SC-007.

### Passage 3 — écart réel de préférences, 14 octobre 2026

Le passage 3 n'est pas convergé. Le compteur après cet ajout est de 56 tâches. Les cases précédentes restent inchangées. Le test MCP de T055 non préfixé `spec163_` n'est pas couvert par la campagne R3 ; cette tâche reste ouverte jusqu'à son résultat ciblé.

- [x] T056 **DEV_CARTAE163** — Raccorder les préférences configurées et figées au constructeur existant de la demande de décision. Traduire leur ordre vers les seuls IDs opaques du snapshot admissible, sans transmettre les IDs de profils privés ni réintroduire un candidat filtré. Réemployer state/instructions bornés et le serveur contrôlé existant ; vérifier la charge HTTP réellement transmise, l'ordre, les préférences devenues inéligibles, l'absence de préférence et les bornes sans nouveau banc. Preuve ROOT sur R4 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/registry.rs:163,1310,1343 définit et valide preferences, mais /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs:426–482 produit toujours criteria=candidate_policy_match et state=initial_admission ; aucun consommateur métier de preferences. Couverture : US2, T007/T020/T023/T039, FR-008/FR-009/FR-010, SC-002/SC-003/SC-008. Dépendance : T054 ; validation ciblée et contre-revue/audit après correction.


### Correction de contre-revue CODE R2 — phase 5 autorisée, 14 octobre 2026

Le verdict externe sur R6 est APPROVE_WITH_CHANGES. ROOT retient F1 après vérification ; T057 corrige le contrat existant T024/T027/T029. Elle ne crée aucune fonctionnalité ni passage P4 formel. Les checkpoints précédents à 56 restent historiques ; total courant 57, quatre cases cochées. F2/F3 restent des réserves documentées dans spec/plan, sans tâche de scheduler, API ou nouvelle campagne.

- [x] T057 [US4] **DEV_CARTAE163** — Corriger F1 : réemployer la validation native pour les bornes reply_timeout/keep_alive/brief avant nouvelle réservation lorsque possible, et conserver un refus auto terminal sous CAS pour les refus métier locaux après chosen, en Shared comme en Worktree et lors de la reprise. Couvrir les sorties avant clôture de admit_delegate et les refus de preflight/création Worktree sans transformer une erreur SQL/technique en fausse clôture. Étendre seulement les oracles existants affectés T024/T027/T029 : reply_timeout=10, base_ref inconnue, rejeu du refus, reprise chosen, slot libéré et absence de redécision/Task/Start non autorisés. Aucun nouveau banc, module ou scénario de confort. Ownership courant R9 exclusif /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs et /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/workspace_git.rs, avec leurs oracles existants affectés. Réemployer la validation et un petit discriminant Git interne séparant absence de base attestée et erreur E/S/timeout/troncature ; whitelist métier fermée aux trois fermetures, sans parser des erreurs arbitraires. Vérifier le cas INSERT Task échoué par fixture_task_sql alors que l’UPDATE de refus reste possible : SQL doit rester visible et le reçu non résolu, sans faux refus durable. Aucun nouveau banc, API ou scénario hors T024/T027/T029. Preuve R6 confirmée ROOT : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs:3401–3473, :2902 et :2939. Couverture : US4, T024/T027/T029, FR-013/FR-014/FR-015/FR-016/FR-017/FR-021, SC-003/SC-005/SC-006. Dépendance : T055–T056 ; validation ciblée, contre-revue et delta d’audit frais après correction.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-13).

## Checkpoint préparatoire T057 R8 — 14 octobre 2026

ROOT et attestation_cartae163 confirment une fausse clôture SQL R7 : INSERT Task échoue avec fixture_task_sql, mais l’UPDATE de refus réussit. T057 reste ouverte. R8 est en cours depuis 00:25Z ; ownership exclusif native_delegation.rs et workspace_git.rs, 22 sources à attester. Le discriminant Git interne sépare absence de base et erreur E/S/timeout/troncature ; whitelist métier fermée aux trois fermetures, aucune interprétation libre des erreurs. Aucun résultat R8 encore attesté.

Preuves R7 et détails : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md. Les drafts d’audit R6 doivent lire le delta R8 des deux sources avant scoring final. P3 NONCONVERGED, phase 5 autorisée et 4/57 conservés ; aucun PASS R8 ou gate futur déduit.

## Checkpoint R8 attesté / R9 préparatoire — 14 octobre 2026

R8 : 16 PASS ciblés indépendamment attestés (deux natifs T024/T025, dix Git et quatre T033) ; SQL INSERT Task laisse chosen sans Task/Start et la reprise conserve le choix. Périmètre final 23 sources, et non les 22 prévues : workspace_git_tests.rs est inclus explicitement, trois changements et vingt sources inchangées par rapport à R7 augmentée. Preuves détaillées : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md. Échec de compilation initial RC101/E0308 et filtre initial zéro test restent conservés/non comptés.

CODE R3 APPROVE_WITH_CHANGES confirme la perte de classe Technical en String : workspace_path_unsafe peut encore terminer un reçu après une erreur E/S. ROOT retient la correction R9 dans T057, sans nouvelle tâche. R9 code-only préparé sur native_delegation.rs et workspace_git.rs ; trois replays affectés attendus, aucun résultat R9 encore attesté. L’oracle EIO typé prévu vérifiera le passage par la méthode de production et la fermeture, chosen/noTask/rx0 ; il ne forcera pas une syscall OS réellement en erreur.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-14).

## Checkpoint R9 final ciblé — 14 octobre 2026

ROOT atteste le transfert final R9 : 23 sources concordantes Mac/Cartae/snapshot, native_delegation.rs et workspace_git.rs changées, 21 inchangées R8. Trois oracles existants exacts passent avec RC0 : T024 local 6,918 s, T025 reprise 5,859 s, Git worktree emboîté 5,909 s. Format canonique des deux sources et diff-check RC0. Preuves exactes et erreurs historiques : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md.

Le correctif préserve la classe technique jusqu’au puits auto. Les refus déterministes de chemin au préflight restent métier ; après réservation, toute erreur de create en mode auto garde chosen avec diagnostic original et éventuels résidus de branche/chemin. Les erreurs E/S de la passe Commit restent non terminales ; ses gardes pures conservent leur traitement métier. L’oracle EIO typé traverse le mapper de production et la fermeture/store : chosen/noTask/rx0. Il ne force pas une syscall système EIO. Les résultats R8/R9 ne sont pas des suites globales ; aucun replay des succès inchangés.

Compteur 4/57 sans cochage, 31/31 couverture documentaire. ROOT relit le delta R9 et demande CODE R4 ; audit32 refresh en cours, aucun verdict/grade final anticipé. P3 NONCONVERGED historique et phase 5 corrective autorisée conservés, aucun P4 formel. T039 configuration off/on non appliquée, T040 service réel, builds/paquet/signatures, Git/livraison/G8/G9 encore ouverts. Quota CLI100 % et children ne prouvent pas des profils opérationnels en production.

### Qualification CODE R4 du checkpoint R9

Une course de garde entre préflight/create peut retenir la place chosen jusqu’à reprise dans une autre époque, avec nouvel identifiant de Task/worktree résiduel possible. Mapper EIO typé et garde réelle .worktrees fichier testés ; create/E/S Commit vérifiés par compilation/lecture, sans oracle direct ni PASS NotFound. CODE R4 APPROVE_WITH_CHANGES demande seulement cette qualification, sans patch/test supplémentaire. Audit32/grade final et cochage ROOT encore ouverts ; aucune case ou tâche modifiée.

## Checkpoint T021 final — delta de tests seulement, 14 octobre 2026

Le manifeste final R9+T021 porte sur 23 sources : deux corps de tests changés dans native_delegation.rs/decision_service.rs, 21 sources inchangées ; les deux préfixes de production restent identiques à R9. L’oracle T054 existant, étendu à la réponse tardive après reprise réelle, donne 1 PASS/RC0, 7,718489 s commande/0,07 s banc. Premier FAIL de fixture conservé ; aucun test supplémentaire ni replay des succès antérieurs. Preuves exactes : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md.

L’entrée de build existante decision-request-max8.json, 709 octets/objet questions choice, a été copiée byte-identique du serveur au miroir Mac. Elle doit être versionnée avant le build Mac T038 ; ce n’est pas une preuve de recette de production. L’ancien artefact privé861 octets/questions tableau reste historique et obsolète. T040 doit utiliser la requête réellement capturée et ses tokens ; aucun fixture ne vaut réussite d’un POST de production. Audit32 et validation finale encore en cours, aucune acceptation49/57 anticipée : 4/57 inchangé. P3 NONCONVERGED historique/exception phase5 intactes, aucun R10 de production ou P4 formel ; builds/G8/G9 encore ouverts.

## Acceptation ROOT du développement — 49/57, 14 octobre 2026

GO ROOT après lecture du delta T021 final et du reçu FINAL_T021_READONLY_CONFORMANT : T005–T037 et T046–T057 acceptées/cochées, ajoutées aux quatre préparatoires déjà prouvées. T038–T045 restent ouvertes ; aucune recette, build Mac, source poussée/installée, activation ou G8/G9 anticipée. P3 NONCONVERGED reste historique ; l’exception utilisateur de poursuite corrective en phase5 est conservée, aucun P4 formel ou faux CONVERGED ajouté.

Audit final readonly : grade A/99,047619 ; 0 CRITICAL, 0 HIGH, 3 MEDIUM, 2 LOW ; TEST-001 fermé par l’oracle T054 existant exécuté. Couverture4,655325 % du snapshot et0,355837 % du contexte global, donc audit partiel et aucun GREEN global. Reçu /Users/moi/.cache/bridget-delivery163-20261013/audit-preparation/audit163-final-t021-receipt.json, SHA-256 057c0b4349388fa46b3cc1c843a7c352c738260d01434161ade6eb5f4810fe87, relu et hash vérifié par le rédacteur. Aucun audit, source, test, Git, build ou service modifié/exécuté par ce cochage documentaire.

## Checkpoint installation et provenance — 53/57, 14 octobre 2026

ROOT accepte T038/T040/T041/T042 sur leurs preuves réelles. Bridget source 5105378bf4f03376e9bc93e52065cb022f81dee5 installé : Mac SHA-256 6c8f89203c85c76e7add2a4d06bb86eedd6cdb3f74470eac11920cc0f6068063, Linux SHA-256 6e5d9717d6054511326f84c684d65c3f3bc1a5086cb514b2d6b64a03ad9b97e0. Maître 44553 né 04:55:03 CEST et pont 44667 né 04:55:05 utilisent /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget ; worker 1411115, naissance 146166371 utilise /home/moi/.local/lib/bridget-communication/163-5105378bf4f0/bin/bridget.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-15).

Sauvegarde retry : phase_complete/activate, projection173→174 de même génération et 33 autres tables exactes, schéma/count inchangés ; halt humain complet préservé. Migration additive reçusauto, quick_check ok/FK0, schémas 7/12/Fleet5. Sauvegarde jointe vérifiée ; après activation, toute rétrogradation exige une nouvelle sauvegarde jointe sans perte. La sauvegarde préactivation ne couvre pas les nouveaux reçus. L’incident171→172 et les anciens PID 86807/86830/1403968 restent uniquement historiques.

Le reçu d’installation conserve handshake_and_cancel_recipe_not_proven=true : poignée de main et annulation opérationnelles restent à vérifier pour G8.

T039 : /Users/moi/.cache/bridget-core/agents.json est appliquée au SHA 5dead7198c0220477d88229d1b0ab0f4b3fe5858a0391bde1cfb687c3265c93b ; fixed/fallback local-codex-pro, agents/machines inchangés. auto_engine=false, auto_placement=false, service_enabled=false. L’application off est réelle, mais l’activation des axes/service et les preuves G8 restent ouvertes : T039 non cochée, comme T043/T044/T045.

Ordre réel : sources committées/intégrées/poussées avant laboratoire T040, pour obtenir un checkpoint déterministe compilé identique sur Mac/Linux ; validation T040 exécutée avant activation. Ce commit de source préalable n’est pas une clôture de livraison ni un hash futur inventé. ROOT accepte T040 sur observations R4 et supplément corrigé 54056f…8d49 ; RC1/course F historique conservé, aucun nouveau RC0 ou POST ajouté. P3 NONCONVERGED historique/exception utilisateur phase5 conservés, aucun P4 formel.

Reçus :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-16).

## Correctif G8 — façade MCP T058, 14 octobre 2026

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-17).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-18).

Compteur courant53/58 : T039/T043/T044/T045/T058 seules ouvertes. La preuve brute est conservée : /Users/moi/.cache/bridget-delivery163-20261013/g8-mcp-off-error-observation.log ; SHA-256 03e823bb6a993a7f1b8b546edc53f6aeded6e163bcd9ed507b22a68dc28f2f47. P3 NONCONVERGED historique et poursuite corrective sous mandat utilisateur restent explicites ; aucun P4 formel inventé.

- [x] T058 [US4] **DEV_CARTAE163 / delivery_helpers, ROOT livraison** — Raccorder minimalement bridget_auto_delegate dans la façade MCP : dispatcher tools/call, périmètre scope et chemin hook/wrapper, avec réemploi de delegation_mcp/native et des gardes d’identité/droits/capacité. Corriger le refus -32602 outil inconnu malgré tools/list, sans changer le protocole métier ni ajouter un framework. Réemployer les oracles MCP existants pour traverser annonce→appel réel→guichet natif, et vérifier droits/refus/idempotence sans admission non autorisée. Grouper la validation affectée ; aucune FULL, outil ou banc nouveau. ROOT fait contre-revue et delta d’audit ciblés, checkpoint source propre/intégré/poussé, builds Mac/Linux de même commit et redelivery du correctif ; reprendre ensuite le G8 déjà prévu avant T039/T043/T044/T045. La source510 installée et T038/T042 antérieures restent historiques, aucun hash/build/recette futur anticipé. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/mcp.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation_mcp.rs ; raccords existants scope/hook/wrapper identifiés par ROOT, ownership exclusif delivery_helpers. Preuve : /Users/moi/.cache/bridget-delivery163-20261013/g8-mcp-off-error-observation.log. Couverture : T028/T029, FR-005/FR-013/FR-014/FR-015/FR-019/FR-020/FR-022, SC-003/SC-005/SC-007/SC-009. Dépendance : constat G8 sur installationT042 ; correctif avant clôtureT039/T043/T044/T045.


## Acceptation opérationnelle ROOT — historique57/58 intégré dans c57, 14 octobre 2026

ROOT accepte T039/T043/T044/T058 sur les preuves de configuration effective, de recette G8 et de nettoyage G9. T045 reste seule ouverte pour la clôture documentaire, le commit et le push finaux. La source active est 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5 ; les métadonnées de clôture ultérieures auront leur propre commit. Le correctif T058 est fermé par son code, douze oracles exacts, la revue R5 avec F1 aligné, le delta d’audit et G8 réelle.

Reçus : /Users/moi/.cache/bridget-delivery163-20261013/g8-t058-operational-final.json ; SHA-256 5b1c00923f2f307136b5831cc8d3328227b7284deacf44c793d71542e3ceb52c. /Users/moi/.cache/bridget-delivery163-20261013/g9-t058-cleanup-final.json ; SHA-256 08091fc94791644308b258f2eb912ee375145d66004ef9f9c5e0d822a34fb5c3. Les faits et limites sont consignés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/163-decision-admission/validation/livraison163.md. Les anciens chemins de worktrees Mac retirés désignent des lieux historiques, sans nouvelle copie de travail active.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/tasks.md#extrait-19).


## Clôture T045 — 58/58, 14 octobre 2026

ROOT a intégré et poussé les documents de livraison G8/G9 au commit c57cfd35efa1338cda5256a02c57d71258d8b8d4 sur github/main. Le reçu vérifie HEAD égal à la référence distante et dépôt propre à ce checkpoint. T045 est acceptée sur cette preuve : 58/58, Implemented ; Pipeline : Livré avec conservations justifiées. Le checkpoint 57/58 précédent, intégré dans c57, reste historique.

Preuve : /Users/moi/.cache/bridget-delivery163-20261013/git-final-doc-checkpoint-c57cfd35.json ; SHA-256 56316fdcb4202865fbb57e7979a11d302bb968e2170453a9523070a490a30390. Le build actif reste 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5. Le commit documentaire c57 n’est pas un nouveau binaire. Les marqueurs Implemented/58/58 sont intégrés et poussés dans af8998a7d7494913d4420e2dca7354a73bd8a4f5. ROOT a vérifié github/main égal HEAD propre ; les sources suivies de Cartae main/worktree sont alignées sur ce commit documentaire, sans changement du build3efa. Preuve d’alignement Cartae : /Users/moi/.cache/bridget-delivery163-20261013/git-final-cartae-af8998a7-command.json ; SHA-256 a8566f602017d52ea0ca9ab3706871038275aa04477b67077fb572b74d638620.

Les conservations G9 sont acceptées et nommées, sans suppression forcée. Anciennes FULL non GREEN, audits partiels et leurs findings, P3 NONCONVERGED historique et poursuite corrective autorisée par l’utilisateur en phase 5 sont maintenus. Aucun nouveau test, audit global, service ou P4 formel n’est déduit. L’ETA initiale 300–540 min et l’ETA après Tasks 360–600 min restent historiques ; dernière fenêtre 05:30–06:00 CEST. Aucun temps écoulé exact de développement ou CPU n’est ajouté.
