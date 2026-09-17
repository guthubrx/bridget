# Tâches 102 — préparation pour implémentation ultérieure

Statut de développement : In Progress au sens livraison non implémentée ; **0/32 tâches exécutées**.
Autorisation actuelle : documents seulement. Toutes les tâches ci-dessous sont
**non tentées volontairement sur instruction utilisateur**, pas en échec technique.
Une demande d'implémentation nouvelle est nécessaire. T001 doit être la première.

## Lecture et règles d'exécution

Racine absolue : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents
Contrat de référence : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/specs/102-fils-inter-agents/contracts/thread-api.md
Matrice V01–V36 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/specs/102-fils-inter-agents/test-plan.md

Chaque tâche porte son oracle ; écrire les tests avant le comportement lorsque
la tâche le demande. Ne cocher qu'après résultat observé, puis consigner commande,
pass/fail et limites dans implementation.md. Les chemins ci-dessous sont les
cibles principales ; modifier les appelants/littéraux affectés seulement après
recherche, sans refactor périphérique. Aucun fichier protégé SpecKit à éditer.

## Phase 1 — Préparation et sécurité des tests

- [ ] T001 Vérifier la base101 et les changements des autres agents, consigner commit de socle, git status et absence de copie manuelle dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/specs/102-fils-inter-agents/implementation.md ; demander l'intégration si101 reste non commité. Sortie : base commune explicitement établie AVANT fichiers Rust. Dépendance de toutes les tâches.
- [ ] T002 Préparer le harnais isolé dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/tests/spec102_threads_test.rs en réutilisant les fixtures sûres ; home/socket/DB/journaux temporaires, faux fournisseurs, arrêt précis sans SIGKILL. Vérifier d'abord les helpers existants. Sortie : test de démarrage/arrêt et assertion socket différente de production, aucun processus partagé touché.
- [ ] T003 Transcrire les scénarios métier et leurs variantes V01–V36 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/tests/features/102-fils-inter-agents.feature ; relier chaque scénario à un nom de test Rust dans test-plan.md. Sortie : scénario par propriété distincte, cas négatifs inclus ; pas d'ajout de moteur Python.

## Phase 2 — Fondations partagées

- [ ] T004 Écrire les tests spec102 du contrat dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-transport/src/protocol.rs : huit actions, champs inconnus, version, notice facultative, Register/Registered anciens et nouveaux, capacités absentes à reconnexion. Oracle : sérialisation ancienne conservée et refus nommés avant mutation (V22,V27,V33).
- [ ] T005 Implémenter les types ThreadRequest/Result et champs de négociation dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-transport/src/protocol.rs, thread_notice dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-core/src/message.rs ; mettre à jour les constructions affectées. Étendre client commun et matrices de rôles daemon, sans nouvel acteur libre ni handshake. Étendre aussi /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/communication.rs : canon conditionnel de la notice selon plan§5, golden historique inchangé et tests V20 de chaque champ de notice. Oracle : T004 vert, ancien DM inchangé, public incapable d'appeler ThreadRequest.
- [ ] T006 Écrire les tests de migration/idempotence transactionnelle dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/tests/spec102_threads_test.rs : base synthétique pré102, deux ouvertures, contraintes FK, rollback et concurrence des compteurs. Oracle V19,V20,V24,V31,V34 ; ne pas lancer sur base réelle.
- [ ] T007 Créer le sous-module SQL autorisé /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/store/threads.rs et le raccorder à Store::init_schema ; six tables/index de data-model.md, version distincte, une transaction pour entrée/opération/intention. Oracle T006 vert, ancien ledger intact, aucun second fichier DB ni fausse transaction entre deux connexions.

## Phase 3 — US1 : fil partagé silencieux (P1)

Test indépendant : quatre membres retrouvent les mêmes contributions, non-membre
refusé, zéro injection causée par create ou post silencieux.

- [ ] T008 [US1] Écrire V01–V03,V19–V21,V32 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/tests/spec102_threads_test.rs : create/list/show/post/close, ACL, auteurs attestés, cibles et reply_to validés, rejeux avant/après clôture. Sortie : échecs correspondant au comportement manquant, pas à un mauvais harnais.
- [ ] T009 [US1] Implémenter ces actions dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/threads.rs et raccorder lib.rs/daemon.rs au Store ; quotas dans transaction, normalisation limitée aux titres/ensembles, corps exact. Oracle T008 vert, aucun appel fournisseur silencieux et aucune donnée fil dans ledger global. FR-001,FR-002,FR-003,FR-011,FR-012,FR-017,FR-018.
- [ ] T010 [US1] Ajouter parse et commandes thread dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/cli.rs via communication/client.rs partagé ; flags stricts, --id préparé, résolution noms explicites sans ambiguïté. Les actions pas encore implémentées retournent un refus explicite temporaire, jamais un faux succès. Oracle V07,V33 pour actions US1, aucun --from ni socket libre.
- [ ] T011 [US1] Ajouter outil fermé bridget_thread, schémas et dispatch dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/mcp.rs ; étendre BRIDGET_SAFE_MCP_TOOLS à15 dans wrapper.rs et tests exacts. Oracle V22,V33, parité CLI/MCP, aucun accès Maicie/global MCP ajouté. FR-015.

## Phase 4 — US2 : mentions et alertes ciblées (P1)

Test indépendant : A↔B dans un fil A/B/C/D, aucun réveil C/D ; dix mentions avant
dispatch coalescées ; pas de réponse directe automatique aux alertes.

- [ ] T012 [US2] Écrire V04–V10,V24,V25 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/tests/spec102_threads_test.rs avec barrières/horloge de test : réservation figée, mention concurrente, crash après commit, issue inconnue puis nouvelle mention supérieure après échéance. Oracle : cible et génération exactes, aucune perte du pending_seq.
- [ ] T013 [US2] Implémenter état de coalescence thread_wakes dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/threads.rs et requêtes spécialisées Store : une ligne par membre/fil, génération figée, last_uncertain, ACK tardif corrélé, close/satisfaction par lecture. Oracle T012 partie état vert ; la même borne inconnue ne repart jamais sous nouvelle clé. FR-004,FR-005,FR-006,FR-007,FR-014.
- [ ] T014 [US2] Brancher projection des intentions vers remise099 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/daemon.rs : scope interne durable, instance/clefs figées, hors verrou/transaction, délai injection120s distinct du TTL de clé7jours ; DND/absence/capacité/busy explicites, 5départs/s et lot16 dans maintenance existante. Dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/idempotency/send_delivery.rs, exclure les notices typées de la réaffectation inter-instance, sans changer DM ; tests V25/V27 de remplacement effectif d'instance. Oracle T012 intégration vert, pas de polling LLM, absence de famine ni blocage DM. FR-013,FR-014,FR-017.
- [ ] T015 [US2] Adapter connect_and_register_at, réception typée et enveloppes dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/wrapper.rs pour Codex/Claude gérés et interactifs ; annonce[1] seulement chemin implémenté, capacité perdue à déconnexion, aucun relaisfinalDM d'alerte. Oracle V27,V29 avec faux providers ; publication d'une réponse seulement par post explicite.
- [ ] T016 [US2] Adapter enregistrement, envelope et pending dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/t3code.rs sur base101 intégrée ; étendre les tests t3code_098_test.rs/fixtures existantes sans toucher T3 réel. Oracle V28 : ACK injection observé mais aucune réponse finale relayée hors fil, observations101 conservées.
- [ ] T017 [US2] Protéger reply implicite dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/cli.rs et écritures du contexte de réponse des adapters : marqueur thread_notice dans fichier existant, ancien format DM lisible, thread_notice_not_replyable après alerte. Oracle V29 : ancien DM jamais visé par erreur, send explicite et prochaines réponses DM inchangés. FR-020.

## Phase 5 — US3 : lecture, confirmation et reprise (P1)

Test indépendant : curseur100 confirmé puis nouveaux101–110 seulement ; page
perdue relisible ; aucune ancienne confirmation ne saute une mention concurrente.

- [ ] T018 [US3] Écrire V11–V15,V26 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/tests/spec102_threads_test.rs : pagination nombre/octets, JSON échappé, double read, reçu courant rejoué, restart. Oracle : through_seq réel, snapshot figé, curseur inchangé tant que non confirmé.
- [ ] T019 [US3] Implémenter read/reconstruction de reçu dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/store/threads.rs et validation threads.rs ; un reçu actif, page complète≤60Kio, entrée indivisible, réservation10min. Brancher CLI/MCP déjà déclarés. Oracle T018 vert, aucune page vide qui boucle devant un gros corps. FR-008.
- [ ] T020 [US3] Écrire V16,V17 et course ACK20/mention21 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/tests/spec102_threads_test.rs : dernierACK rejoué, ACK tardif courant accepté/remplacé refusé, acteur/fil incorrect, transaction post+ACK qui échoue. Oracle : aucun saut/perte/avance partielle.
- [ ] T021 [US3] Implémenter ACK et ack_receipt joint au post dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/store/threads.rs et branches threads.rs ; vérifier dernierACK avant recherche du reçu actif supprimé, CAS base_seq, préserver pending supérieur. Oracle T020 vert et V17/V19 rejeu post exact après suppression du reçu. FR-009,FR-011.
- [ ] T022 [US3] Implémenter history avec tests V18 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/threads.rs et requêtes Store : plage explicite, to_seq figé, pagination sans reçu ni déplacement du curseur ; compléter façades CLI/MCP. Oracle : reprise de contexte et lecture de synthèse possibles sans réinitialiser acked_seq. FR-010.
- [ ] T023 [US3] Exécuter et compléter les points de coupure V12,V19,V24–V26 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/tests/spec102_threads_test.rs ; restart daemon simulé, réponse perdue et ACK tardif après nouvelle génération. Sortie : preuves durables avant/après, zéro post dupliqué et états inconnus nommés ; aucune terminaison dangereuse. FR-014,SC-003.

## Phase 6 — US4 : synthèse demandée et usage agent (P2)

Test indépendant : un membre lit la plage demandée et sait rendre une synthèse
sourcée, éventuellement partielle, sans lancement d'agent ou génération automatique.

- [ ] T024 [US4] Documenter les recettes de synthèse et de lecture/ACK dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/skills/bridget/references/commandes.md : destination humaine/fil, plage et désaccords, pas de consensus inventé ni @all implicite ; exemples JSON validés. Oracle V30,V36, aucune action summary ou dépendance modèle ajoutée. FR-016.
- [ ] T025 [US4] Mettre à jour /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/skills/bridget/SKILL.md et README.md avec choix silencieux/ciblé/all, réception≠lecture≠réponse, absence de garantie cache, catalogue15 et compatibilité de version ; liens aux recettes. Préserver publications100/101 lors intégration. Oracle V36 et core_089_skill_test ; ne pas casser ses quatre exemples JSON sans adapter explicitement le test. FR-019.

## Phase 7 — Recette transversale et livraison future

- [ ] T026 Vérifier chaque limite N/N+1 et accès restant à saturation dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/tests/spec102_threads_test.rs ; ajouter tests unitaires de validation manquants dans threads.rs. Oracle V14,V31, quotas atomiques malgré create/post concurrents ; compteurs cohérents. FR-017.
- [ ] T027 Vérifier autorisations, contenu inerte, absence de fuite et impossibilité de forger thread_notice dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/tests/spec102_threads_test.rs ; matrice rôles complète, token/reçu jamais loggé. Oracle V21–V23, SC-006 ; pas d'exécution de texte ou fuite corps/titre via ledger/erreurs.
- [ ] T028 Finaliser tests de parité/inventaire dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/mcp.rs et wrapper.rs, commandes CLI et compatibilité au registre ; vérifier les huit actions et permissions15 exactes. Oracle V27,V33 : ancien client garde DM, nouveau ne prétend pas disposer d'un catalogue MCP ancien.
- [ ] T029 Ajouter/exécuter la recette locale V35 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/tests/spec102_threads_test.rs ; publier p50/p95/max et DM témoins dans implementation.md. Oracle SC-005 avec jeu10000entrées et200opérations ; requêtes indexées, mémoire bornée, aucun scan de corps au réveil.
- [ ] T030 Exécuter fmt, tests ciblés102, clippy et régressions sûres089/094/097/098/099/100/101 selon /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/specs/102-fils-inter-agents/quickstart.md ; enregistrer chaque commande, pass/fail et exclusions justifiées dans implementation.md. Ne pas inventer de réussite workspace si harnais dangereux ou ressources insuffisantes.
- [ ] T031 Effectuer Analyze puis Converge face au code/test réel et revue adverse si joignable ; consigner couverture FR/SC et corrections dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/specs/102-fils-inter-agents/analysis.md et implementation.md. Oracle : aucun HIGH/CRITICAL non traité, tous les scénarios obligatoires vérifiés ; une tâche manquante reste ouverte, pas de déclaration prématurée.
- [ ] T032 Préparer remise à l'humain dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/specs/102-fils-inter-agents/implementation.md : limites, versions/capacités nécessaires, commandes de recette, diff et étapes éventuelles d'adoption sauvegardée. Mettre à jour ADR038/statuts seulement selon preuves. Aucun commit/fusion/déploiement sans nouvelle autorisation.

## Dépendances et parallélisme possible

T001 → T002–T003 → T004–T007 → US1(T008–T011) → US2(T012–T017)
→ US3(T018–T023) → US4(T024–T025) → recette(T026–T032).

Ne pas livrer US1 seul comme fonctionnalité complète ; c'est seulement le premier
jalon testable. L'ordre US2 avant US3 utilise des destinataires simulés : pas de
recette auprès d'agents réels tant que lecture/ACK ne fonctionnent pas.

Aucune tâche marquée [P] par défaut : Store/daemon/wrapper sont partagés. Après
T014, T016 (T3) peut être confiée à un second agent pendant T015 (wrapper) SI le
contrat de connexion est figé et les propriétés des fichiers explicitement séparées.
US4 références (T024) et README peuvent être rédigées séparément après stabilisation
des contrats, mais un seul propriétaire de la skill. Aucun lancement d'équipe
implicite ; règles de délégation du projet applicables.

## Couverture exigence → tâches

| Exigence | Tâches |
|---|---|
| FR-001 | T008,T009,T010,T011 |
| FR-002 | T006,T007,T008,T009 |
| FR-003 | T008,T009,T012 |
| FR-004 | T012,T013,T014 |
| FR-005 | T010,T012,T013 |
| FR-006 | T012,T013,T014 |
| FR-007 | T012,T013,T014,T023 |
| FR-008 | T018,T019,T026 |
| FR-009 | T018,T019,T020,T021 |
| FR-010 | T022 |
| FR-011 | T006,T007,T008,T009,T020,T021,T023 |
| FR-012 | T004,T005,T008,T009,T020,T027 |
| FR-013 | T012,T014,T015,T016 |
| FR-014 | T006,T007,T012,T023 |
| FR-015 | T004,T005,T010,T011,T015,T016,T028 |
| FR-016 | T022,T024 |
| FR-017 | T007,T009,T014,T019,T026,T029 |
| FR-018 | T008,T009,T013 |
| FR-019 | T024,T025,T028 |
| FR-020 | T015,T016,T017,T027,T028,T030 |
| SC-001 | T008,T012,T014 |
| SC-002 | T018,T019,T021 |
| SC-003 | T006,T020,T023 |
| SC-004 | T012,T013,T020 |
| SC-005 | T029 |
| SC-006 | T004,T008,T020,T026,T027 |
| SC-007 | T024,T025,T028 |

T001–T003 et T030–T032 sont les tâches transversales de préparation, sécurité
et preuves ; elles ne représentent pas de fonctionnalités supplémentaires.
Évitements explicites : aucun broker, résuméLLM automatique, APIadmin, GUI,
workflowMaicie, abstractiongénérique de groupes ni dépendance nouvelle.
