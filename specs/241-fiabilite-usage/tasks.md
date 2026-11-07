# Tâches 241 — Confier un travail sans administrer Bridget

Statut : In Progress. Aucune tâche métier historique ne sert de banc destructif.
Les chemins ci-dessous sont relatifs au worktree de la session. Les commandes
Cargo passent toutes par python3 scripts/build.py cargo. Une case exige sa preuve.

## Préparation

- [x] T001 Conserver cadrage, spec, plan, recherche, contrats et audit de réutilisation dans specs/241-fiabilite-usage/ ; préciser les exclusions et les mécanismes existants. Vérifié : artefacts présents, reuse-audit PASS, pas de doublon proposé.
- [x] T002 Exécuter Analyze sur specs/241-fiabilite-usage/spec.md, plan.md et tasks.md ; corriger les écarts non ambigus puis relire. Vérifié : 13 FR et 7 SC couverts, aucun CRITICAL persistant, compte rendu analyze.md.

## US1 — Raccordement, état et récupération

- [x] T003 [US1] Mesurer les phases réelles de communication/client.rs, t3code_mcp.rs, daemon.rs et scripts/federate-ssh.sh ; capturer un délai ou reproduire sa cause dans le vrai parcours de sockets. Conserver les preuves privées et une conclusion causale dans validation.md ; un succès ultérieur ne clôt pas l'inconnue. FR-001.
- [x] T004 [US1] Corriger la cause confirmée dans les composants existants et supprimer l'entretien synchrone sans rapport du chemin de réponse si sa dépendance est écartée ; test d'une inscription pendant une vue lente dans daemon.rs. FR-001, FR-008.
- [x] T005 [US1] Réutiliser registered_connection_until dans communication/client.rs pour la reprise automatique des défaillances transitoires avant envoi avec un seul budget ; tester refus d'identité, annulation MCP et mutation non envoyée une seule fois. FR-002.
- [x] T006 [US1] Vérifier la récupération 171/232 par vrais sockets et magasin temporaire dans delegation_mcp.rs/daemon.rs : coupure avant envoi, réponse perdue après enregistrement, même clé concurrente, état et annulation accessibles. Zéro mission dupliquée, preuves par phase. FR-003, SC-001.

## US2 — Autorité durable d'une mission

- [x] T007 [US2] Arrêter et documenter le marqueur interne de Task, les sources natives de révocation et le retour dans contracts/admission.md, data-model.md et docs/decisions/241-autorite-admission.md ; aucune nouvelle table ou preuve HTTP persistée. FR-004, FR-005.
- [x] T008 [US2] Ajouter les régressions avant correction dans les fixtures existantes de native_delegation.rs/native_permission_change.rs : fin de tour, tour suivant à droits équivalents, révocation propriétaire/instance/racine, annulation et source de permissions changée. FR-004, FR-005, SC-002.
- [x] T009 [US2] Étendre Task et l'admission attestée dans delegation.rs/native_delegation.rs ; appliquer l'autorité de mission au démarrage et à native_start_recovery.rs sans réutiliser un ancien tour pour une nouvelle demande. Vérifier même propriétaire, snapshot, placement, définition et contrôles natifs avant Release. FR-004, FR-005.
- [ ] T010 [US2] Vérifier réouverture de la base, compatibilité du marqueur, anciennes tâches non promues et image de retour capable de traiter le contrat ; tests de lecture/reprise dans delegation.rs et protocoles existants. Aucun retour réel pour tester. SC-002, FR-013.

## US3 — Dossiers autorisés et capacités utiles

- [x] T011 [US3] Reproduire le refus Mac avec parent+pièces jointes attestés et mapping local sans additional_root dans native_permissions.rs ; tester aussi origine étrangère, chemin protégé, lien, répertoire disparu et répertoire non attesté. FR-006.
- [ ] T012 [US3] Corriger la projection locale dans native_permissions.rs et ses appelants de remote_worker_read.rs/remote_worker.rs en liant explicitement origine=cible authentifiée ; garder la traduction autorisée obligatoire à distance. Le prochain appel ordinaire doit passer sans éditer un registre. FR-006, FR-010.
- [x] T013 [US3] Vérifier et corriger le parcours general/shared hors Git déjà autorisé dans lifecycle.rs, origin_proof.rs et scripts/prepare-project.py ; source dans une racine supplémentaire attestée, refus précis d'une source réellement extérieure. Aucun faux dépôt, déplacement destructif ou autorisation nouvelle. FR-006, FR-010.
- [x] T014 [US3] Inventorier nombre, volume, durée, concurrence et rétention dans specs/241-fiabilite-usage/limits.md, y compris origin_proof, protocol, workspace_git, commit_view, t3code, handoff et service_limits. Décision et preuve par plafond : conserver, supprimer, adapter ou paginer. FR-007, SC-005.
- [x] T015 [US3] Appliquer les suppressions/adaptations de T014 dans les composants existants, notamment l'arrêt au volume sélectionné quand le contrôle de disque et le traitement en flux suffisent ; vérifier écrivains ET lecteurs et les erreurs de ressources réelles. FR-007, FR-008.
- [x] T016 [US3] Vérifier les lectures par clé et les pages au-delà des bornes historiques dans delegation.rs, delegation_lineage.rs et lecteurs MCP concernés ; aucune collection volumineuse ne rend indisponible le statut d'une tâche distincte. FR-008, SC-003.

## US4 — Outillage réellement disponible

- [x] T017 [US4] Définir les exigences de commandes identifiables et leurs limites dans contracts/workload-tools.md ; rechercher les préflights et manifestes existants, retenir un contrat négocié interne sans nouveaux paramètres obligatoires pour l'appelant. FR-009.
- [x] T018 [US4] Étendre CandidateProbeV2/capacités dans bridget-transport et les sondes de remote_worker_read.rs : outils absents, présents mais inutilisables, inconnus ou disponibles. Commandes fixes en lecture seule, budget/cache par lot, aucun shell du dépôt exécuté. FR-009.
- [x] T019 [US4] Raccorder l'identification des prérequis et la qualification à native_delegation.rs avant la sélection finale ; respecter la machine imposée, ne pas transformer une mention facultative/négative en exigence et vérifier les cas Docker et compilation sans Docker. Aucun installateur. FR-009, SC-004.

## US5 — Parcours complets et livraison

- [x] T020 [US5] Faire passer les scénarios complets dans les fixtures de vrais échanges, dont lecture/annulation pendant une préparation et réponse tardive sur autre mission ; réutiliser les preuves 240 inchangées. Publier essais directs/récupérés/refusés, délais et interventions dans validation.md. FR-011, FR-012, SC-003, SC-006.
- [ ] T021 [US5] Exécuter les vérifications Rust/Python utiles et les assemblages via scripts/build.py ; comparer les résultats avant/après dans les mêmes conditions. Pas de répétition générale sans changement ou risque nouveau. FR-012.
- [ ] T022 Exécuter Converge après la dernière modification, demander la contre-revue adverse de l'implémentation et traiter les objections vérifiées ; consigner CONVERGED ou ajouter les tâches manquantes sans changer le périmètre.
- [ ] T023 Exécuter l'audit applicable sur le code final, ses chemins d'erreur, sa complexité et sa charge cognitive ; corriger les findings bloquants et conserver les preuves dans specs/241-fiabilite-usage/. Après toute correction, réexécuter T022/Converge et les contrôles affectés.
- [ ] T024 Intégrer sur main et pousser les seuls fichiers autorisés ; vérifier les refs distantes. Construire/publier la version et le retour compatibles, activer les sept composants sur les quatre hôtes avec préservation des registres, superviseurs, tâches et pointeurs natifs. FR-013, SC-007.
- [ ] T025 Vérifier les images/processus réellement chargés, le lien natif, les façades et les scénarios réels autorisés sans redémarrer T3 ; informer les propriétaires concernés avec reçus, sans mission métier rejouée. Toute activation impossible reste nommée incomplète. FR-010, FR-013.
- [ ] T026 Aligner les sources des quatre hôtes, retirer worktree/branche intégrés et caches/copies inutiles avec les outils existants ; conserver preuves, actif, retour, travaux modifiés/utilisés. Vérifier main propre, push et bilan réel dans validation.md.

## Dépendances et exécution

T001 → T002 → US1 → US2 → US3 → US4 → US5 → livraison/nettoyage.
Les observations et lectures sans mutation de US2/US3 ont déjà servi au plan.
Une tâche causalement bloquée est nommée comme telle ; poursuivre les suivantes
indépendantes, sans déclarer le chantier complet. Aucun MVP ne clôt cette liste.

Parallélisme utile : seule la revue indépendante porte un livrable distinct.
Un responsable conserve l'implémentation pour éviter les conflits sur daemon.rs.
Les mesures du relais et du cœur peuvent être collectées ensemble ; aucun agent
supplémentaire ni réveil périodique pour surveiller les autres projets.

## Vérifications prévues

Les noms précis des tests seront fixés au point d'ajout dans les modules existants.
Commandes : python3 scripts/build.py cargo test -p bridget-daemon --lib spec241 -- --nocapture ;
python3 scripts/build.py cargo test -p bridget-transport --lib -- --test-threads=1 ;
python3 -m unittest pour les fichiers Python concernés ; bash scripts/test-federate-ssh.sh
si le relais est modifié. Vérifier les commandes de projet avant exécution.
Un test non exécuté reste bloqué avec sa commande et son erreur, pas coché.
