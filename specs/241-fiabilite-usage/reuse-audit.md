# Audit de reutilisation de l'existant — 241

## Decision

Statut: PASS
Date: 2026-11-07
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/241-fiabilite-usage/specs/241-fiabilite-usage

Le plan étend les composants natifs. Aucun service, table, moteur ou nettoyeur
parallèle n'est proposé. Les détails d'autorité, de qualification d'outils et
la cause JSONL restent à vérifier dans les tâches, sans reconstruction cachée.
Les scripts SpecKit projet sont absents ; audit exécuté directement avec rg,
lecture des contrats, des magasins et des consignes chargées.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 14 |
| Items audites | 14 |
| Reutilisations deja prevues | 14 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 9 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Connexion et reprise avant envoi | DaemonConnection, registered_connection_until | crates/bridget-daemon/src/communication/client.rs:1095,1354 | Réutiliser le budget global et les codes typés ; pas de client bis. |
| Reçu après réponse perdue | recover_admission_reply171, send_then_wait | crates/bridget-daemon/src/delegation_mcp.rs:672 | Canon et request_id existants. |
| Rattachement de session | resolve_binding | crates/bridget-daemon/src/t3code_mcp.rs:61 | Examiner le double handshake sans supprimer l'authentification. |
| Dispatch et entretien des vues | handle_wrapper_message_with_peer, collect_closed_attach_views | crates/bridget-daemon/src/daemon.rs:19800,4047 | Isoler la maintenance des demandes indépendantes si la mesure le justifie. |
| Relais de la machine source | federation_stream_run et installation signée par reçu | scripts/federate-ssh.sh:428 | Le runner installé refait les validations et une connexion SSH pour chaque socket. Mesurer ; ne pas créer de relais concurrent. |
| Autorité de mission | Task et NativePermissionSnapshot | crates/bridget-daemon/src/delegation.rs:1044; crates/bridget-transport/src/native_permissions.rs:23 | Étendre le contrat existant si nécessaire, sans nouvelle table ni credential persistant. |
| Vérification finale et récupération | before_release, check_reattested_parent, start_recovery | crates/bridget-daemon/src/daemon/native_delegation.rs:7550; crates/bridget-daemon/src/daemon/native_permission_change.rs:54 | Propriétaire/révocation/sources restent natifs ; les anciens contrats restent distingués. |
| Répertoires hérités | project_remote_permissions_in_operation, project_policy_directories171 | crates/bridget-daemon/src/native_permissions.rs:274,388 | Distinguer identité locale authentifiée et traduction distante. |
| Résolution de placement | ProjectMapping::for_origin | crates/bridget-daemon/src/registry.rs:460 | Ne pas écrire de registre pour une projection locale. |
| Sources hors Git | validate_parent_source_base et prepare-project.py | crates/bridget-daemon/src/lifecycle.rs:480; scripts/prepare-project.py:521 | Étendre uniquement dans l'autorité attestée ; pas de dépôt fictif. |
| Volumes et traitement progressif | workspace_commit_view, compact, lecteurs de manifeste | crates/bridget-daemon/src/workspace_commit_view.rs:307,1438; workspace_compact.rs:10 | Inventorier avant de changer ; ne pas remplacer le contrôle de ressources par de la mémoire non bornée. |
| État par clé et rétention | DelegationStore, projection de lineage | crates/bridget-daemon/src/delegation.rs:377; delegation_lineage.rs:29 | Les pages et l'historique ne doivent pas conditionner une lecture ciblée. |
| Qualification de candidats | CandidateProbeV2 et WorkerReadView::readiness_at | crates/bridget-transport/src/protocol.rs:3323; crates/bridget-daemon/src/remote_worker_read.rs:1170 | Les nouveaux faits d'outillage doivent rester dans ce protocole négocié. |
| Vérification/livraison/entretien | fixtures de sockets, scripts/build.py, maintenance et pointeurs natifs | specs/240-reponses-tardives/validation.md:81,113 | Réutiliser preuves valides, outils de build et de nettoyage ; aucune copie de target. |

## Existant potentiellement pertinent non mentionne

Aucun après ajout du relais de fédération au périmètre d'observation : il porte
le chemin réel de la socket Mac et doit être mesuré avant d'attribuer JSONL au cœur.

## Duplications evidentes

Aucune création d'un composant ayant déjà un équivalent n'est proposée.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| AGENTS.md projet | Livraison activée et vérifiée | Sept composants et retour, pas seulement compilation. |
| Décision utilisateur du fil | T3 reste ouvert | Aucun redémarrage de l'application ou contournement de l'exclusion. |
| Constitution XIX/XX | Réutilisation et charge cognitive | Pas de framework, couche ou option sans besoin réel. |
| AGENTS.md projet | Build par scripts/build.py | Cache réutilisé, entretien avant/après. |
| Skill my-specify-all | Analyze, Converge, revue, Git/nettoyage | Toutes les tâches sont tentées ; une inconnue n'est pas un succès. |
| Consignes de maintenance | Préserver missions, reçus et usages | Aucun essai sur une tâche métier ; nettoyage sous preuves d'usage. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 168/171 | Admission/reprise identique | Garder la clé et le canon. |
| 177/206 | Instantanés, refus de droits typés | Garder les révocations réelles et distinguer vie du tour. |
| 221 | Préparation, capacités et routage | Étendre les sondes existantes. |
| 229 | Contexte de projet transmis | Ne pas reconstruire un registre parallèle. |
| 232 | Volumes et récupération | Examiner les bornes et lecteurs ensemble. |
| 234 | Inventaires, formats, réserve | Réutiliser les mesures et le retour compatible. |
| 235/237 | Canaux et lectures ciblées | Ne pas réintroduire de scans globaux. |
| 239 | Mandat transversal | Accord existant conservé. |
| 240 | Réponses tardives et causes | Réutiliser les huit preuves tant que le code ne change pas. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| registered_connection, pre_dispatch_failure, recover_admission | client/MCP | Connexion et reprise déjà natives. |
| parent_fact, permission_snapshot, reattest, before_release | autorité et Task | Contrôles existants, dépendance HTTP finale au tour démontrée. |
| permission_mapping_unavailable, additional_directories, for_origin | projection/registry + log worker | Traduction locale du dossier supplémentaire sans mapping. |
| compilation_capacity, CandidateProbeV2, required_tools | transport/worker/registry | Sonde existante ; absence d'exigence d'outillage structurée. |
| MAX_BYTES, MAX_RETAINED, MAX_AUTO_ADMISSION, MAX_ORIGIN | préparation/admission | Bornes existantes à inventorier ; pas de nettoyeur à créer. |
| handle_connection, close_and_join, socat, stream-run | cœur + runner installé | Chemin de handshake et maintenance identifiés ; causalité JSONL ouverte. |

## Arbitrages

REUTILISER les quatorze éléments.
T003 : instrumentation directement dans DaemonConnection::transmit, labels
fermés et durée seulement ; aucun nouveau helper ou compteur durable. Une extension de contrat ou un champ nécessaire
dans une entité existante sera justifié ici avant création. Aucun doublon connu
à arbitrer et aucune nouvelle dépendance. Le plan n'a pas été refactoré par
cette phase : les observations complètent ses tâches de diagnostic prévues.

## Gate avant tasks

- [x] Plan et artefacts lus ; sources et consignes pertinentes consultées.
- [x] Chaque item extrait possède une décision et une preuve.
- [x] Aucun doublon évident ou arbitrage architectural non résolu.
- [x] Les inconnues causales sont des tâches d'observation, pas des correctifs supposés.
- [x] Le périmètre conserve les exclusions et les travaux étrangers.

## Gate d’implémentation — entretien des vues et autorité

2026-11-07 : recherche `collect_closed_attach_views`, `run_maintenance`,
`pending_post_response_controls`, `close_and_join`, `Task`, `permission_snapshot`.
L’entretien horaire ne convient pas à la fermeture des vues. Le fil de rappel
porte des échéances métier et ne doit pas attendre ces rédacteurs. Décision :
réutiliser le collecteur et les contrôles existants dans un unique fil dédié
aux vues, sans nouvelle file ni service, avec référence faible vers l’état.
Le nombre de fils ne dépend pas des appels ; aucun rédacteur n’est abandonné.
Le test avant correctif retient effectivement une réponse d’identité tant que
le rédacteur d’une autre vue n’est pas libéré (150 ms, puis fermeture propre).
Cette preuve établit le couplage ; elle ne prouve pas que tous les délais
historiques JSONL ont cette cause.

Autorité : ajouter un seul champ optionnel à Task et une méthode de lecture
validée, `admitted_parent`, plutôt qu’une preuve, table ou couche nouvelle.
La méthode partage la validation entre démarrage, reprise et lecture durable.
Seule l’admission attestée produit la version 1 ; les fixtures historiques
gardent None. Une valeur inconnue reste refusée sans réécriture.

Gate outillage : recherche des runners dans lifecycle, workspace_git,
provider_program_compat, host_resources et remote_worker_read. Le runner Git
impose ses arguments Git et ne convient pas aux autres outils. Réutiliser sa
récolte SIGTERM du Child, sans copier le mécanisme. Ajouter la qualification
fermée dans remote_worker_read et la réexporter pour le candidat local, hors
mutex du cœur. Les faits rejoignent CandidateProbeV2 et ses diagnostics, sans
magasin ni cache permanent. Types fermés WorkloadTool, WorkloadToolState et
WorkloadToolObservation nécessaires au contrat négocié ; pas de chaîne libre.
La détection de commandes reste dans le routage natif. Aucun script de dépôt
n’est exécuté pour découvrir un besoin. Contrat détaillé dans workload-tools.md.

Gate sources supplémentaires : les gardes prefixées sont réparties entre
lifecycle, origin_proof et native_delegation ; aucune ne centralise les racines
déjà présentes dans le fait fournisseur. Réutiliser validate_parent_source_base
et ajouter source_root(parent,cwd), fonction pure de sélection dans lifecycle,
sans I/O sur l’hôte étranger. Seuls le cwd du parent et les racines explicites
additional_directories/writableRoots déjà attestées sont candidats. Les
contrôles physiques restent sur l’origine et les protections du worker restent
appliquées. Le helper Python réutilise select_mapping/publish_mapping pour le
cas local hors Git ; aucun miroir fictif ni modification d’un mapping existant.
Un project_id explicitement incompatible reste refusé ; il n’est pas réécrit.

Gate profil de commandes : recherche permits_command_execution, child_policy,
Bash et outils dans native_permissions/remote_worker_read. Aucun prédicat commun
n'existait ; étendre native_permissions avec le seul prédicat pur qui lit le
profil déjà projeté. Le routage local et la sonde distante réutilisent ce fait,
sans seconde projection ni ajout de Bash. Aucun service ou store ajouté.

Revue R1 : réutiliser les règles de chevauchement du registre pour empêcher la
publication automatique d'un mapping sur la racine, le home ou les stores de
contrôle locaux. Le helper de publication fait le contrôle avant toute écriture.
Cela ne retire pas au parent son accès aux fichiers autorisé ; seul un mapping
qui recouvrirait le plan de contrôle est refusé, comme par le worker existant.

Gate capacité physique : fstatvfs existait dans la vue de commit seulement.
Réutiliser sa lecture dans workspace_git::available_space, appelée par la vue
et le worktree. Ne pas copier la lecture ni inventer une réserve fixe. La vue
conserve son estimation par manifeste ; le worktree réutilise les tailles de la sélection compacte ou un ls-tree borné
pour le checkout complet, puis garde les erreurs d'allocation pendant Git.
Aucun quota par nom ni réserve fixe.

## Delta R3 — mots des commandes

Recherche `shell_words|shlex|split_shell` : aucun analyseur réutilisable dans les
crates du projet. Le paquet shlex 2.0.1 existe déjà dans Cargo.lock via cc.
Décision PASS : utiliser exactement ce paquet pour les mots cités, sans exécuter
ni développer le shell. Le séparateur local ne coupe que les opérateurs hors
quotes ; il ne cherche pas à interpréter les programmes. Aucun nouveau paquet
ni version résolue, une arête directe ajoutée. Les exigences restent indicatives
et fermées ; les programmes arbitraires et constructions inconnues restent inconnus.
