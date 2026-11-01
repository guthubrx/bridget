# Audit de réutilisation de l'existant — session222
## Decision
Statut: PASS
Date: 2026-11-01
Feature dir: /home/moi/10.Scripts/64.bridget/.worktrees/222-roles-profils/specs/222-roles-profils
Conclusion courte: Le plan étend le domaine des profils et les frontières natives.
Aucune nouvelle autorité d'exécution, file de missions ou dépendance. Les nouvelles
données métier n'existent pas dans les étiquettes actuelles. La contre-revue de
conception reste une vérification indépendante distincte de ce gate de réutilisation.

## Synthese
| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 13 |
| Items audites | 13 |
| Reutilisations deja prevues | 9 |
| Existants potentiellement pertinents | 4 |
| Duplications evidentes non arbitrees | 0 |
| Regles/memoires applicables | 7 |
| Specs existantes applicables | 7 |

## Reutilisations correctement identifiees
| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Identités et profils actuels | AgentProfileStore | crates/bridget-daemon/src/agent_profile.rs:169 | Étendre le même domaine et conserver les anciennes lignes |
| Sessions persistantes | FleetSupervisor | crates/bridget-daemon/src/fleet.rs:533 | Persistance, génération et rétablissement conservés |
| Démarrage et arrêt | lifecycle/ManagedSupervisor | crates/bridget-daemon/src/lifecycle.rs:175 ; crates/bridget-daemon/src/daemon.rs:22494 | Seuls chemins d'effet natif |
| Budget facultatif | AutonomyBudgetPolicy | crates/bridget-daemon/src/fleet.rs:78 | Absence par défaut, pas de nouveau quota implicite |
| État des missions et file | ExecutionStore | crates/bridget-daemon/src/execution_store.rs:425 | Les métadonnées222 ne copient pas les états d'exécution |
| Remises/idempotence | send_deliveries/NativeDelegation | crates/bridget-daemon/src/daemon/native_delegation.rs:5354 | Même canon, mêmes références, pas de reroutage inconnu |
| Approbation humaine | MobileAdministration/ControlState/user_input | crates/bridget-daemon/src/daemon/mobile_admin.rs:12 ; crates/bridget-daemon/src/user_input.rs | Recontrôler l'acteur, pas de champ humain déclaratif |
| Catalogue UI | Android/HTTP existants | android/app/src/main/java/app/cartae/bridget/MainActivity.kt:157 ; crates/bridget-daemon/src/mobile/http.rs:663 | Ajouter le catalogue aux mêmes clients et scopes |
| Sources d'expérience | journaux/artefacts | crates/bridget-daemon/src/artifact_service.rs:281 ; crates/bridget-daemon/src/daemon/mobile_operations.rs:54 | Référencer les sources, pas de copie générale d'historique |

## Existant potentiellement pertinent non mentionne
| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Rôles multi-valeurs et désignations | agent_profile_labels ; domain | crates/bridget-daemon/src/agent_profile.rs:196 ; crates/bridget-daemon/src/mcp.rs:1629 | Créer métadonnées de rôle avec portée et autorité, conserver les labels généraux |
| Profils réutilisables/versionnés | agent_profiles ; instructions_revision | crates/bridget-daemon/src/agent_profile.rs:191 | Étendre le domaine ; l'actuel profil est lié à une identité et ne conserve pas toutes ses versions |
| Mémoire corrigible et rappel | artefacts versionnés + ledger | crates/bridget-daemon/src/artifact_service.rs:188 | Réutiliser les références ; index métier nécessaire pour projet/type/oubli et rappel |
| Reçus de demande de rôle | work_submissions ; auto_admission_receipts | crates/bridget-daemon/src/execution_store.rs:383 | Petite association d'attribution/canon ; état des tâches reste natif |

## Duplications evidentes
Aucune duplication non arbitrée. Une nouvelle file, une copie des états natifs,
un nouveau mécanisme de budget ou un scheduler sont explicitement exclus du plan.

## Memoires et regles applicables
| Source | Regle | Impact sur le plan |
|---|---|---|
| AGENTS.md | Développement serveur et worktree | Toutes modifications fonctionnelles dans le worktree Cartae222 |
| AGENTS.md | Autorité produit native | Guides décrivent les parcours Bridget |
| AGENTS.md | Livraison complète | Git, activation, retour et nettoyage inclus |
| AGENTS.md | Cargo géré | build.py obligatoire même pour test filtré |
| Constitution XIX/XX | Réutilisation et responsabilité | Aucun wrapper passif ni dépendance nouvelle |
| Constitution XXI.3 | Coordination limitée | Une contre-revue pour un livrable précis, pas d'équipe permanente |
| Entretien utilisateur | Budget désactivé/mémoire interprojets | Pas de plafond implicite ni silo de projet automatique |

## Specs livrees applicables
| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 078-profils-agents | Profils d'identités et consignes | Réutiliser le domaine existant |
| 110-reprise-nom | Nom humain distinct d'identité | Rôle et libellé ne remplacent pas l'identité |
| 145-bridget-thread-viewer | Projection de conversations | Ne pas reconstruire les fils par nom |
| 177-bridget-mobile | Appairage et scopes humains | Catalogue via canal authentifié existant |
| 180-file-mobile | File et reçus natifs | Réutiliser les commandes, pas une file222 |
| 220-catalogue-observe | Déclaré/observé/accès distincts | Prérequis non vérifiés refusés honnêtement |
| 221-capacites-completes | Conservation/admission/continuation | Aucune clôture221 ni rejeu des tâches historiques |

## Journal de recherche
| Requete | Portee | Resultat |
|---|---|---|
| rg Specialist/specialist/role_claim/definition_revision/memory_recall/MemoryEntry | crates, scripts, Android | Aucun domaine équivalent |
| rg AgentProfileUpdate/profile_for_agent_id | crates | Domaine lié aux identités, consignes versionnées sans catalogue réutilisable |
| rg request_spawn/persistent_named/request_relaunch | fleet.rs, daemon.rs | Cycle persistant déjà présent |
| rg AutonomyBudgetPolicy/evaluate_autonomy_budget | fleet.rs | Option absente par défaut |
| rg WorkSubmission/admit_message_submission | execution_store, core | Autorité durable à réutiliser |
| rg MobileAdministrationRequest/ControlState | protocole, daemon/mobile_admin | Contrôle humain existant |
| rg profile/memory/identity | specs | Sept specs applicables relues au niveau des contrats |
| cat Cargo.toml | workspace | serde/SQLite/hash/UUID existants, aucun nouveau package |

## Arbitrages
| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Domaine profils222 | réutiliser et étendre | Même AgentProfileStore, séparation en sous-module pour responsabilité et tests ; pas de service profil concurrent | 2026-11-01 |
| Rôles | créer métadonnées liées aux identités | Les étiquettes n'ont ni portée ni désignation ; les détourner en pseudo-autorisation serait opaque | 2026-11-01 |
| Versions et mémoire | créer métadonnées métier ciblées, sources réutilisées | Le catalogue réutilisable, l'oubli et la préférence interprojets n'existent pas dans le journal ou les profils d'identités | 2026-11-01 |
| Reçus de rôle | créer association/canon uniquement | Une attribution doit précéder l'effet natif ; ne pas ajouter un deuxième état de mission | 2026-11-01 |
| Interface | étendre Android et MCP/CLI | Clients livrés existants ; interfaces futures221 hors de cette transaction | 2026-11-01 |

## Gate avant tasks
- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

### Raccord transactionnel de création des identités
REUTILISER agent_profile.rs:395 ensure_agent_ids. Son BEGIN IMMEDIATE ne pouvait pas être imbriqué dans le savepoint de commande222 (test réel : activation renvoyait storage_unavailable). Extraire sa boucle métier dans ensure_profile_identities ; le chemin historique conserve Immediate, le caller222 conserve son savepoint. Aucun second créateur de profils ni SQL de création copié.

### Sources de mémoire
REUTILISER attach::lineage_journal_page/remote_page et native_lineage::read_task_journal pour la lecture ciblée. CREER specialist_memory_sources pour le dernier constat de lecture (date, curseur, lacune, code fermé) : aucun résultat, phase ou PID de tâche n’est recopié. agent_profile_applications concerne les consignes appliquées, provider_catalogue les modèles ; aucune ne porte ce constat de journal. La collecte conserve ses tombstones et ne lance aucun modèle.


### Raccord composition à l'implémentation
REUTILISER : work_submissions, executions, tracked_requests (store/ledger_requests.rs:272) restent seules autorités de mission et de réponse. CREER : specialist_compositions ne contient que canon et plan immuables, pour figer version, clés de membres et dépendances avant une attribution partielle. Aucun état d'exécution ou résultat copié ; reprise explicite de la même enveloppe, pas de second ordonnanceur ni relance d'un membre inconnu. Chaque dépendance exige une réponse native answered ; ce n'est pas une preuve de réussite métier.


### Client Android et retour humain
REUTILISER le client HTTP/VM et les scopes existants. CREER SpecialistsModel.kt
pour les constructeurs purs consommés par les dialogues, la pagination et les
tests JVM ; pas de store ni client réseau nouveau. Le retour humain222 réutilise
tracked_requests et record_message_in_transaction, car le suivi en mémoire
pending_replies nécessite une vraie socket d'agent. Aucune socket humaine fictive,
aucun champ libre de profil ne donne cette exception.


### Premier lancement temporaire depuis une définition
REUTILISER le port AutoDelegateV2 complet et son schéma MCP. Le raccord
request_temporary fige le prompt depuis le profil puis appelle ce port hors
mutex, sous les mêmes preuves et contraintes. CREER specialist_temporary_requests
pour réserver le premier emploi de la version avant admission asynchrone ;
aucune phase, résultat ou PID copié. Une autre clé ne réemploie pas sans décision
humaine cette définition déjà consommée. Le Task connu est ensuite lié au profil
par la preuve parent/enfant existante. Un rattachement tardif en échec ne relance
pas le travail déjà admis.


### Historique et progression du catalogue
REUTILISER specialist_memory_revisions pour memory_versions : lecture paginée
des révisions déjà stockées, aucun magasin historique nouveau. REUTILISER
specialist_requests/executions pour les16demandes récentes du catalogue ; elles
ne portent aucun corps métier ni copie d'état. Le bouton conversation appelle
MobileViewModel.select et la navigation déjà existants ; aucun contrôle d'arrêt
ou lecteur de mission parallèle.


Plancher de retour222 : REUTILISER storage_compatibility.rs, aucune nouvelle table ni autorité. Le fait nouveau prepared+promotion contrôlée exige specialists.requests.v1 : ancien lecteur refusé même s'il accepte le JSON. Deux images compatibles devront être qualifiées ; l'ancienne image221 ne sera pas annoncée comme retour222.


Contrôle de lecture par appareil : REUTILISER SpecialistPrincipal. permits_record applique la même restriction de projet aux lectures humaines et à la collecte ciblée ; pas de second système de droits.
