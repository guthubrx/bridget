# Archives techniques de l’adaptateur T3

Document d’origine : [specs/102-fils-inter-agents/tasks.md](../../../../../../specs/102-fils-inter-agents/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [x] T012 [US2] Écrire V04–V10,V24,V25 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/tests/spec102_threads_test.rs avec barrières/horloge de test : réservation figée, mention concurrente, crash après commit, issue inconnue puis nouvelle mention supérieure après échéance. Oracle : cible et génération exactes, aucune perte du pending_seq.
- [x] T013 [US2] Implémenter état de coalescence thread_wakes dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/threads.rs et requêtes spécialisées Store : une ligne par membre/fil, génération figée, last_uncertain, ACK tardif corrélé, close/satisfaction par lecture. Oracle T012 partie état vert ; la même borne inconnue ne repart jamais sous nouvelle clé. FR-004,FR-005,FR-006,FR-007,FR-014.
- [x] T014 [US2] Brancher projection des intentions vers remise099 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/daemon.rs : scope interne durable, instance/clefs figées, hors verrou/transaction, délai injection120s distinct du TTL de clé7jours ; DND/absence/capacité/busy explicites, 5départs/s et lot16 dans maintenance existante. Dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/idempotency/send_delivery.rs, exclure les notices typées de la réaffectation inter-instance, sans changer DM ; tests V25/V27 de remplacement effectif d'instance. Oracle T012 intégration vert, pas de polling LLM, absence de famine ni blocage DM. FR-013,FR-014,FR-017.
- [x] T015 [US2] Adapter connect_and_register_at, réception typée et enveloppes dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/wrapper.rs pour Codex/Claude gérés et interactifs ; annonce[1] seulement chemin implémenté, capacité perdue à déconnexion, aucun relaisfinalDM d'alerte. Oracle V27,V29 avec faux providers ; publication d'une réponse seulement par post explicite.
- [x] T016 [US2] Adapter enregistrement, envelope et pending dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/t3code.rs sur base101 intégrée ; étendre les tests t3code_098_test.rs/fixtures existantes sans toucher T3 réel. Oracle V28 : ACK injection observé mais aucune réponse finale relayée hors fil, observations101 conservées.
- [x] T017 [US2] Protéger reply implicite dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/crates/bridget-daemon/src/cli.rs et écritures du contexte de réponse des adapters : marqueur thread_notice dans fichier existant, ancien format DM lisible, thread_notice_not_replyable après alerte. Oracle V29 : ancien DM jamais visé par erreur, send explicite et prochaines réponses DM inchangés. FR-020.

<a id="extrait-2"></a>

## Extrait 2

Aucune tâche marquée [P] par défaut : Store/daemon/wrapper sont partagés. Après
T014, T016 (T3) peut être confiée à un second agent pendant T015 (wrapper) SI le
contrat de connexion est figé et les propriétés des fichiers explicitement séparées.
US4 références (T024) et README peuvent être rédigées séparément après stabilisation
des contrats, mais un seul propriétaire de la skill. Aucun lancement d'équipe
implicite ; règles de délégation du projet applicables.
