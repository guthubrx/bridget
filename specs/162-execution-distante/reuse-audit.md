# Audit de reutilisation de l'existant — 162-execution-distante

## Decision

Statut: PASS
Date: 2026-10-13
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante

Conclusion courte: Le plan révisé R2 réemploie les autorités et les primitives existantes. Les vingt créations de types, messages ou frontières sont justifiées par le passage entre machines ; elles ne constituent ni un second ordonnanceur ni une seconde base métier. Les résultats finaux utilisent un blob immutable distinct des métadonnées du reçu worker, et le contact réemploie Heartbeat. Ce PASS concerne le réemploi avant Tasks, pas la validité globale du code futur ni l’activation d’une route réelle.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 47 |
| Items audites | 47 |
| Reutilisations deja prevues | 27 |
| Existants potentiellement pertinents | 4, tous intégrés ou explicitement arbitrés |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 8 |
| Specs existantes applicables | 9 |
| Créations justifiées | 20 |
| Arbitrages de réemploi encore requis | 0 |

Comptage par responsabilité, pas par fichier : les huit types du modèle de données, les nouvelles commandes/capacités/enveloppes et chaque surface du plan ont une ligne. Un type neuf peut réemployer un stockage et un transport existants. Aucun package ni dépendance externe nouveau n’est proposé.

## Reutilisations correctement identifiees

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/reuse-audit.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/reuse-audit.md#extrait-2).

## Existant potentiellement pertinent non mentionne

Les quatre voisins suivants ont été précisés pendant la préparation. Le plan gelé les prend désormais en compte. Aucune décision utilisateur supplémentaire n’est nécessaire pour le réemploi.

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| I30 Contact worker | Heartbeat | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-transport/src/protocol.rs:3646 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/daemon.rs:22181 | REUTILISER : extension à connexion worker authentifiée. Pas nouveau ping. Décision incorporée. |
| I32 Version de lecture Lineage | HumanLineageRequest.version | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-transport/src/lineage_protocol.rs:5 | REUTILISER ce type et son réexport ; pas contrat parallèle dans protocol.rs. Décision incorporée. |
| I15 Préparation du lancement | launch_preflight, prepare_spawn_parts et build_environment | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/lifecycle.rs:102 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/lifecycle.rs:362 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/lifecycle.rs:413 | REUTILISER ; extraction pub(crate) seulement si nécessaire. submit_spawn et handle_managed_command ne sont pas réutilisables inchangés côté worker, car ils réservent/libèrent au mauvais niveau. |
| I27/I28 Résultat final | capture_reply, save_projected, RetainedResultAuthority et fsutil | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/daemon/native_delegation.rs:2677 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/daemon/native_delegation.rs:2813 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/delegation_lineage.rs:460 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-transport/src/fsutil.rs:49 | REUTILISER les primitives du reçu technique et la transaction maître. Créer capture_remote_result pour l’entrée authentifiée ; RetainedResultAuthority ne vaut qu’après capture vers le parent. ACK métier nouveau justifié ; pas d’outbox générique. Décision incorporée. |

Le manque de durabilité sortante est concret : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/wrapper.rs:5531 ferme son writer après erreur sans conserver le message sortant ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-transport/src/journal.rs:715 écrit puis flush, sans fsync dans ce chemin. Le détail des ACK obsolètes vient aussi de la recherche code161 transmise par ROOT ; la branche anciennes rondes de capture_reply a été relue. Cela justifie le complément prévu, sans affirmer qu’il est déjà implémenté.

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucun doublon 1:1 non arbitré | Aucun | Recherches de noms et responsabilités ci-dessous | Aucune. Les vingt créations justifiées sont identifiées individuellement par CREER dans les lignes I01–I47. |

Le worker ne doit pas instancier une seconde Fleet. Le reçu sortant final ne doit pas devenir une deuxième base de résultats. L’ACK de transport, l’ACK d’injection et la confirmation du résultat sont trois preuves différentes. Les réutiliser sans distinction serait une erreur de contrat, pas une simplification.

## Memoires et regles applicables

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/reuse-audit.md#extrait-3).

CLAUDE.md est absent de ce worktree. La mémoire /Users/moi/.Codex/projects/64.bridget/memory/MEMORY.md est absente au chemin contrôlé ; aucune mémoire inventée. Les instructions AGENTS reçues dans le contexte restent applicables. Aucun conflit de réemploi exigeant un arbitrage nouveau n’a été trouvé.

## Specs livrees applicables

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/reuse-audit.md#extrait-4).

Les plans proches sont vérifiés sur leurs passages utiles ; ce tableau n’atteste pas une nouvelle exécution de leurs recettes. La livraison161 est un contexte de transport, documenté par AGENTS et le plan162, pas une preuve d’un worker déjà présent.

## Journal de recherche

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/reuse-audit.md#extrait-5).

Les requêtes sont retranscrites sous forme de motifs lisibles. Les chemins inexistants essayés pendant la recherche, dont les anciens noms delegation_store.rs et project_root_policy.rs, n’ont pas servi de preuve positive. Le script /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/.specify/scripts/bash/check-prerequisites.sh est absent au contrôle direct ; le FEATURE_DIR a été fourni explicitement par ROOT. Aucun script Git de substitution exécuté.

## Arbitrages

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/reuse-audit.md#extrait-6).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/reuse-audit.md#extrait-7).

Vertus LLM & Responsabilité Future : le plan augmente nécessairement la logique de frontière, mais limite la charge de maintenance en gardant une autorité maître et une seule chaîne native. Les nouveaux types matérialisent des preuves différentes, pas des wrappers de confort. Les recherches externes sur Claude sont portées par le rapport placement162 et la recherche162 ; cet audit ne les a pas relancées et ne certifie aucune version installée.

## Révision R2 ciblée

Le verdict est réétabli sur le plan R2 lu. La contre-revue R1 n’est pas assimilée à une approbation R2. Ses corrections sont vérifiées ici sous l’angle du réemploi :

| Point | Décision vérifiée | Preuve de l’existant et limite |
|---|---|---|
| O1 Store et écrivains | Store technique distinct ; worker seul écrivain des métadonnées, wrapper seul producteur du blob final immutable. Slot réservé avant dispatch. Enveloppe sérialisée de 2 Mio maximum ; plafond16 Mio comprenant réservations, anciens/nouveaux fichiers atomiques, métadonnées et temporaires. Moins de huit slots maximaux. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/receipt_store.rs:340 et :361 : verrou exclusif et format Seen/Acked à deux lignes ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-transport/src/fsutil.rs:49 : primitive atomique réemployée. Le reçu entrant n’est pas transformé. |
| O2 Résultat et fin de retransmission | capture_remote_result contrôle connexion/époque, placement et identité complète. Mutation métier commune à capture_reply. Dispositions accepted, discarded_terminal, rejected_conflict, deferred. Confirmation après transaction Task/projection. Deferred conserve le slot ; les issues finales arrêtent la réémission. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/daemon/native_delegation.rs:2677 exige déjà result_available pour RetainedResultAuthority ; :2813 dépend aujourd’hui de la connexion wrapper. Son extension ne suffit donc pas sans nouvelle garde d’entrée. Blob collectable après disposition finale durable, producteur terminal et commande ou slot irrévocablement fermé. Reçu compact, empreinte et disposition conservés sept jours ; tous les octets conservés restent comptés. |
| O3 Contrat complet | parent_proof préserve les parents distants. journal_page dérive sa racine de l’instance/génération attestées par le reçu ; lecteur Attach réemployé sur le worker. Le suivi vivant conserve LiveJournalFragment/AttachView, sans nouveau follow/stop. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/native_permissions.rs:598 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/attach.rs:34 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/daemon.rs:21507. Le lecteur local seul n’est pas un transport de pages distantes. |
| O4 Reconnexion | Même worker, credential et révision : remplacement atomique de connexion, nouvelle époque, fermeture ancienne. Anciennes trames refusées. Aucun permis ou droit de relancer restauré par le handshake. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/plan.md:24. Extension de l’index de connexions et du transport ; aucun second superviseur métier. |
| O5 Supervision | Unité utilisateur worker dérivée095, SIGTERM, KillMode=process et SendSIGKILL=no ; réconciliation avant nouveau lancement. Unknown demeure retenu jusqu’à preuve fiable ; pas nouvel outil destructif. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/scripts/federate-ssh.sh:439 fournit le patron utilisateur, Restart, UMask et SendSIGKILL. KillMode=process est l’adaptation explicite pour préserver les wrappers. Préflight systemd est fourni par ROOT, sans nouvelle connexion SSH de cet auditeur. |
| Compatibilité et quotas | Seul l’ancien registre ignore machines ; MCP et demande native refusent déjà les champs inconnus. Client Lineage Rust à étendre explicitement pour succès v2 et erreurs historiques v1. Quota Fleet global toutes catégories, limites fournisseur157 par agent_type, aucun nouveau quota machine. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/delegation_mcp.rs:118 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-transport/src/protocol.rs:2154 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/lineage_client.rs:38 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/fleet.rs:1011 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/provider_limits.rs:107. |

Recherche complémentaire R2 : rg sur ReceiptEntry/flock, RetainedResultAuthority/capture_reply, LiveJournalFragment/AttachView, allowed/deny_unknown_fields, response.version, systemd/KillMode et quota/by_agent_type dans les fichiers cités, puis lectures ciblées de leurs corps. Aucun résultat futur de test n’est inféré. Les nouveaux noms capture_remote_result et RemoteResultDisposition désignent le contrat proposé ; aucune implémentation livrée équivalente n’a été trouvée.

### Corrections après contre-revue R2

A à D ont été intégrées dans les artefacts gelés, puis lues. Elles ne créent aucune nouvelle autorité métier :

- **A — Stores stricts.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/plan.md:52 fixe Task.remote_placement et DesiredEquipier.remote_placement ; Fleet5 assume le changement de schéma. Le retour arrière :138 interdit de relire un état distant avec l’ancien binaire. Sauvegarde jointe cohérente après quiescence ; sans restauration sans perte, conserver162 et désactiver la route. Les gardes strictes existantes sont conservées, pas assouplies.
- **B — Entrée unique.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/plan.md:58 réserve final_result au worker. Send et SendIdempotent ordinaires refusent les résultats distants courants ou passés par Nack remote_result_requires_worker, sans chute vers prepare_dispatch. Le wrapper ne gère plus d’ACK de résultat distant ; il publie seulement le blob. La mutation métier demeure partagée.
- **C — Réservation à chaque dispatch.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/plan.md:66 définit reserve_result_slot. La nouvelle ligne I47 distingue stockage technique et capacité Fleet. Réservation hors verrou puis revalidation, identité inchangée sur refus, canon d’admission avant exécution distinct de l’empreinte du résultat à venir.
- **D — Même UID.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/plan.md:26 nomme le domaine de confiance partagé. Réemployer les contrôles canoniques pour exclure credential et état worker des mappings/cwd/racines fournisseur ; préserver le seul slot du wrapper d’infrastructure. Les modes full-access ne deviennent pas isolés. Credential en mémoire worker seulement, jamais dans pass_env ; remplacements de connexion tracés sans secret.
- **Contact.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/plan.md:76 retire la prétention de gel existant des données volatiles. Réutiliser la séquence de projection pour invalider les curseurs sur transition ; la date est observée à chaque page. Pas cache de snapshots supplémentaire.
- **Expiration, journal et collecte.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/plan.md:48 réemploie abandon/EOF ; :74 dérive le journal depuis l’identité attestée ; :68 permet la collecte sûre du blob et conserve sept jours les reçus compacts. Une commande ambiguë ou un slot encore actif demeure conservé et peut saturer le budget. Unknown peut réduire aussi la capacité locale puisque Fleet est global.

A/B/C sont aussi appuyés par la lecture ciblée code161 transmise à ROOT, D par placement162. Cet audit reprend ces preuves déclarées et relit le plan et les contrats corrigés ; il ne prétend pas avoir rejoué leur revue complète. La recherche de reserve_result_slot porte sur les sources existantes et ne fournit aucun équivalent implémenté. La contre-revue externe R2 est APPROVE_WITH_CHANGES ; ROOT a confirmé séparément la clôture des corrections. Ce PASS reste limité au réemploi.

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

Passage possible à Tasks après la contre-revue du plan pilotée séparément par ROOT. Les mappings de production, identités provisionnées, permissions fournisseur et recettes réelles restent des gates d’activation. Ils ne rendent pas le réemploi indécis et ne justifient pas de réduire le lot à une coquille.

## Empreintes et limites de preuve

Sources de base annoncées par ROOT : Bridget84cd2430, T3d833387203e113ec2a24f947ba8b8064a1c2542b. Aucun Git exécuté pour les réattester. Plan et artefacts lus après le signal de gel final R2 du rédacteur ; delta après objections A/B/C/D inclus ; captures SHA ci-dessous. Aucun build, test, SSH, accès service, migration, nettoyage ou provisionnement lancé. Aucune preuve de bon fonctionnement de162 n’est déduite de cet audit.

| Artefact lu | SHA256 | Lignes |
|---|---|---:|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/adversarial-review-claude.md | 5f01d9756d145b8c32073517999acbe6a1c5dc195050e6d0898bc94f6c634387 | 52 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/checklists/requirements.md | 97686340cac6d2852f3a4c75dbe7b3efdd054b9775cf40e18add8ed02c0ce3b7 | 22 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/contracts/lineage-v2.md | 25fd15e2bc6a6d09aa4cb716e5e184a5da47aefad3dcc1df8202646931730cd8 | 23 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/contracts/remote-worker.md | 0ae699b06accaa9e1bde9eedf7f2aece4ad9822950feb51c8b69497945445d4d | 75 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/data-model.md | 0ba4cda0870b003242660e9b6fb4b882e6dd082917549e48456f0c2291c80cfe | 47 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/plan.md | 7361a37f9cc47356534d5400626258b99d3d243691e2710bb6913de7855cf7aa | 148 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/quickstart.md | 6ff73ca734ce072c835e96f2d23a965fbf53b590d58bce5a6c673cfe8f2dc23b | 32 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/research.md | dd3281f9c33da5a12ad6a0edcef59458c75a6763877dc9ade3d68d32859499da | 71 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/spec.md | 02ebe79e074000273bf799e77bd86abee36460088331d89549928e526ceb0faa | 138 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/docs/decisions/162-execution-distante-sous-autorite-unique.md | 6cd47a35f35a781122aeefefb72c1e00473b49f3fc5739886535e80a6ed1fc8c | 50 |

Les sources d’implémentation ont été inspectées par recherches et extraits ciblés, pas intégralement. Des sorties longues ont été réduites puis les passages décisifs relus ; aucun pourcentage global de couverture n’est annoncé. Les détails de la publication finale et de l’ACK legacy combinent lecture directe et constat code161 transmis par ROOT. Le seul fichier écrit par cet auditeur est /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/reuse-audit.md.

## Annexe de raccordement — lecture du code du 13 octobre 2026

Cette annexe documente des décisions déjà autorisées et leur présence dans le code en cours. Elle ne remplace ni les empreintes du gel R2 ni le PASS 47/47 historique. Aucune nouvelle revue globale, compilation, exécution de test ou recette de production n’a été menée pour cette annexe. Les numéros de ligne décrivent cette lecture ; les sources restent modifiables par leurs propriétaires.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/reuse-audit.md#extrait-8).

Ces extensions précisent les items déjà audités, notamment I40 ; elles n’ajoutent pas une quarante-huitième fonctionnalité ni un nouveau verdict de validation. Les résultats de tests restent exclusivement ceux consignés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/validation/tests162.md.

### Limites de validation et corrections en cours transmises par ROOT

La compilation ciblée daemon162-runtime-r1 a terminé avec RC101 après 32,24 secondes : chrono absent et deux comparaisons agent_id incorrectes. Aucun test de ce groupe n’a été exécuté. Les logs, le manifeste et le reçu JSON restent dans /Users/moi/.cache/bridget-delivery162-20261013 sous ce préfixe. Ces faits sont transmis par ROOT ; cette annexe n’en revendique pas une exécution ni une validation indépendante. Les propriétaires préparent les corrections.

Deux raccordements restent à vérifier après correction, dans T023/T028/T029 existantes : récupérer un blob arrivé avant ExecutionStateChanged perdu par le CAS d’exécution existant, puis persister résultat et reçu sans rouvrir un terminal ni contourner quota ou annulation ; rétablir le curseur de reprise Lineage après live_feed_overrun, avec contrôle instance/placement/commande avant émission. Le wrapper rejoue déjà depuis from_seq, selon la lecture de ROOT. Aucun résultat de test futur ni tâche supplémentaire n’est déduit.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/reuse-audit.md#extrait-9).

### Complément de réemploi E2E et observation — checkpoint R5

ROOT autorise le sous-module /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/tests/fixtures/remote_worker_162.rs pour trois E2E des tâches T014/T028/T031. Le module est déjà déclaré dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/tests/native_delegation_e2e.rs:6 ; son fichier est encore en préparation lors de cette lecture. Réemployer les fixtures existantes OwnedChild (:33), Fixture (:126) et le fournisseur fermé. Un relais Unix de test pilote le contrôle maître ; le daemon et le wrapper restent réels. Aucune injection de test dans la production, nouvelle API ou tâche hors des 42 prévues. Cette justification documentaire ne certifie ni la compilation du sous-module ni ses trois recettes ; la cible E2E est expressément exclue de R5.

Le raccordement de l’observation au snapshot conserve WorkerObservation et ses gardes existantes. Le reçu technique reste borné à 64 Kio par MAX_METADATA_BYTES dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/remote_worker.rs. Aucun type public supplémentaire nommé SnapshotObservation n’a été trouvé dans la lecture ciblée ; ne pas interpréter cette désignation de raccordement comme une nouvelle API. Le contrôle transport remote162_observation_recontrole_borne_et_refus_prepare reste en cours à ce checkpoint, sans résultat revendiqué.

Pour T029, le replay après live_feed_overrun réemploie AttachRelay::restart_from_seq, avec lecture froide à destination autorisée par l’abonnement et garde de snapshot à chaque émission. WorkerJournalPage conserve la responsabilité de lecture froide explicite. La reprise conserve les gaps irrécupérables au lieu d’inventer les lignes manquantes. Le test affecté passe en R5 ; les essais E2E restent distincts.

### Delta T044 — primitives retenues et réconciliation bornée

Lecture ciblée des sources de R4, sans test exécuté par le rédacteur :

| Surface | Réemploi et justification |
|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/managed_process.rs:228 et :327 | wait_ready_or_cancel_retained et persist_marker_retained conservent erreur et Child. Le lecteur de statut est partagé et les signatures historiques restent disponibles. L’arrêt signalé n’est pas assimilé à une mort prouvée ; try_wait reste nécessaire. Aucun nouveau canal IPC. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/daemon.rs:1100 et :3488 | Curseur remote_start_reconcile_last volatil pour cadence et rotation ; budget CANCEL_NOTIFICATION_BUDGET existant. Ni preuve métier, ni store, ni fil d’exécution supplémentaire. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/daemon/native_delegation.rs:5562 | Un seul candidat coûteux par cadence. Le curseur avance avant sa validation. Les observations et push_control_message_until sont réemployés ; les E/S restent hors mutex. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/remote_worker.rs:2349 et :2358 | finish_command réemploie finish_result_slots et close_result_slot après état terminal attesté. Réservation vide libérée ; Unknown reste conservé et Deferred ne devient pas collectable par simple fermeture. |

R4 valide les primitives et la collecte avec 73 lib PASS et 4 E2E PASS. Après R4, ROOT a trouvé que reconcile_all répétait l’inventaire des slots pour chaque commande terminale retenue. Le correctif retenu groupe cette lecture par passage, sans nouveau store ni flag ; sa validation reste attendue. Ce delta justifie le réemploi, sans prétendre à un audit final de la future source ni à un gain de performance mesuré.

### Clôture du delta T044 sur R5

La source /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/remote_worker.rs:2378 groupe désormais l’inventaire des commandes et slots. Un BTreeMap temporaire indexe Terminal/Refused ; l’identité entière est comparée avant finish_result_slot. Ces commandes ne repassent plus chacune par reconcile et son inventaire global. Les transitions actives gardent leur preuve fraîche. C’est une réorganisation du parcours existant, sans nouvelle persistance ni option.

SHA worker vérifié : 7b749f35859b85487792afd82af409db672687abf347146e26770fb5b898229e. R5 donne 17 worker PASS et 4 E2E PASS, RC0 ; R4 atteste les primitives retained et la sonde maître inchangées. Les preuves et leurs limites figurent dans le registre de tests. Aucun benchmark ni gain de performance mesuré ; aucun verdict d’audit global anticipé.

### Delta D1 — libération avec conservation du handle

La variante release_retained étend la primitive managed_process existante. Elle conserve RunningManagedChild et le marqueur sur erreur de libération ; l’API historique reste disponible. Le worker garde le handle avant les opérations susceptibles d’échouer. Aucun nouveau store, protocole ou framework. R6 atteste 29 lib PASS et 4 E2E PASS, avec quatre helpers ignorés. Une garde native_status en ReleaseAuthorized est corrigée ensuite ; sa preuve exacte R7 reste attendue. Cette annexe n’anticipe ni clôture D1 ni verdict de revue final.

### Réattestation ciblée D1 close

R7 exact passe (1 PASS, RC0) après correction de native_status en ReleaseAuthorized. La relecture indépendante confirme failed=true avec marqueur sans promotion de phase, et le refus de failed=false. R6 conserve la preuve des primitives retained et des quatre E2E avant cette garde finale. Worker final SHA 3ee35d1a54376f40891c2b8d9f4eb763d1d1a2726076ca936471d584721fb2d6 ; managed_process SHA 1d4937145011edc55ff9875a83f25998dd4a000d7a962b526237d1116d040c83. Aucun nouveau store ni protocole ; D1 clos, sans nouveau verdict d’audit global anticipé.

### Correction G8 — arrêt des descendants Linux, arbitrage de réemploi

Le groupe wrapper ne suffit pas à arrêter les groupes détachés par la sandbox : la recette réelle maintenait deux descendants après Terminal. Selon l’exploration ciblée du correctif, les primitives PR_SET_CHILD_SUBREAPER et pidfd nécessaires ne sont pas déjà disponibles dans le chemin d’arrêt existant. Leur ajout est justifié par une frontière de sécurité prouvée, pas par un nouveau service de supervision.

Réemplois : managed_process et process_birth/lectures /proc ; contexte WorkerResultContext validé ; marqueur de naissance ; namespace markers privé, publication atomique/fsync et budget du store ; gardes de rétention et transitions worker. Un fichier .drained fermé et corrélé par commande porte l’attestation, réserve maximale4Kio temporaire compris. Il n’est pas une seconde base métier ni un nouveau protocole réseau. L’extension distincte évite une lecture comme ManagedMarker.

Choix minimal : subreaper uniquement pour le wrapper distant natif Linux. L’inventaire couvre les children de tous les TID. Pidfd puis relecture naissance/UID/PPID précèdent SIGTERM individuel. Un thread temporaire signale sans récolter pendant stop/Drop afin de libérer les pipes ; après la barrière, waitid(P_PIDFD) récolte les seuls enfants validés et P_ALL/WNOWAIT sonde la fin sans récolte globale. Aucun waitpid(-1), SIGKILL nouveau, cgroup, store SQL, registre global ou commande d’effacement forcé. Les trois branches Terminal (running, abandoned, marker) doivent exiger la preuve dès que RELEASE est possible.

Le plan final et son arbitrage sont autorisés ; l’implémentation et les essais T045/T046 ne sont pas encore validés. Unknown conserve capacité et preuves si le wrapper meurt sans attestation. Le subreaper ne constitue pas un confinement kernel persistant après sa mort. Cette limite est conservée, sans prétendre qu’une procédure opérateur existante saurait toujours la résoudre.
