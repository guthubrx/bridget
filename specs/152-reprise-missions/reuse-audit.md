# Audit de reutilisation de l’existant — 152-reprise-missions

## Decision

Statut: PASS
Date: 2026-10-11
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions

Conclusion courte: les 26 items du plan sont confrontés au code réellement présent
dans le worktree 152, base cd2bb015755fde7af26cf87fdb8f4e658172e237.
Les apports 148 à 151 sont présents. Les hypothèses périmées sont corrigées
dans les documents ; aucun doublon évident non arbitré. Les risques de reprise
restent des preuves à obtenir dans les tests déjà prévus, avant leur code.

Le prérequis officiel de la skill est absent. Les chemins sont résolus à la main
depuis le worktree explicite. Les 50 tâches préexistaient à cette passe : ce reçu
est un gate frais avant implémentation, pas une génération rétroactive de tâches.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 26 |
| Items audites | 26 |
| Duplications evidentes non arbitrées | 0 |
| Règles et mémoires applicables recensées | 9 |
| Specs existantes applicables vérifiées | 5 |
| Nouvelles tâches ou dépendances créées par l’audit | 0 |

## Reutilisations correctement identifiees

Les références de source sont celles de la base actuelle, avant tout code 152.
« Ajouter » décrit du travail prévu, pas une fonction déjà livrée.

| ID | Item du plan | Citation du plan | Existant et preuve | Décision |
|---|---|---|---|---|
| P01 | Champs de tâche et compteur durable | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:63 | Task, serde(default), acknowledged_at — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/delegation.rs:12 | Étendre le payload existant ; préserver l’accusé 151 et les sauvegardes projetées. |
| P02 | Date du premier lancement | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:44 | spawn_task(st, task) — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:823 | Ajouter launch_issued_at ; garder le canon d’une saga existante. |
| P03 | Date du premier envoi de mission | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:54 | tick, admission de la mission — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1450 | Ajouter mission_issued_at avant le premier envoi ; même enveloppe au rejeu. |
| P04 | Garde de remise autonome | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:38 | admit_autonomous_effect, NativeDelegation — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/referent_control.rs:107 | Ajouter NativeMissionDispatch au même bras de garde, sans recopier sa logique. |
| P05 | Calcul durable du temps de pause | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:55 | control_events et control_state — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/referent_control.rs:165 | Ajouter pause_overlap au module existant ; ne pas utiliser history limité aux derniers événements. |
| P06 | Index temporel de pause | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:88 | Schéma et migrations de control_events — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/referent_control.rs:165 | Index (at, id) pour pause_on/pause_off après migration ; coût O(log N + P). |
| P07 | Échéance effective de mission et descendants | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:55 | mission_deadline_at depuis l’admission — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1287 | Ajouter le chevauchement ; conserver la date brute, réutiliser pour la borne imbriquée. |
| P08 | Gel du suivi en mémoire | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:43 | PendingReply, restore_pending_for_agent, collect_reminder_actions — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:1624 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:4107 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:7184 | Ajouter frozen_secs, affectation absolue depuis la demande durable ; aucun second moteur. |
| P09 | Échéance métier de RequestWait151 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:82 | wait_request et request_wait_snapshot — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:289 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/store/ledger_requests.rs:272 | Calculer l’échéance native effective sans mutation ; wait_timeout inchangé. |
| P10 | Dernière exécution de la chaîne | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:42 | CTE execution_budget_facts et reconstruct_active_for_agent — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/execution_store.rs:818 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/execution_store.rs:1298 | Ajouter latest_in_chain ; feuille de plus grande génération, pas tri par date/id. |
| P11 | Consommer une interruption abandonnée | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:66 | record_pause_interruption et pending_pause_interruptions — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/execution_store.rs:1488 | Ajouter abandon_pause_interruption, sans nouvelle table ni nouvel état. |
| P12 | Reprise seulement après interrupted | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:41 | garde de schedule_execution_recovery — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:4540 | may_resume couvre la chaîne et refuse annulation ; garde de pause durable. |
| P13 | Deux niveaux du refus de remise | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:52 | rejected puis gestionnaire DeliveryRejected — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1099 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:18452 | Qualification booléenne puis retour avant retrait du suivi et avis au parent. |
| P14 | Boucle de reprise | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:41 | tick natif appelé à la seconde — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:4992 | Réutiliser la boucle ; aucun nouveau thread ni temporisateur. |
| P15 | Décision de reprise après panne | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:45 | prepare_restart(st) -> Result<HashSet<String>, String> — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1211 | Remplacer l’échec systématique ; compteur sauvegardé avant tout effet ; bail expiré refusé. |
| P16 | Identités au démarrage et marqueurs | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:56 | restart_identities, réconciliation des groupes — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1119 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/managed_process.rs:1233 | Échec nommé de tâche ; vérifier le marqueur avant publication. Exclure seulement la map ne ferme pas self.load ; corruption/illisibilité hors FR-029, refus sûr conservé. |
| P17 | Deux chemins de reprise de flotte | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:49 | reserve_managed_recoveries et prepare_recovery — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:4174 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/lifecycle.rs:313 | Préserver candidat en vol et enfant connecté ; environnement et propriété sur les deux chemins. |
| P18 | Environnement et autorisations de reprise | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:46 | contrôles de spawn_task puis bloc NativeMissionBootstrap — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:824 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:883 | Extraire seulement le bloc env ; refaire les contrôles révoqués/sources/cwd avant reprise. |
| P19 | Propriété de la génération connectée | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:68 | ownership_from_desired(&DesiredEquipier) -> Option<SpawnOwnership> — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/fleet.rs:1666 | Fonction privée : visibilité crate prévue dans T024 ; aucune copie. |
| P20 | Lecture du lien attesté et commande courante | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:78 | IdempotencyStore::agent_link_by_id — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/idempotency/agent_links.rs:96 | Réutiliser l’API déjà présente ; pas de nouveau SQL ni façade sans règle métier. |
| P21 | Classification pure avant arrêt | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:47 | ManagedStopTarget, prepare_managed_stop, await_managed_stop — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:1243 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:9561 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:9626 | Extraire classificateur sans effets ; garder Stopped sans superviseur et documenter coût réel. |
| P22 | Annulation depuis la racine et reçus des arrêts | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:57 | cancel_tree et collecte des tâches/liens — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1672 | Vérifier tous les membres avant mutation ; conserver erreur terminale ; cancel_stopped durable. |
| P23 | Reçus des cinq refus définitifs | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:48 | admission, tombstones et reçus d’annulation existants — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:660 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/delegation_lineage.rs:93 | Table distincte justifiée : les faits ne sont pas ceux des tombstones ; aucun reçu transitoire. |
| P24 | Entretien des reçus à 30 jours | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:69 | maintain_retention et run_maintenance — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1909 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:7498 | Réutiliser le passage 150 ; aucune nouvelle boucle d’entretien. |
| P25 | Vue courante et Lineage compatible | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:85 | task_view pur, current_task_view, visible_changed/save_projected — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:73 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:126 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/delegation_lineage.rs:69 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/delegation_lineage.rs:269 | suspended calculé dans la vue courante ; pas d’écriture de suspension ni changement Lineage ; epoch après commit. |
| P26 | Tests, documentation et mémoire architecturale | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:70 | Modules et E2E existants, documentation native, ADR007/027 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1921 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/tests/native_delegation_e2e.rs:1 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/docs/delegation-native.md:1 | Tests native152 dans les fichiers existants ; ADR152 attendue, pas de module Rust ni dépendance nouvelle. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision |
|---|---|---|---|
| Propriété de reprise connectée | ownership_from_desired existe mais est privée | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/fleet.rs:1666 | Visibilité crate dans T024 et fichier flotte ajouté au plan. |
| Lecture de lien R24 | agent_link_by_id est déjà livrée | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/idempotency/agent_links.rs:96 | Lever l’hypothèse d’API manquante ; réutiliser le magasin. |
| Table des demandes supposée immuable | align_request_deadline est livré par150 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1780 | Conserver l’alignement brut explicite ; aucun décalage relatif de pause. |
| Attente de réponse 151 | wait_request calcule l’expiration brute | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:289 | T018 calcule l’échéance native effective ; budget de l’appelant intact. |
| Démarrage non fatal | self.load et second contrôle du marqueur | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/managed_process.rs:1233 | T043/T048 ferment les incohérences natives lisibles ; fichiers corrompus hors scope, exclusion seule insuffisante. |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucun doublon non arbitré | — | Passage source et recherche de symboles152 | Aucun blocage de réutilisation |

NativeMissionDispatch reste un effet nommé distinct de l’admission NativeDelegation.
Son bras de garde est partagé. Les nouvelles fonctions latest_in_chain et
pause_overlap portent une lecture métier absente ; les extractions env/arrêt
ont chacune deux usages et une contrainte d’identité ou de pureté documentée.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/AGENTS.md | Français, worktree, scope et protection des fils actifs | Docs152 uniquement ; principal propriétaire de Git/tests/livraison. |
| /Users/moi/.speckit/constitution.md | III, VII, XV, XVI | Spec/plan/tasks avant code ; ADR ; preuve avant cochage ; livraison et nettoyage par session. |
| /Users/moi/.speckit/constitution.md | XVIII | Index temporel et coûts réels ; pas de promesse O(m) sans preuve. |
| /Users/moi/.speckit/constitution.md | XIX et XX | Réutilisation ; aucune couche, dépendance ou tâche de confort ; six questions traitées dans Analyze. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/.specify/memory/constitution.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/.specify/memory/standards.md | Ponts vers les références utilisateur | Sync152 RC0 signalée par le principal ; sources globales chargées. |
| /Users/moi/.speckit/ref/standards-tests.md | Tests du comportement et des erreurs | Cas intégrés aux scénarios natifs existants ; mocks aux frontières. |
| /Users/moi/.speckit/ref/code-quality-details.md | Contrats, erreurs et idempotence | Effets après validation ; conserver les faits durables 150/151. |
| /Users/moi/.speckit/ref/speckit-workflow.md ; /Users/moi/.speckit/ref/adversarial-review.md | Statuts canoniques et revue hostile | Ready et compteurs canoniques ; objections tracées dans leurs tâches. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md ; /Users/moi/.speckit/ref/agent-orchestration.md | Garde unique et un passage Explore pour la source | Une seule exploration déléguée en lecture seule ; aucun sous-agent producteur. |

Aucun MEMORY.md Bridget trouvé dans /Users/moi/.Codex/projects lors de la
recherche ciblée. Cette absence n’est pas une règle manquante inventée. La recette
privée est confirmée par les commandes et journaux de baseline.

## Specs livrees applicables

| Spec | Pattern | Impact |
|---|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/087-reprendre-controle/spec.md | ADR027, pause durable et interruption | Réutiliser les tables/gardes et le filtre spec_087. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/148-identite-delegation/spec.md | Identité durable et mission en un appel | Conserver identifiant, instance attestée et canon. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/149-sous-agents-lineage/spec.md | Filiation, permissions héritées, projection Lineage | Rechecks à la reprise ; projection inchangée. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/150-socle-delegation/spec.md | Délais explicites, alignement durable, rétention et garde | Préserver ces apports ; réutiliser entretien et garde. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/151-attente-fin-de-tache/spec.md | WaitEpoch, vues pures, EOF et accusé explicite | Calcul métier gelé sans toucher budget/ACK/epoch après commit. |

Les textes proches sont lus pour les points de contact ; leur code est
présent. Baseline actuelle : 239 PASS/0 FAIL/1 ignoré sur six filtres,
exécutée par le principal et décrite dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/baseline.md.
Aucune recette de modèle réel ni validation de production déduite.

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| rg --files, puis lecture constitution/standards/artefacts | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions et /Users/moi/.speckit | Instructions et 50 tâches existantes chargées. |
| check-prerequisites.sh --json --paths-only, résolution manuelle | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/.specify/scripts/bash/check-prerequisites.sh | Script absent ; pas d’installation ni de helper de remplacement. |
| rg --files --hidden --no-ignore, noms check-prerequisites/common | /Users/moi/.speckit/scripts ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.specify ; /Users/moi/Documents/MAICompany/runtimes/speckit-portfolio-679aabe2-r14 | Aucune copie stock portable trouvée. |
| Recherche groupée des symboles152 et analogues | Sources des modules natifs, flotte, exécution, contrôle, demandes | latest_in_chain/effective_deadline/pause_overlap/native_child_env/classify_stop/champs152/refus152 absents ; réutilisations ci-dessus présentes. |
| Comparaison plan/R22–R28/code livré150–151 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions | Dates, visibilité, API de lien, pureté des vues et marqueurs rectifiés. |
| Lecture JSON et compteurs des journaux existants | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/baseline-results.json | Six filtres RC0 ; aucun lancement de test par cet audit. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|
| Travaux152 à160 | Un worktree et une livraison/nettoyage par session | Dernier contrat utilisateur coordonné par le principal ; anciennes références150–158 remplacées. | 2026-10-11 |
| Variante NativeMissionDispatch | Maintenue avec bras de garde partagé | D3 distingue deux effets métier sans duplication. | 2026-10-11 |
| Refus durables | Table dédiée, cinq codes fermés | Tombstones et reçus d’annulation attestent d’autres faits ; fusion trompeuse. | 2026-10-11 |
| Marqueur illisible | Ne pas prétendre fermé par exclusion de la map | self.load reste appelé ; Les incohérences natives lisibles sont couvertes par T048 ; corruption/illisibilité hors FR-029, refus sûr existant conservé. | 2026-10-11 |
| Nouveaux tests/outils de confort | Aucun | Les cas nécessaires sont dans les tâches et tests déjà prévus. | 2026-10-11 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne dans l’audit (26/26)
- [x] Memoires et regles projet lues et appliquees
- [x] Specs livrees proches verifiees pour reutilisation/patterns
- [x] Plan refactore ou divergences justifiees avant le prochain travail de code

Les 50 tâches étaient déjà présentes. Ce gate les confronte au code frais avant
implémentation. Première tâche restante : T002, puis les cas rouges requis.

## Complément T004 après R4 — montage privé de permissions réutilisé de149

Ce complément précise P26 et le test T004 existant. Les 26 items et le gate
initial restent conservés. La première tâche indiquée ci-dessus décrit le
moment de ce gate, avant implémentation ; ce n’est pas l’avancement courant.
La recette E2E de reprise imbriquée reste ouverte après R4, malgré la mission
réellement remise. Son refus permission_source_unavailable est correct pour
la fixture héritée non attestable.

| Besoin contraint T004 | Existant relu | Décision de réutilisation |
|---|---|---|
| CLI directe, profil/PATH privés et borne des ancêtres | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs:42, lignes 42–61 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/native_permissions149_tests.rs:26, lignes 26–35 | Reprendre .git privé, bin/claude dans PATH et CLAUDE_CONFIG_DIR privé. Aucun accès aux réglages utilisateur. |
| Contexte réel du CLI et sources de réglages | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs:133, lignes 133–134 | Utiliser la capture existante, avec chemin CLI, SHA et révisions réelles. Ne pas fabriquer un launch_context ou snapshot. |
| Snapshot figé par l’admission | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs:405, lignes 405–412 | Obtenir les permissions par faits attestés puis admission réelle. Conserver tous les rechecks. |
| Fournisseur privé capable de poursuivre la mission et déléguer | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/tests/native_delegation_e2e.rs:438, compile_provider152 ; fournisseur Python148 existant incorporé | Compléter seulement la fixture T004 par un binaire compilé qui incorpore son Python, avec init/settings observés. Aucun nouvel outil produit ni scénario autonome. |
| Compilateur déjà requis | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/Cargo.toml:18, rusqlite bundled ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/Cargo.toml:29, héritage workspace | /usr/bin/cc et /usr/bin/clang disponibles selon le principal. Aucune nouvelle dépendance ou installation. |

Arbitrage autorisé par le principal avant la prochaine recette : l’empreinte
du CLI compilé couvre le Python incorporé. La capture de permissions passe
par la voie réelle CLI/settings/init. Aucun snapshot injecté ni garde changée.
Le propriétaire qualité possède seul les changements E2E ; cet audit n’écrit
aucun code. La lecture du montage ne prouve pas sa compilation ou son succès.

Statut du complément : réutilisations identifiées, décision bornée au T004
déjà prévu ; **recette du nouveau montage non encore validée**. Aucun nouveau
test, ID ou outil produit n’est ajouté au plan. Les tâches restent 24/50.
La preuve R4 et son historique restent dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

## Validation ciblée du montage T004 après R6

Le montage149 a maintenant été exécuté. R5 observe le CLI compilé privé,
son empreinte, le profil/settings et l’init réel de la fixture. Ces fichiers
ne constituent pas un fact mémoire publiquement sérialisé ou accepté.
Son refus permission_attestation_unavailable révèle une garde de transports
qui excluait claude_stream_json, pourtant rapporté par le registre réel.

Après correction minimale de cette garde, R6 donne **5 PASS/0 FAIL E2E**,
RC0. L’E2E garde la même SHA que R5. Le montage privé permet deux reprises,
la vérification de l’identité/bootstrap/filiation et l’admission MCP du
descendant après chacune. Aucun snapshot injecté ni nouvelle dépendance
ne remplace la capture réelle. Les autres validations de la garde restent
présentes dans le code relu.

Ce GREEN clôt la recette ciblée du montage T004 ; il ne clôt pas encore
toute US3. T025 attend sa preuve de résultat unique après panne.
Les tâches et le gate initial restent inchangés, 24/50. Aucun modèle réel
ou runtime de production n’est déduit de ce fournisseur de fixture.
Les empreintes, lignes de code, oracles et limites sont conservés dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

## Clôture documentaire US3 après T025 R4

Le complément T025 est maintenant GREEN : 1 PASS/0 FAIL, RC0. Il vérifie
le résultat rendu une seule fois après la panne, avec vrais ACK, entretien
du magasin, état/flags durables et compteur1. Les corrections R1/R2/R3
concernaient seulement la fixture ; la production garde son préfixe antérieur.
Les refus de permissions R4/R5 et le GREEN de montage R6 restent conservés.

Après autorisation du principal, les onze tâches US3 et rouges associés
sont cochés, progression **35/50 (70 %)**. Le gate de réutilisation initial
et ses 26 items ne changent pas. La réserve T025 du reçu précédent est
historique et désormais close. Aucun nouveau test autonome, ID, dépendance
ou outil produit n’est ajouté par cette clôture documentaire.

La preuve finale et les huit captures T025 sont consignées dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

## Clôture documentaire US4 — réutilisation de la classification et des magasins

Les trois filtres US4 R1 sont GREEN : 4 PASS annulation, 2 PASS rouges
annulation et témoin148 1 PASS, tous RC0. La classification existante est
extraite sans mutation et conserve Desired Stopped sans superviseur.
Les reçus d’arrêts utilisent cancel_stopped dans la tâche racine existante.
La panne SQL réelle et la réouverture vérifient cette persistance ; les
confirmations d’arrêts restent de fixture, sans PID/SIG réel.

Le coût local O(m + f + p + n log n) et O(n²) octets cumulés de sauvegarde
sont commentés, sans mesure de performance. Aucun nouveau magasin, outil
produit, scénario autonome ou ID n’est créé par ce reçu.
Après GO du principal, T007/T040/T026/T027/T028/T046 sont cochées,
progression **41/50 (82 %)**. Le gate initial de 26 items reste inchangé.
US5 et les finitions restent ouverts.

Les références exactes, limites et captures sont conservées dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

## Clôture documentaire US5 — admission et entretien réutilisés

La preuve US5 R1 complète le magasin initial : **15 PASS, 0 FAIL,
0 ignored, RC0**. L’admission existante lit le rejeu avant la pause,
normalise le type inconnu, puis appelle la liste fermée du magasin.
L’entretien existant appelle la purge des reçus, sans nouveau balayage
planifié, module, outil ni dépendance. Les tests utilisent ces chemins
réels avec des données privées ; ils ne prouvent pas l’activation de
production.

T029/T030/T031 sont cochées après GO : **44/50 (88 %)**.
Le gate initial de 26 items reste inchangé. Les historiques restent
conservés. T033/T035 sont autorisées pour les seules finitions prévues.

Preuves, références exactes et limites :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

## Finitions documentaires T033/T035 — 46/50

Le guide, la skill et sa référence existants sont mis à jour. Le seul nouveau
fichier documentaire est l’ADR152 prévu par T035, avec D1–D22 et statut Accepté.
Les comportements décrits réutilisent la garde unique, la reprise de flotte,
la chaîne de continuations, la classification et les magasins existants.
Les deux gardes149 réutilisent `restart_identity` et le refus nommé existant ;
aucun mécanisme alternatif ni nouvel outil.

T033/T035 sont cochées après relecture : **46/50 (92 %)**.
Le gate initial reste inchangé. T032/T034/T036/T037 restent ouvertes.
Les défauts de production R1/R2 sont distingués des adaptations de contrat ;
aucun waiver ni échec hérité inventé. Les sources finales relues et les
résultats historiques ne valent pas verdict final R3 ou activation.

Preuves, périmètre exact, six réponses XX et limites :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

## Prototypes et réutilisation après la revue terminale

Relecture du correctif autorisé, sans nouvel audit de résultat. Même
contrat29FR/9SC/6US et mêmes50 tâches, compteur46/50 inchangé.

Prototypes actuels dans :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs

- Ligne65 : `pub(super) fn may_resume(st: &DaemonState, task: &Task, execution: &str, instance: &str) -> Result<bool, String>`.
- Ligne1285 : `pub(super) fn rejected(st: &DaemonState, conn: &str, mission: &str, reason: &str) -> Result<bool, String>`.
- Ligne1525 : seuls neuf refus métier nommés sont convertis en échec par
  `prepare_restart` ; les autres erreurs remontent.

L’unique appelant de `rejected` est relu ici :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:18654.
Sur `Ok(true)` et `Err`, il revient avant les mutations générales.
Connected réutilise `schedule_execution_recovery` après identité Fleet,
avec les gardes d’instance/commande/connexion primaire et état libre :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:3096.

Les deux chemins d’échéance utilisent `store.get_request`, déjà présent :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1623
et
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1808.
Aucune demande absente/illisible ne crée une date supposée. Le journal de
pause, les magasins, gardes et CAS existants restent réutilisés.

Aucun helper, module, outil, dépendance, tâche ou scénario ajouté.
R5 en cours : lecture du code seulement, pas nouveau GREEN revendiqué.
Les gates finaux et le rafraîchissement de l’inventaire restent ouverts.

## Clôture ciblée T032/T034 et traitement d'une demande absente

Le gate initial conserve son contrat29FR/9SC/6US et50 tâches. T032/T034 sont
cochées après R5 et relecture ciblée :48/50. Quatre oracles149 sont réellement
adaptés ; le cinquième témoin ligne866 reste byte-identique. Aucun contrôle
F2, E2/O4, révocation, source, filiation ou CAS n'est affaibli.

Réutilisation actuelle du correctif None, deux branches :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1623
et
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1820.
get_request existant reste l'unique origine durable d'échéance. Sur Ok(None)
après admission ou working sans échéance, close_lost_execution ligne1445
contrôle cible et CAS avant la sauvegarde failed/request_unavailable. Sur
SQL Err ou clôture refusée, le traitement est reporté sans mutation de tâche
dans ces branches. Le chemin avant admission est conservé. Aucun helper,
outil, journal autonome, dépendance, ID ou scénario nouveau.

FR-029 est explicitement limitée à commande/instance/lien natifs lisibles.
Les refus de cible étrangère, CAS ou stockage peuvent bloquer le démarrage.
La boucle de clôture métier vient après les arrêts confirmés ; son continue
ne provoque aucun arrêt. Une clôture CAS partielle reste possible.

R5 GREEN295/0/1 et suite R1 2103/8/72 restent antérieurs à ce patch. R6
48/3/0 ne valide pas encore celui-ci ; E2E non exécutés, diagnostic en cours.
T036/T037, Converge, audit et livraison restent ouverts. L'inventaire
préparatoire doit être rafraîchi après gel final ; aucun scoring n'est lancé.

Compte rendu requis et preuves :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/tests-152.md.

## Ajustement R3 avant R8

N1464–1469 réutilise abandon_pause_interruption pour request_unavailable,
après les gardes cible/CAS existantes. Une condition, aucun helper ou continuation.
Deux fixtures réutilisent create_request avec vrais IDs/délai, oracles inchangés.
Le comptage corrigé est cinq adaptations149 (O4 incluse), plus le témoin
reprise_d_execution_ignore… byte-identique ; sa garde reste stricte.
R7 a deux FAIL de données sans suivi ; R8/098 encore attendus,48/50 inchangé.
Preuves et six réponsesXX : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.
