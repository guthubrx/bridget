# Tâches - Session155 remontée des demandes de l'enfant

**Entrées** : `spec.md`, `plan.md`, `research.md` (R1-R15), `data-model.md`,
`contracts/child-requests155.md`, `reuse-audit.md`, `adversarial-review-codex.md`,
`analyze-report.md`.
**Racine** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation`.
**Recette de test** : `quickstart.md` (`umask 077`, racine `/tmp/b.XXXX`,
`BRIDGET_HOME` privé, `--features test-support`).

Règle : une tâche n'est cochée qu'après son test vert. Les tâches « Rouge d'abord »
sont écrites et vues en échec avant le code. Pas de commit automatique.

## Phase 0 - Correctif de sécurité (livrable seul, indépendant du reste)

- [x] T000 CORRECTIF DE SÉCURITÉ, préexistant en production, livrable seul : dans `approval_response` de `crates/bridget-transport/src/codex_app_server.rs` (`:2195-2226`), quand la décision voulue (`accept` si `permissions == "allow"`, sinon `decline`) n'est pas proposée dans `availableDecisions`, le code actuel retombe sur la première décision proposée (`:2209-2213`), qui peut être `acceptForSession` ou un amendement de règle. Remplacer ce repli par un repli fermé : `accept` n'est écrit que si `accept` est proposé ; sinon `decline`, puis `cancel` si proposés ; sinon une erreur JSON-RPC (`-32000`, `child_request_refused`) ; jamais une autre décision. Écrire le test rouge d'abord : offre `["acceptForSession"]`, offre `["acceptWithExecpolicyAmendment"]`, offre sans `decline` en mode `deny`, offre sans `accept` en mode `allow`, offre vide ou absente ; garder le test existant `approval_response_choisit_accept_ou_decline_selon_permissions` vert ; consigner dans `specs/155-demandes-enfant/validation/correctif-deny.md` ; ne dépend d'aucune autre tâche et peut être livré avant le reste. Cette tâche est la seule exception volontaire au comportement 149 du mode `deny` (FR-003)

## Phase 1 - Préparation

- [ ] T001 Établir la référence de non-régression avant tout changement : lancer les filtres `native148`, `native149`, `native150`, `delegation149`, `spec_087`, `elicitation` et `approval_response` (daemon et transport) et le test `claude_native_permissions_test` avec la recette, et consigner les compteurs PASS/FAIL/ignorés dans `specs/155-demandes-enfant/validation/baseline.md` ; vérifier que les acquis 150 (champ `mission_deadline_at`, variante `AutonomousEffect::NativeDelegation`, purge d'arbre) sont présents
- [ ] T002 Figer les formes réelles du fournisseur : lancer `codex app-server generate-json-schema` (Codex 0.161.0) vers un dossier privé de `/tmp`, puis écrire `specs/155-demandes-enfant/contracts/codex-requests.md` avec, pour CHAQUE méthode (`item/commandExecution/requestApproval`, `item/fileChange/requestApproval`, `item/permissions/requestApproval`, `mcpServer/elicitation/request`, `item/tool/requestUserInput`, `serverRequest/resolved` si elle existe) : les paramètres, la liste blanche des champs d'effet (chemins, racines, réseau, permissions additionnelles, amendements de règles), la durée de chaque octroi (une fois, tour, session), les décisions proposées et la forme exacte de chaque réponse ; toute méthode dont les effets ou la durée de l'octroi ne sont pas établis par le schéma est ajoutée à la liste des méthodes refusées (`approval_method_unsupported`) ; T003 ne démarre pas tant que cette liste n'est pas écrite ; si une forme contredit `research.md`, `data-model.md` ou `spec.md`, les corriger avant T003

## Phase 2 - Fondations (bloquantes)

- [ ] T003 [P] Dans `crates/bridget-transport/src/protocol.rs` : ajouter `child_requests: Option<String>` à `Delegate` (`#[serde(default, skip_serializing_if = "Option::is_none")]`), la variante `NativeDelegationRequest::Respond { task_id, request_id, decision, answer }`, les trames `WrapperToDaemon::ChildRequest`, `WrapperToDaemon::ChildRequestAnswered`, `DaemonToWrapper::ChildRequestAnswer` (avec session et tour) et les types fermés de type de demande, de verdict et de décision ; test : une requête `Delegate` sans le champ produit les mêmes octets `serde_json::to_vec` qu'avant, un champ inconnu de `Respond` est refusé
- [ ] T004 [P] Dans `crates/bridget-transport/src/managed_session.rs` : ajouter `ManagedEventKind::ProviderRequest` et `ManagedSession::answer_provider_request` avec une implémentation par défaut qui refuse ; test : un pilote sans implémentation refuse, aucun autre pilote ne change
- [ ] T005 [P] Dans `crates/bridget-daemon/src/delegation.rs` : ajouter `child_requests: Option<String>` à `Task` (`#[serde(default)]`), la table `native_child_requests` et ses index (voir `data-model.md` : instance, session, tour, empreinte, verdict, `delivery_state` à six valeurs, `void_reason`, `sent_at`, `inbox_settled`) et l'API de magasin (insérer de façon idempotente avec contrôle d'empreinte, lire les ouvertes par tâche et par échéance, clore une fois, passer `pending` à `void` ou `delivered`, compter ouvertes et totales) ; annoter la complexité ; tests : un payload 150 sans le champ se relit avec `None`, un doublon au même contenu rend la même ligne, un doublon au contenu différent rend `request_conflict`, un état terminal ne change plus, `void` ne revient pas à `pending`, `sent` ne revient pas à `pending`

## Phase 3 - Limite absolue et politique (US2, US3, US5, US7)

**But** : la limite existe avant toute remontée ; aucune écriture de fichier n'est approuvée sous un bac à sable confiné.
**Test indépendant** : filtres `native155_rights`, `native155_files`, `native155_admission`.

- [ ] T006 [US3] Rouge d'abord : écrire dans un fichier de test voisin de `crates/bridget-daemon/src/native_permissions149_tests.rs` les tests `native155_rights_*` de la table de verdicts de `data-model.md` (au moins 20 cas) : question valide, parent absent, secret, mode URL, méthode `permissions`, méthode inconnue, détails tronqués, champ d'effet inconnu, enfant `readOnly`, parent `readOnly`, parent `externalSandbox`, politique du parent incomplète, écriture de fichier sous `workspaceWrite` (toujours `file_confinement_unproven`), écriture de fichier sous `dangerFullAccess`, réseau accordé, réseau interdit, commande avec plein accès, commande avec bac à sable, demande mixte (le plus restrictif gagne), durée de session ; les cas « réseau interdit » et « commande hors confinement » doivent donner `Forbidden` même pour une décision humaine
- [ ] T007 [US3] Implémenter `classify_request` dans `crates/bridget-daemon/src/native_permissions.rs` : fonction pure sur des faits déjà résolus, trois verdicts, motifs nommés, échec fermé ; annoter la complexité ; T006 passe
- [ ] T008 [US3] Rouge d'abord : tests `native155_files_*` : sous un parent `workspaceWrite`, toute approbation d'écriture est `Forbidden(file_confinement_unproven)` pour un chemin dans la zone, avec `..`, avec un lien symbolique, pour un fichier neuf, pour un dossier qui devient un lien après le contrôle, pour `.git`, pour `/tmp` ; sous `dangerFullAccess`, une écriture va à l'humain ; les chemins cités sont affichés bornés, nettoyés et marqués non vérifiés, et un test prouve qu'ils ne changent jamais le verdict
- [ ] T009 [US3] Implémenter dans `crates/bridget-daemon/src/native_permissions.rs` le refus `file_confinement_unproven` dans `classify_request` et la mise en forme des chemins cités (au plus 64, 4096 octets, sans caractères de contrôle ni de direction) ; aucune résolution de chemin, aucun accès au système de fichiers ; la preuve de confinement à l'exécution et la résolution de chemins sont reportées à une session future ; T008 passe
- [ ] T010 [US7] Dans l'admission de `crates/bridget-daemon/src/daemon/native_delegation.rs`, `crates/bridget-daemon/src/delegation_mcp.rs` et le catalogue : accepter `child_requests`, refuser `child_requests_unsupported` pour un fournisseur non supporté, refuser `child_requests_reviewer_unsupported` si la politique du parent est incomplète ou si `approvals_reviewer` n'est pas `user`, inclure le champ dans l'enveloppe idempotente, annoncer `child_requests_default`, `child_requests_supported` et les deux fenêtres ; mettre à jour le test qui fige les clés ; tests `native155_admission_forward_codex`, `native155_admission_forward_non_supporte`, `native155_admission_auto_review_refuse`, `native155_admission_guardian_subagent_refuse`, `native155_admission_octets_canoniques_inchanges`, `native155_admission_rejeu_autre_valeur_envelope_mismatch`, `native155_admission_politique_enfant_inchangee`
- [ ] T011 [US7] Dans `spawn_task` de `crates/bridget-daemon/src/daemon/native_delegation.rs` : poser une variable d'environnement voisine de `CHILD_POLICY_ENV` qui porte le mode figé ; dans `crates/bridget-daemon/src/wrapper.rs` (près de `:3705`), la lire et la transmettre à la session ; test : un enfant `deny` ne reçoit pas la variable et garde le chemin 149

## Phase 4 - Transport Codex (US1, US2, US7)

**But** : le fournisseur attend, le transport ne se bloque pas, rien n'est accordé par défaut.
**Test indépendant** : filtre `native155_transport` dans `bridget-transport`.

- [ ] T012 [US2] Rouge d'abord : dans `crates/bridget-transport/src/codex_app_server.rs`, tests avec faux serveur : en mode `forward`, une approbation de commande ou de fichier n'écrit aucune réponse automatique et émet un événement `ProviderRequest` ; une fin de tour reçue ensuite est traitée ; la 5e demande simultanée est refusée ; `item/permissions/requestApproval` reçoit `decline` avec `approval_method_unsupported` ; une offre qui ne contient que `acceptForSession` ou des amendements de règles produit une erreur JSON-RPC, jamais une décision d'accord ; une demande hors du tour de la mission active reçoit `request_unlinked` ; en mode `deny`, les tests existants `approval_response_*` restent verts
- [ ] T013 [US2] Implémenter dans `crates/bridget-transport/src/codex_app_server.rs` la table des attentes (4 entrées au plus, clé session + tour + identifiant), l'événement `ProviderRequest` pour les deux méthodes d'approbation retenues, `answer_provider_request` (réponse JSON-RPC `accept`, `decline` ou `cancel` seulement, repli fermé partagé avec le correctif T000, une seule fonction), le contrôle du tour actif de la mission, la normalisation des détails avec indicateur de troncature, et le nettoyage à la fin du tour, à l'interruption et à l'arrêt ; le chemin `deny` ne change pas ; T012 passe
- [ ] T014 [US1] Rouge d'abord : tests avec faux serveur pour l'élicitation MCP (formulaire d'un serveur tiers, mode URL refusé, formulaire vide de Bridget toujours accepté) et pour `item/tool/requestUserInput` (forme de T002, question à secret refusée)
- [ ] T015 [US1] Implémenter les deux types de questions dans `crates/bridget-transport/src/codex_app_server.rs` : normaliser (type, identifiants de question, secret, mode), valider `answer` contre les identifiants proposés, écrire la réponse ou le refus ; T014 passe
- [ ] T016 [US2] Dans `crates/bridget-daemon/src/wrapper.rs` : transformer l'événement en `WrapperToDaemon::ChildRequest`, publier `waiting_approval` ou `waiting_user_input` par `publish_execution_transition` (sans `publish_approval_wait`), tenir la table des attentes sous un seul verrou qui sérialise la réponse et la fermeture, garder un anneau des 64 dernières demandes réglées, appliquer `ChildRequestAnswer` seulement si identifiant, session et tour correspondent exactement et si la demande est encore `waiting` (sinon `applied = false`, ou l'état exact si elle est dans l'anneau), traiter `ChildRequestCancel` sous le même verrou et répondre `ChildRequestClosed` (`closed_before_apply` ou `already_applied`), répondre `ChildRequestAnswered`, répéter les demandes en attente après reconnexion, revenir à `running` ; vérifier que `transition_if_current` accepte `waiting_*` vers `running` ; tests `native155_wrapper_demande_puis_reprise`, `native155_wrapper_reponse_ancien_tour_refusee`, `native155_wrapper_fermeture_puis_reponse_jamais_ecrite`, `native155_wrapper_reponse_puis_fermeture_already_applied` (course forcée par une barrière entre les deux threads), `native155_wrapper_renvoi_apres_reglement`

## Phase 5 - Cœur du daemon (US1, US2, US3, US4)

**But** : une demande reçue a un verdict, un délai, une validité de remise et une issue.
**Test indépendant** : filtres `native155_request`, `native155_lifecycle`.

- [ ] T017 [US2] Rouge d'abord : tests daemon `native155_request_*` : une trame crée une ligne `open` ; la même trame deux fois ne crée qu'une ligne ; même identifiant fournisseur avec un autre contenu rend `request_conflict` ; identifiant réutilisé dans un autre tour crée une autre ligne ; trame d'une instance qui n'est pas celle de l'enfant rejetée ; accusé d'une autre connexion ignoré ; verdict `Parent`, `Human` et `Forbidden` selon la table ; 5e ouverte et 33e totale donnent `refused:request_budget_exhausted` ; moins de 30 s de tâche donne `refused:deadline_too_close` ; une décision reçue après `deadline_at` rend `request_expired` ; un refus `Forbidden` n'ouvre aucun item de boîte
- [ ] T018 [US2] Créer `crates/bridget-daemon/src/daemon/native_child_requests.rs` (déclaré dans `daemon.rs`) : réception de `ChildRequest`, rapprochement avec l'instance attestée, identifiant dérivé, résolution des chemins (T009), verdict (T007), budgets, échéance (600 s parent, 3600 s humain, plafonnée par `mission_deadline_at - 30`), faits vérifiés et texte du modèle séparés, refus nommés ; annoter la complexité ; T017 passe
- [ ] T019 [US1] Dans `native_child_requests.rs` : avis au parent pour les questions par `send_as` (message système, identifiant `native-request-<request_id>`, faits d'abord, texte cité et nettoyé) ; test `native155_request_avis_idempotent`
- [ ] T020 [US2] Dans `crates/bridget-daemon/src/daemon.rs` : faire rendre à `daemon_deposit_human_inbox` le résultat du dépôt (les deux appelants actuels l'ignorent) ; dans `crates/bridget-daemon/src/human_inbox.rs` : ajouter la recherche d'un item par clé exacte ; dans `native_child_requests.rs` : avant un dépôt, réutiliser l'item de même clé, puis lier `inbox_item_id` ; tests `native155_human_depot_idempotent_apres_panne`, `native155_human_item_deja_resolu_reutilise`
- [ ] T021 [US2] Dans `native_child_requests.rs` et `daemon.rs` : envoi de `ChildRequestAnswer` à la connexion de l'enfant (patron `ControlExecutionDispatch`) seulement si `delivery_state = pending` et que FR-027 est vrai à cet instant, échéance comprise ; écrire `sent` et `sent_at` AVANT de pousser la trame ; accusé positif vers `delivered`, accusé négatif vers `void`, renvoi identique à chaque tick tant que `sent` sans accusé et que la connexion existe ; si la connexion disparaît, l'accord `sent` devient `unknown` et l'issue est `cancelled_outcome_unknown` ; tests `native155_lifecycle_renvoi_jusqu_a_accuse`, `native155_lifecycle_accuse_negatif_annule`, `native155_lifecycle_enfant_disparu_accord_sent_reste_unknown`, `native155_lifecycle_plantage_entre_sent_et_envoi`, `native155_lifecycle_accord_apres_echeance_jamais_envoye`
- [ ] T022 [US4] Rouge d'abord : tests `native155_lifecycle_*` : expiration donne `decline` ou `cancel`, jamais `accept` ; annulation de la tâche ; fin de tâche ; interruption par la pause ; révocation de la délégation ; changement d'instance de l'enfant ; redémarrage du daemon ; accord enregistré, puis annulation, puis reconnexion de l'enfant : aucun accord appliqué (`void`) ; envoi, puis annulation du daemon, puis réception du wrapper : soit l'accord est écrit avant la fermeture (`already_applied`), soit il n'est jamais écrit ; accord `pending` à l'échéance : `void` ; accord `sent` à l'échéance : fermeture confirmée avant tout état final
- [ ] T023 [US4] Implémenter dans `native_child_requests.rs` et `native_delegation::tick` (et `prepare_restart`) l'expiration et la clôture : `pending` vers `void` avec raison ; `sent` vers l'envoi de `ChildRequestCancel` puis attente de `ChildRequestClosed` (`already_applied` vers `delivered`, `closed_before_apply` vers `void`), `unknown` si le wrapper ne répond plus ; l'état final n'est jamais « annulé » tant que la fermeture n'est pas confirmée ; T022 passe

## Phase 6 - Réponse du parent aux questions (US1, US5, US6)

**Test indépendant** : filtre `native155_respond`.

- [ ] T024 [US1] Rouge d'abord : tests `native155_respond_*` : `accept` avec `answer` sur une question réussit et l'enfant reçoit la réponse ; `decline` ; réponse du parent à une approbation rend `parent_cannot_approve` ; appelant qui n'est pas le propriétaire rend le refus d'identité ; rejeu identique rend l'état ; autre décision sur demande close rend `request_closed` ; décision durable rend `decision_not_offered` ; `answer` sur une approbation ou identifiant de question inconnu rend `answer_invalid` ; parent révoqué rend `permission_not_inherited` ; réponse après l'échéance rend `request_expired` ; un enfant ne répond pas à sa propre demande
- [ ] T025 [US1] Implémenter `Respond` dans `crates/bridget-daemon/src/daemon/native_delegation.rs` (ordre des contrôles du contrat) et l'outil `bridget_task_respond` dans `crates/bridget-daemon/src/delegation_mcp.rs` et `mcp.rs` (listes `:475` et `:2607`, `tools().len()` de 4 à 5, liste blanche des clés) ; ajouter `bridget_task_respond` à `BRIDGET_SAFE_MCP_TOOLS` dans `crates/bridget-daemon/src/wrapper.rs` (20 vers 21 outils) et mettre à jour les tests de liste (`wrapper.rs` autour de `:8116-8172` et `crates/bridget-daemon/tests/claude_native_permissions_test.rs:400`) ; test d'un parent natif réel Claude et Codex qui voit l'outil ; T024 passe
- [ ] T026 [US6] Ajouter `pending_requests` et `recent_requests` à la vue de `bridget_task_status` (clés ajoutées, aucune retirée ; les demandes `human` y figurent sans réponse possible) ; tests `native155_respond_statut_liste_ouvertes`, `native155_respond_statut_cles_existantes_inchangees`
- [ ] T027 [US4] Pendant la pause du référent, `Respond` rend `referent_paused` par `admit_autonomous_effect` (variante de 150) ; la lecture du statut reste disponible ; tests `native155_respond_pause_refuse`, `native155_respond_apres_levee_reussit`

## Phase 7 - Route humaine (US2)

**Test indépendant** : filtre `native155_human`.

- [ ] T028 [US2] Rouge d'abord : tests `native155_human_*` : une approbation `Human` dépose un item avec les faits d'abord et les options `accept` et `decline` ; la décision humaine (principal humain attesté) est appliquée puis acquittée ; une décision humaine arrivée après expiration, annulation ou purge est acquittée sans exécution ; 250 décisions humaines obsolètes n'empêchent pas le traitement d'une nouvelle décision et n'exécutent rien ; un item ouvert est fermé à l'expiration ; l'humain ne peut pas répondre à une question
- [ ] T029 [US2] Dans `native_child_requests.rs` : à chaque tick, lire chaque item lié par son identifiant exact (au plus 512 lignes `inbox_settled = 0`), appliquer une décision si la demande est ouverte, si l'échéance n'est pas passée et si FR-027 est vrai, sinon acquitter sans exécuter, fermer par `close_self` un item encore ouvert, puis passer `inbox_settled` à 1 ; aucune relève par balayage de la file ; T028 passe

## Phase 8 - Pas de boucle, texte hostile (US5)

**Test indépendant** : filtres `native155_chain`, `native155_hostile`.

- [ ] T030 [US5] Rouge d'abord : tests `native155_chain_*` : chaîne parent, enfant, petit-enfant avec de vrais parents natifs ; la question du petit-enfant va à l'enfant et se calcule avec le snapshot de l'enfant ; le parent racine ne peut pas répondre à la demande du petit-enfant ; aucun agent ne répond à sa propre demande ; la demande de l'enfant n'est pas transférée au parent racine quand l'enfant se tait
- [ ] T031 [US5] Ajuster les contrôles de propriétaire et de route de `native_child_requests.rs` et de `Respond` pour que T030 passe (un seul saut, identité attestée du propriétaire, instance d'origine acceptée comme dans `recover_owner`) ; ne rien ajouter qui ne soit exigé par un test
- [ ] T032 [US5] Rouge d'abord puis implémentation : tests `native155_hostile_*` (usurpation d'instructions dans `reason`, faux identifiant de demande dans le texte, commande cachée par retours à la ligne, caractères de contrôle et de direction, faux « déjà approuvé ») : le texte du modèle est cité à part, nettoyé et borné ; les faits affichés viennent des champs structurés ; l'avis et l'item ne contiennent jamais un identifiant ou une décision tirés du texte du modèle

## Phase 9 - Intégrations 150 et 151

- [ ] T033 [US4] Dans `crates/bridget-daemon/src/delegation_lineage.rs` : supprimer les lignes de `native_child_requests` d'un arbre dans la transaction de purge de la session 150 ; tests `native155_purge_supprime_les_demandes_de_l_arbre`, `native155_purge_demande_ouverte_impossible_sur_arbre_terminal`
- [ ] T034 [US6] Condition : si la session 151 est livrée, faire rendre la main à son outil d'attente dès qu'une demande s'ouvre (`pending_requests` non vide), avec la forme de résultat de 151 ; test `native155_attente_reveillee_par_une_demande`. Si 151 n'est pas livrée, laisser cette tâche non cochée et le noter dans `analyze-report.md` (aucun code spéculatif)

## Phase 10 - Bout en bout, documentation, clôture

- [ ] T035 Dans `crates/bridget-daemon/tests/native_delegation_e2e.rs` : scénario `native155_e2e` avec faux serveur Codex : délégation `forward`, question, avis, réponse du parent, reprise de l'enfant ; approbation dans la limite, item humain, décision, reprise ; approbation hors limite refusée sans item ; expiration sans réponse ; aucun service réel relancé
- [ ] T036 [P] Écrire l'ADR `docs/decisions/155-remontee-des-demandes-de-l-enfant.md` : contexte, décision, conséquences, statut Accepté ; elle dit explicitement en quoi elle change « aucun dialogue de validation de permission » (spec 149 FR-008, ADR 149), ce qui reste vrai (pas de grant par mission, pas de gain de droits, limite absolue sans exception humaine), l'option par tâche, la répartition parent pour les questions et humain pour les approbations, et les alternatives rejetées (défaut `forward`, approbation par le parent, escalade automatique, décisions durables, relève par balayage, Claude)
- [ ] T037 [P] Documenter `child_requests`, `bridget_task_respond`, les refus, les fenêtres et la route humaine dans `docs/delegation-native.md`, `skills/bridget/SKILL.md` et `skills/bridget/references/commandes.md`
- [ ] T038 Relire le diff complet (Articles XIX et XX : lignes suppressibles, abstractions, chemins d'erreur), vérifier les annotations de complexité, écrire `specs/155-demandes-enfant/implementation.md` et enregistrer `git diff` dans `specs/155-demandes-enfant/validation/diff-155.patch`
- [ ] T039 Relancer les filtres de T001 et comparer à `validation/baseline.md` : aucun test 148, 149, 150, 087 ne passe de vert à rouge ; lancer la suite complète (`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast` avec la recette) ; rejouer seuls les tests sensibles à la charge avant de conclure ; consigner dans `validation/tests-155.md`
- [ ] T040 Recette manuelle avec l'utilisateur (voir `quickstart.md`) dans un daemon de test isolé : question répondue par le parent, approbation acceptée par l'humain, approbation hors limite refusée, expiration ; consigner dans `validation/recette.md` ; ne cocher qu'après validation explicite de l'utilisateur

## Dépendances

- T001 puis T002 avant tout code.
- T003, T004, T005 en parallèle, avant les phases 4 à 8.
- T000 est indépendant : à livrer en premier, seul.
- T006 à T009 avant T017 (le verdict utilise la limite).
- T010 et T011 avant T012 (le mode doit exister).
- Phase 4 (transport) avant T016 (wrapper) et avant T035.
- Phase 5 (daemon) avant Phase 6 (réponse) et Phase 7 (humain).
- T033 dépend de la purge de la session 150.
- T034 dépend de la session 151.
- Les phases 4 à 8 touchent `native_delegation.rs`, `wrapper.rs` et `daemon.rs` :
  exécution séquentielle.
- T036 et T037 en parallèle ; T038 à T040 en dernier.

## Exemples de parallélisme

- T003, T004, T005 : fichiers distincts.
- T036, T037 : documentation seule.

## Stratégie

MVP = Phases 1 à 6 : questions remontées au parent, limite absolue vérifiée, issue
explicite, défaut `deny` intact. Puis route humaine pour les approbations (Phase 7),
boucles et texte hostile (Phase 8), intégrations, clôture.

## Charge future (Article XX)

- Réduisent la charge : T018 (un seul verdict calculé côté daemon), T007 (limite en un
  point testé par table), T029 (lecture par identifiant, plus de balayage), T036
  (décision 149 corrigée par écrit).
- Ajoutent de la complexité justifiée : T013 et T016 (attente hors boucle de lecture),
  T021 (accusé et validité de remise : une réponse perdue bloquerait l'enfant, un
  accord tardif donnerait un droit), T016 et T021 (verrou et état `sent` : sans eux, un accord
  déjà envoyé pourrait survivre à une annulation).
