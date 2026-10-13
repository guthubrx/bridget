# Données SPEC-162

## MachineProvision

Section machines du registre privé : machine_id, enabled=false par défaut, worker_id, provisioning_revision, credential_fingerprint, allowed_agent_types, mappings. Bornes : 16 machines et 64 mappings par machine. Le worker conserve son credential privé et ses définitions fournisseur. Aucun secret dans Debug, projection ou reçu. Le registre fournisseur épinglé ne vaut pas autorité machine.

## ProjectMapping

project_id, source_host, source_root, remote_root, expected_git_identity, mapping_revision. Racines canoniques approuvées, chemin relatif calculé par composants. Conserver le commit de base pour les tâches de code. Contrôler cette base au démarrage, puis autoriser les changements de la branche de travail. Aucun reset forcé, aucune synchronisation automatique des fichiers non committés. Réemployer l’isolation 154 sur la machine choisie.

## RemotePlacement

machine_id, worker_id, provisioning_revision, project_id, mapping_revision, origin_host, origin_cwd, source_root_identity, relative_cwd, execution_cwd, base_commit, definition_digest, permission_digest. Origine et exécution restent distinctes. Le parent peut déjà être distant : sa preuve est relue sur son hôte attesté. Placement figé dans canon, réservation, tâche et exécution. Stockage : Task.remote_placement Option default et DesiredEquipier.remote_placement Option default ; Fleet schéma 5, lecture 1–5 et écriture 5. La requête native persistée conserve ses champs machine_id/project_id. Anciennes lignes locales sous 162 seulement ; aucun downgrade sur ces stores inchangés. Aucun champ ajouté par défaut à la projection Lineage v1.

## WorkerConnection

worker_id authentifié, machine_id, version, build, connection_epoch, last_contact_observed_at, freshness. La reconnexion avec les mêmes identité, credential et révision autorisés remplace atomiquement l’ancienne connexion ; credential divergent refusé. Une seule connexion fait autorité. Heartbeat est unitaire et reçoit son époque du contexte de connexion. Toute trame visant un autre placement ou une ancienne connexion est refusée. Une nouvelle époque ne restaure aucun permis.

last_contact est volatile et affiché comme observation à la lecture de chaque page. Notifier seulement les transitions de fraîcheur. Les transitions de contact_state incrémentent seq et invalident les curseurs. last_contact_at reste propre à la lecture de chaque page, sans nouvelle base, gel global ou âge vivant inventé.

## WorkerCommandReceipt et blob résultat

Store technique distinct de ReceiptStore Seen/Acked. Réemployer les gardes fsutil, verrou, enrollment et quarantaine. Le worker écrit seul : command_id, canon, placement, phases, références de préparation, permis, marqueur, observation et disposition confirmée. Phases : reserved, marked, release_authorized, released, terminal/refused/unknown.

Le wrapper écrit seul un blob immutable dans un slot privé réservé pour tâche, mission, ronde, instance, génération, execution_id et génération d’exécution. Il contient l’enveloppe finale, message_id stable, corps et empreinte. Publication atomique avec fsync avant réseau. Le worker ne modifie pas ce blob et le wrapper ne modifie pas le reçu. Blob existant égal : lecture ; divergent : quarantaine, sans remplacement. Aucun nouvel IPC ni store métier.

reserve_result_slot idempotent réserve la capacité avant RELEASE, chaque ronde et chaque reprise quota reconstruite. Clé : tâche/mission/ronde/instance/génération/exécution/génération, digest du canon d’admission ; le digest du résultat est inconnu jusqu’à publication. Refus laisse la même identité non dispatchée. Clôture d’un slot caduc exige preuve durable de non-dispatch ou annulation, sans arrêt du wrapper réutilisé. Corps maximum 262 144 octets ; enveloppe sérialisée maximum 2 Mio. Le budget de 16 Mio compte réservations, blobs, métadonnées anciennes/nouvelles, statut et temporaires. Plafond de 4 096 entrées distinct du nombre réel de slots : moins de huit slots maximaux simultanés. Saturation : refus avant dispatch.

## RemoteResultDisposition

Identité canonique complète, digest, disposition et raison, persistés au maître avec Task/projection avant réponse. accepted : nouveau ou égal committé ; discarded_terminal : cancelled/stale_round/obsolete_execution/expired prouvés ; rejected_conflict : divergence sans écrasement ; deferred : quota_hold/execution_settlement_pending temporaire. Stockage en échec ou autorité non prouvée : aucune confirmation terminale.

capture_remote_result vérifie la connexion worker et son époque contre le placement durable, sans conn_instances du wrapper mort. RetainedResultAuthority reste réservé à la sortie vers le parent après capture. Rejeu post-commit : même disposition ; quota_hold différé ne devient pas un succès ni un rejet définitif. Le worker persiste la confirmation et cesse les essais sur issue finale.

La disposition finale solde la réserve ; les octets conservés restent comptés. Collecter le blob après preuve de fin du producteur et fermeture irrévocable de la commande ou du slot. Conserver le reçu compact, l’empreinte et la disposition 7 jours en comptant leurs octets. Aucun retrait de blob en concurrence avec son écrivain. Différé, inconnu ou temporaire orphelin : conserver sans inventer résultat ou mort. Un store disparu après enrollment entraîne une quarantaine.

## ReleasePermit

permit_id, command_digest, identité complète, connection_epoch, permission_revision, mapping_revision, provisioning_revision, validité. Émis après relecture des droits sur leur hôte réel. Worker : révisions locales et délais vérifiés avant RELEASE. Préparation au plus 30 secondes ; permis au plus 5 secondes. Timer de préparation démarré avant Prepared et terminé à ReadyToRelease avant 30 secondes ; timer du permis démarré à ReadyToRelease, jamais à sa réception. Deadline maître inchangée. Aucun permis restauré après crash comme autorisation de relancer.

## LineagePlacementV2

placement nullable, machine_id, display_name facultatif, worker_id, last_contact_at attesté au maître, contact_state=connected|stale|unknown. Le statut métier et la disponibilité du lecteur maître restent distincts. Aucun PID, credential ou secret affiché. Réemployer révisions et notifications existantes, sans watch parallèle.

## Stockage

Migrations dans les stores métier existants, avec Fleet schéma 5 explicite. Les anciens lecteurs stricts ne doivent pas reprendre des lignes distantes. Retour arrière seulement par sauvegarde cohérente jointe DB/Fleet/registre/état worker après quiescence, sans perte d’activité postérieure ; sinon rester sur 162 avec route désactivée. Aucun SQL métier sur le worker. Journal durable distant, lecture autorisée à la demande par Attach. Aucun miroir permanent ni nouveau store de curseurs. Les opérations persistantes restent corrélées à la commande et à sa génération. Une fraîcheur dégradée ne devient pas un état métier inventé.

## Raccordements durables observés — 13 octobre 2026

Ces précisions décrivent le code en cours et les décisions retenues. Elles ne prouvent pas une recette exécutée.

- **WorkspacePreparationReceipt** appartient au store technique I40. Champs fermés : version, worker, command, prepare_digest et record optionnel. Le worker écrit le reçu avec record absent avant toute création Git. Une interruption entre création et persistance du record produit une issue inconnue conservée, jamais une autorisation de recréer. Reçus Git, remplacements atomiques et temporaires utilisent le budget et le compteur communs du store.
- **Conflit de blob** : le wrapper peut publier le marqueur immutable payload/conflict sous son verrou de publication. Ce marqueur conserve le premier blob. Il ne donne au wrapper aucun droit d’écrire les métadonnées du worker. Le worker traite le conflit comme une preuve de quarantaine ; les octets restent comptés.
- **WorkerObservation.native_ready** est un booléen avec valeur absente false. Il atteste la fermeture du canal de démarrage sans StartupFailed. La phase Released exprime uniquement la libération du bootstrap. Connected exige en plus le Register exact du wrapper, lié au lease, à son instance et à son nom ; aucune connexion native n’est inventée à partir de Released.
- **Task.remote_prepared** vaut false par défaut. La tâche réservée reste queued jusqu’à la préparation effective, dont le worktree attesté et les permissions reprojetées. **remote_result_slots** et **remote_result_receipts** sont des collections durables, vides par défaut. Un reçu résultat conserve worker, identité d’exécution complète, message_id, digest, disposition et conflict_seen=false par défaut. Les écritures de tâche conservent les preuves déjà persistées.
- **Borne par tâche** : 64 slots et au plus 64 reçus résultat. Le maximum normal décrit par les bornes actuelles est 8 rondes × 3 exécutions quota + 2 reprises processus = 26. Les reprises supplémentaires consomment la même borne de 64. Ce plafond de métadonnées par tâche est distinct du budget worker de 16 Mio et de ses 4 096 entrées ; il ne promet pas 64 blobs simultanés. Saturation : garder l’identité non dispatchée et les preuves existantes.
- **Deferred** n’est pas une clôture. Le worker conserve son slot et son blob. Le maître ne considère pas l’arbre soldé tant qu’un reçu résultat porte cette disposition ; la purge de l’arbre reste empêchée même si les autres conditions terminales sont satisfaites.

La connexion worker commence par DaemonIdentityRequest. DaemonIdentityReport.features doit contenir remote_worker_v1 avant WorkerHello sur cette même connexion. Aucun credential n’est transmis si la capacité manque, si la réponse est invalide ou si la sonde expire. Un ancien maître reçoit seulement le message historique de sonde.
