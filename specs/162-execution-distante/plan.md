# Plan SPEC-162 — Exécution distante explicite

**Statut :** Implemented (46/46 ; livré, G8/G9 validés avec conservations justifiées). **Branche :** main ; branche de session intégrée puis retirée. **Date :** 2026-10-13.
**Estimation annoncée :** 665–1 210 minutes à 09:16 CEST, borne haute le 14 octobre à 05:26. Elle inclut les tests prévus, les revues, la livraison et le nettoyage. Recalibrer après la génération des tâches.

## Architecture retenue

Le maître reste l’unique propriétaire des tâches, réservations, droits et résultats métier. Un mode worker du binaire Bridget utilise la fédération existante. Il possède ses processus, ses journaux et des reçus techniques privés. Il ne possède ni seconde base métier ni ordonnanceur autonome. Le module remote_worker.rs matérialise cette frontière. Il réemploie managed_process, fsutil et les gardes du ReceiptStore.

La sélection est explicite : machine_id et project_id complètent la demande native, avec moteur et modèle imposés. Le client exige la capacité remote_worker_v1 avant une demande distante. NativeDelegationRequest refuse déjà les champs inconnus. L’ancien outil MCP refuse aussi les clés hors allowed. Seul l’ancien registre ignore la section machines : tester ce cas et les refus CLI/MCP historiques. Le maître vérifie la capacité et le placement dans la demande courante, sans se fier à un préflight séparé. Aucun repli local n’est permis. Sans placement, la voie historique reste inchangée.

## Contexte et constitution

Rust, JSON versionné, socket Unix fédérée par SSH, SQLite métier au maître et fichiers privés au worker. Aucune dépendance nouvelle prévue. Le registre agents.json porte une section machines facultative, désactivée par défaut. Aucun environnement ni secret fournisseur du Mac n’est envoyé à Linux.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/plan.md#extrait-1).

## Configuration et identité

Étendre AgentRegistryFile/AgentRegistry et leurs loaders. Au départ : 16 machines et 64 mappings par machine, identifiants bornés et enabled=false par défaut. Chaque machine porte worker_id, révision de provisionnement, empreinte du credential, mappings et types de fournisseur autorisés. Les définitions fournisseur restent locales. Un registre fournisseur épinglé n’est jamais une autorité de configuration machine ; from_resolved/from_json doivent conserver ou exclure cette section explicitement selon leur rôle.

Le credential privé du worker est provisionné par le canal administratif SSH autorisé. Réemployer les types secrets et gardes existants, sans journaliser sa valeur. Le rôle Service existant est aujourd’hui réservé au guichet : ajouter un handshake worker explicite et authentifié, sans affaiblir ce guichet ni assimiler un Register d’agent à une identité de worker. Un ancien maître refuse proprement. Ne pas lui envoyer une variante inconnue d’un enum fermé avant négociation.

Une seule connexion fait autorité par worker. Une reconnexion avec les mêmes worker_id, credential et révision autorisés remplace atomiquement la connexion précédente, ferme celle-ci et reçoit une nouvelle époque. Refuser un credential divergent. Rejeter toute trame de l’ancienne connexion et toute commande non placée sur ce worker. Le verrou local du store garantit un seul worker écrivain. Le nouveau handshake ne restaure ni permis ni droit de recréer un processus inconnu ; il réconcilie les commandes. Désactivation ou révocation bloque les nouveaux permis. Les exécutions existantes restent sous le contrôle d’annulation du maître.

Le worker, les wrappers et les fournisseurs sous le même UID partagent un domaine de confiance. Les modes 0600/0700 ne les isolent pas entre eux. Un fournisseur full-access compromis peut lire le credential ou modifier l’état ; l’authentification prouve la possession du credential, pas l’intégrité du processus. À Prepare et avant RELEASE, refuser tout chevauchement canonique dans les deux sens entre cwd, mappings, writableRoots ou additional_directories et le credential, son répertoire privé ou l’état worker. Une résolution impossible entraîne un refus. Cette garde prévient l’exposition explicite ou accidentelle, pas un accès global dangerFullAccess/bypassPermissions. Le wrapper d’infrastructure peut publier uniquement dans son slot préalloué ; ce slot n’est jamais ajouté aux droits du fournisseur. Un UID séparé ou une sandbox OS restent hors périmètre. Journaliser les remplacements de connexion avec machine, worker et anciennes/nouvelles époques, jamais le secret. Le credential est chargé depuis le fichier privé dans la seule mémoire du worker. Il ne passe jamais par l’environnement du wrapper ou du fournisseur. build_environment/pass_env ne constitue pas un filtre générique de secrets ; ne pas créer une variable d’environnement contenant ce credential.

## Projets, Git et permissions

Le mapping nomme le projet, les racines source et distante et l’identité Git autorisée. Il ne remplace pas un préfixe de chemin. Conserver séparément le cwd d’origine, l’identité du parent, le chemin relatif et le cwd d’exécution. Calculer les chemins par composants canoniques, puis contrôler leur appartenance sur leur hôte réel. Tout writableRoot ou additional_directory exige son mapping. Une règle opaque ou non traduisible entraîne un refus.

Pour le code, vérifier le commit de base avant de préparer le worktree local à la machine choisie. Réemployer la spec 154. Ne pas exiger un HEAD inchangé pendant le travail : les commits et fichiers modifiés dans le périmètre autorisé sont attendus. Ne jamais faire de reset forcé pour retrouver la base. Chaque agent agit dans sa copie et sa branche selon sa mission. Son résultat indique dépôt, branche, commit, tests et disponibilité du push éventuel. Un intégrateur unique gère main. Aucun service de synchronisation de fichiers, forge ou création automatique de PR n’est ajouté. Un worker perdu ou un commit non récupéré interdit le nettoyage de ce travail.

Le maître autorise la preuve parent ; les fichiers qui constituent cette preuve sont relus sur l’hôte d’origine du parent. Cet hôte peut lui-même être un worker. Les sources de l’enfant sont relues sur la destination. Réemployer la profondeur maximale actuelle de 8. Relire la preuve parent dans une opération parent_proof hors verrou pendant Prepare, puis la revalider avant Release. Ce second échange est borné par les délais existants : un dépassement refuse le lancement, sans promettre une latence interhôtes inférieure à 5 secondes. Si l’origine n’est plus attestable, refuser. Ne pas lire un chemin /home sur le Mac, ni envoyer une source Mac comme fichier à ouvrir sur Linux.

native_launch_registry et admit_delegate canonicalisent aujourd’hui les contextes localement. Ils doivent séparer validation de l’origine et préparation distante. native_child_env ne transmet au worker que les références de sources applicables sur cet hôte. Le contrôle Claude parent.cwd==cwd ne doit pas être contourné par une substitution de chaînes : la projection restrictive doit être démontrée. Codex conserve approbation, reviewer, réseau et exclusions ; aucun élargissement workspaceWrite vers dangerFullAccess. Claude conserve les règles permises et interdites. Capturer les sources gérées propres à Linux, dont /etc/claude-code, et réserver MDM à macOS. Capturer les révisions ; laisser Claude appliquer sa propre fusion. Les règles et helpers non maîtrisés restent refusés.

## Lancement : trois frontières nécessaires

Une enveloppe WorkerRequest/WorkerResponse couvre prepare, start, release, reconcile, cancel, parent_proof et journal_page. reserve_result_slot réserve idempotemment la place du résultat avant chaque dispatch. final_result monte du worker ; result_disposition revient du maître. WorkerHello porte identité, version et époque. Réutiliser le framing borné ; le contrat détaille chaque sens et autorité. Aucune seconde réservation Fleet distante : Fleet reste au maître. reserve_result_slot ne réserve que le stockage technique du résultat. Le quota Fleet est global, toutes catégories confondues. Les limites fournisseur 157 s’ajoutent par agent_type. Local et distant consomment ces mêmes plafonds ; aucun quota par machine. Les slots techniques du worker sont une contrainte supplémentaire de stockage, pas un quota fournisseur.

1. **Prepare** : observer configuration, binaire, cwd et permissions du worker sans processus. Appliquer la structure de delegate_worktree : probe sous verrou, préparation hors verrou, réservation sous verrou et validation finale. Aucun appel réseau ni lecture distante lente sous le mutex DaemonState. Cette règle couvre aussi parent_proof, release, cancel, reconcile, les journaux et le relais de résultat : préparer sous verrou, effectuer les E/S hors verrou, puis revalider la révision avant mutation.
2. **Start** : le maître a validé Prepared et réservé commande, génération et capacité. Le worker persiste le reçu avant création. Il réemploie spawn_managed_bootstrap_with_stderr, wait_ready_or_cancel et persist_marker. Garder le MarkedManagedChild bloqué jusqu’à une autorisation indépendante.
3. **Release** : le maître recontrôle origine, mapping et politiques puis émet un permis corrélé au canon, à l’époque et aux révisions. Le worker persiste ce permis, relit ses sources et libère une seule fois. Une révocation avant émission interdit le permis. Après émission, le maître annule l’exécution corrélée ; aucune transaction atomique entre machines n’est prétendue.

Réemployer lifecycle::launch_preflight/build_environment. Extraire prepare_spawn_parts en pub(crate) uniquement si nécessaire. Ne pas appeler submit_spawn au worker, car il réserverait une seconde Fleet. Ne pas appeler handle_managed_command inchangé : il réserve et libère immédiatement. FleetSupervisor::request_spawn/mark_starting restent au maître.

Préparation valide 30 secondes et permis valide 5 secondes au maximum, sous les délais de démarrage existants. Le worker démarre le délai de préparation avant sa réponse Prepared ; ReadyToRelease doit survenir avant la fin des 30 secondes. Le délai de permis commence à cet événement local ReadyToRelease, jamais à la réception tardive du permis. Le maître contrôle aussi ses propres délais. Le maître conserve les échéances métier. Crash ou reconnexion invalide le token et le permis en mémoire, sans nouvelle libération automatique. À expiration de Prepare ou Release, appeler abandon et fermer le canal RELEASE : le bootstrap bloqué sort sur EOF. Attester ensuite la fin avant de libérer la capacité.

## Durabilité, résultat et reprise

Stocker le placement canonique dans Task.remote_placement Option avec valeur par défaut, et dans DesiredEquipier.remote_placement Option avec valeur par défaut. DesiredFleet passe explicitement du schéma 4 au schéma 5 : lecture de 1 à 5, écriture en 5. La requête stockée conserve machine_id/project_id ; ne pas retirer ses champs pour prétendre une compatibilité ancienne. Les anciennes lignes sont locales pour le binaire 162. Les lecteurs précédents sont stricts : aucun downgrade sur un état 162 inchangé, même si une partie du JSON reste lisible. Créer un store technique worker distinct : ReceiptStore conserve son format Seen/Acked à deux lignes et n’est pas modifié pour porter ces données. Réemployer seulement ses gardes d’enrollment, de verrou, de quarantaine et les primitives fsutil.

Le worker est l’unique écrivain des métadonnées WorkerCommandReceipt. Le wrapper est l’unique producteur d’un blob résultat immutable dans un slot privé préalloué et corrélé à sa mission, ronde et exécution. Il ne modifie jamais les métadonnées du worker. Le worker ne remplace jamais le blob publié. Réemployer write_private_file_atomic dans ce namespace contrôlé : avant publication, vérifier propriétaire, absence de symlink et producteur unique. Si le blob existe déjà, égal signifie lecture du même résultat ; divergent signifie quarantaine, jamais remplacement. Ne pas réutiliser le FD managed-status fermé après le démarrage, ni ouvrir ReceiptStore depuis deux processus.

Le reçu suit reserved, marked, release_authorized, released puis terminal/refused/unknown. Un crash après création mais avant marqueur reste ambigu : ne pas recréer. Après marqueur ou permis, observer le processus attesté ; ne pas reconstruire un canal RELEASE. Connected exige le Register réel du wrapper. Unknown conserve la capacité jusqu’à preuve terminale fiable ou intervention humaine hors périmètre. Un worker perdu peut donc réduire aussi la capacité disponible pour les tâches locales, puisque le quota Fleet est global. Aucun outil de reset destructif n’est ajouté.

Le résultat final natif nécessite un correctif limité au couloir distant 162. Aujourd’hui send_wrapper_message perd le writer après erreur et ignore les ACK ; le journal asynchrone ne fournit pas cette durabilité. Avant réseau, le wrapper écrit dans son blob l’enveloppe finale canonique : message_id, tâche, mission, ronde, instance, génération, execution_id, génération d’exécution, corps et empreinte. Le wrapper distant n’envoie jamais ce résultat par Send ou SendIdempotent ; il termine la publication locale du blob, puis le worker est l’unique émetteur final_result, y compris après sa mort. Pour une mission distante courante ou passée, capture_reply retourne l’erreur remote_result_requires_worker avant récupération enfant, mutation ou traitement terminal. Ses deux appelants le convertissent en Nack : jamais Ack, Ok(false) ou continuation vers prepare_dispatch. Aucune nouvelle inscription ni relance fournisseur.

Ajouter capture_remote_result comme garde d’entrée distante. Vérifier la connexion worker courante et son époque, le placement durable, toutes les identités de commande et d’exécution et le canon. Extraire la mutation métier partagée de capture_reply ; conserver ses gardes locales. L’admission distante ne dépend pas d’une conn_instances vivante du wrapper. RetainedResultAuthority intervient seulement après capture pour la notification sortante vers le parent, jamais comme autorité d’entrée.

Persister la disposition canonique dans la même transaction Task/projection que le résultat, avant confirmation. Issues fermées : accepted pour un résultat nouveau ou égal ; discarded_terminal avec raison cancelled, stale_round, obsolete_execution ou expired après preuve irréversible ; rejected_conflict pour canon divergent, sans écraser le premier ; deferred pour quota_hold ou execution_settlement_pending. Une panne de stockage ou une autorité non prouvée ne produit aucune confirmation terminale. Quota_hold est temporaire : conserver le slot et reprendre de manière bornée sur échéance ou fait nouveau, sans boucle immédiate.

Le worker persiste la disposition retournée. Une issue finale arrête la réémission ; un ACK perdu restitue la même disposition durable. accepted rejoint result_message_id et wait 149–153. Un parent absent retrouve son résultat. discarded_terminal et rejected_conflict ne sont pas des succès métier ; conserver leur preuve sans réémission infinie. Le résultat de mission reste distinct de l’intégration Git.

Ajouter reserve_result_slot idempotent, sous autorité du maître courant. Figer l’identité d’exécution avant cet échange ; le canon de réservation contient tâche, mission, ronde, instance, génération, execution_id et génération d’exécution, avec digest de ce canon d’admission. Le digest du résultat encore inconnu n’est fixé qu’à sa publication. Réserver hors verrou puis revalider identité, révisions et autorisation sous verrou avant dispatch. Initialement avant RELEASE ; après begin_round avant send_as mission_pending ; après reconstruct_quota_failed et sauvegarde resume_execution_id avant deliver_reconstructed. Un refus laisse la même identité non dispatchée, sans nouvel UUID. Un slot devenu caduc n’est libéré qu’après preuve durable de non-dispatch ou d’annulation du slot, sans tuer le wrapper réutilisé. La clé inclut l’exécution pour quota 160. Corps plafonné à 262 144 octets ; enveloppe JSON sérialisée plafonnée à 2 Mio, métadonnées fermées et bornées. Le budget total de 16 Mio compte blobs, réservations de 2 Mio, métadonnées anciennes et nouvelles pendant écriture atomique, fichiers d’état et temporaires orphelins. Garder aussi le plafond de 4 096 entrées ; il ne promet pas 4 096 résultats simultanés. Moins de huit slots maximaux tiennent dans 16 Mio. Refuser avant dispatch si la réserve manque.

Un worker arrêté pendant publication ne supprime pas le producteur. À sa reprise, relire le blob validé et le reçu sous verrou. Un temporaire ou une absence ne prouve pas un résultat. Collecter le blob après disposition finale durable, producteur terminal prouvé et commande ou slot irrévocablement fermé. Conserver le reçu compact, l’empreinte et la disposition pendant 7 jours en comptant leurs octets. Une disposition finale solde la réserve de publication ; les octets réellement conservés continuent à compter. Un résultat différé ou ambigu n’est jamais purgé automatiquement. Une commande ou un slot encore vivant peut conserver son blob et saturer le budget ; cette limite reste explicite.

Quota 160 garde son instance et ne devient pas un spawn. Une reprise 152 qui crée une génération exige la preuve terminale de la précédente. Machine, moteur, projet et échéance restent fixés. Seul le maître applique les gels autorisés ; déconnexion n’est pas prolongation.

## Journal, contact et annulation

Le journal durable reste sur le worker. Les pages froides passent par journal_page vers le worker. Leur racine est dérivée de l’instance et de la génération attestées par le reçu, selon le patron ManagedStderrStore. Aucun champ de chemin arbitraire fourni par le CLI ni nouveau champ de racine stocké. Réemployer le lecteur local Attach sur le worker. Pour le suivi vivant, conserver LiveJournalFragment et AttachView déjà relayés au maître ; aucune nouvelle famille follow/stop distante. Réemployer les bornes, annulations d’abonnement et séquences existantes. En cas de gap, reprendre par page froide autorisée ; worker perdu, annoncer le gap et l’indisponibilité, sans inventer des lignes. Ne pas utiliser la lecture froide des sessions locales du parent pour une tâche distante. Aucun miroir permanent ni nouveau store de curseurs. Worker indisponible : journal explicitement indisponible, dernier état et résultat déjà reçus conservés au maître.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/plan.md#extrait-2).

Cancel vérifie machine, worker, commande, instance, génération, PID et naissance via managed_process. PID seul interdit. Sans contact, garder l’annulation en attente ; ne pas inventer stopped. Les demandes de permission enfant 155 et l’état métier gardent leur autorité au maître.

## Lineage adaptateur de bureau

Garder la réponse v1 par défaut strictement inchangée. `bridget lineage inspect --response-version 2` enrichit uniquement list/show. La version de projection 2 est distincte de l’authentification HumanLineageViewV1. Conserver la capacité de lecture existante et refuser proprement une version non supportée.

Le nouveau reader tente v2 puis une seule lecture v1 sur refus structuré unsupported_version ou invalid_request attribué à cette tentative. Abandonner tout le staging v2 puis reprendre depuis le curseur nul. Timeout, refus d’autorisation et données malformées restent des erreurs. Aucun repli pour watch, journal ou cancel. Garder les limites de pagination, propriétaire et projet.

Succès list/show v2 : version=2. Les erreurs structurées gardent leur enveloppe historique version=1 ; le nouveau client accepte explicitement cette forme pour une tentative v2. Toute autre version de succès est une erreur. V2 expose placement et contact worker attesté ; last_contact_at est affiché comme observation au moment de lecture de la page, jamais comme âge vivant recalculé sans lecture. En v1 de repli, afficher « Métadonnées distantes indisponibles ». La projection existante thread:bridget-task n’est pas une conversation fournisseur ; conserver activeProviderThreadId nul et son absence de la colonne des conversations.

## Surfaces prévues

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/plan.md#extrait-3).

Étendre les tests 149/152/154/155/160 et les suites registre/protocole/processus. Le besoin d’un fichier de tests worker est soumis au réemploi, sans suite parallèle redondante.

## Étapes et livraison

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/plan.md#extrait-4).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/plan.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/plan.md#extrait-6).

## Déploiement et supervision du worker

Sur Cartae, systemd --user répond et Linger=yes. L’unité bridget-worker-cartae.service n’existe pas encore. Réutiliser les procédures 095 et le déploiement 161 pour installer un binaire versionné ; aucun clone de travail n’est requis pour exécuter le worker. Le tunnel 095 reste maintenu sur le Mac, sans tunnel par mission.

Provisionner /home/moi/.config/bridget/worker-cartae.credential en 0600 et /home/moi/.local/share/bridget-worker/cartae en 0700, propriétaire moi. Ne jamais consigner la valeur du credential. Le registre maître garde seulement son empreinte et sa révision. Le répertoire de configuration existant est déjà privé ; ne pas modifier les répertoires partagés de l’utilisateur.

Installer l’unité utilisateur dans /home/moi/.config/systemd/user/bridget-worker-cartae.service avec chemins absolus et binaire versionné. Réemployer les limites et la politique de reprise du service existant 095. Arrêt du worker par SIGTERM, KillMode=process et SendSIGKILL=no : ne pas tuer les wrappers vivants lors d’une relance du superviseur. Arrêter le worker et annuler ses missions sont deux actions distinctes. Après redémarrage, reprendre le verrou et réconcilier les marqueurs, blobs et reçus avant tout lancement nouveau.

Ordre : route désactivée, installation et sauvegardes, credentials privés, unité inactive validée, démarrage worker, handshake et réconciliation, activation du mapping autorisé, recette synthétique. Retour arrière : fermer les nouvelles admissions, annuler seulement les identités attestées si demandé, conserver les preuves inconnues, arrêter le worker puis restaurer l’unité ou désactiver la route sans effacer le store. Le downgrade du binaire maître suit la règle stricte ci-dessous. Aucun privilège root ni nouvelle infrastructure.

Préflight réel ROOT : /Users/moi/.cache/bridget-delivery162-20261013/preflight/cartae-worker-supervisor.json, SHA-256 736d736d71e35cc8dbdbfc9723653149bc00a67117a489e293223f15a51e5b92. Deux portails desktop déjà en échec restent hors périmètre ; ils ne prouvent pas un défaut du worker absent.

## Retour arrière des stores stricts

Avant activation, fermer les admissions, vérifier la quiescence et produire une sauvegarde cohérente DB, Fleet, registre et état worker. Un ancien binaire ne doit jamais reprendre un état contenant du distant. Le rollback complet exige une restauration jointe de cet ensemble, des missions réconciliées et l’absence de perte d’activité depuis l’activation 162. Archiver l’état et les preuves post-162 avant toute restauration autorisée. S’il existe une activité postérieure non récupérable dans la sauvegarde, rester sur 162 avec route désactivée et conserver l’état et les preuves. Ne pas restaurer une vieille DB par-dessus des résultats récents, ni promettre qu’un schéma additif est lisible par les anciens types stricts.

## Gates de production encore ouvertes

### Checkpoint courant — 13 octobre 2026

Le plan est validé et le développement/préparation atteint 42/42 sur preuves ciblées, avec limites documentées. Converge et les gates de livraison G8/G9 restent à exécuter ; aucun pipeline achevé n’est déclaré.

L’utilisateur a autorisé Bridget, Roadie et Opus2D ; Cartae est exclu. Le reçu /Users/moi/.cache/bridget-delivery162-20261013/provision-projects162.final.json atteste les trois clones Linux propres, leurs origines et commits. Les configurations candidates privées dans /Users/moi/.cache/bridget-delivery162-20261013/worker-config-candidate portent machine cartae, worker worker-cartae, révision 1 et les trois mappings, avec routes désactivées. Codex login status est RC0 ; aucun quota ni appel réel de modèle n’est prouvé. Le reçu opérationnel /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/validation/livraison162.md distingue ces préparatifs de l’installation, de la recette réelle et de l’activation encore NON EXÉCUTÉES.

### Constats historiques du préflight initial — remplacés par le checkpoint ci-dessus

ROOT attend le choix de projets autorisés. Cartae possède /home/moi/20.cartae, dépôt guthubrx/cartae, HEAD observé 75e0ad6c et une modification à préserver. Le chemin Mac /Users/moi/20.cartae est absent et /Users/moi/11.Repositories/cartae n’est pas un dépôt Git. Aucun mapping de production ne peut être déduit de ces noms. Aucun clone Bridget distant n’a été trouvé aux chemins vérifiés.

Les binaires fournisseur présents ne prouvent pas leur authentification. Le seul modèle déclaré distant observé est codex gpt5.6terra ; aucune disponibilité réelle n’est déduite. Les décisions de projet, credentials privés et contrôles de permissions réels conditionnent la recette et l’activation de chaque route. Leur absence n’empêche pas l’implémentation complète et les recettes isolées. Ne pas déclarer une route activée tant que ses preuves manquent.

## Fallback SpecKit

Sync exécuté par ROOT et sélection 162 vérifiée. setup-plan.sh tenté après sélection : RC127, script absent. Templates spec/plan et extensions.yml absents aussi. Appliquer le fallback documentaire des skills lues, sans installer de toolkit. Aucun tasks.md avant les deux gates. ROOT gère AGENTS et les pointeurs hors propriété documentaire.


## Checkpoint après Converge1 — 42/43, développement en cours

Converge1 est clos avec verdict non convergé : une trace de remplacement de connexion worker, demandée par T008, manque. Ce constat porte sur la journalisation sans secret, pas sur les autorisations ou le lancement. ROOT ajoute T043 après la clôture du passage ; le correctif est préparé hors phase de convergence. Les 42 tâches précédemment validées restent versionnées, T043 est ouverte jusqu’à preuve ROOT. G8/G9 restent ouverts. Ce checkpoint remplace le précédent statut 42/42 ; aucune livraison n’est déclarée.


Checkpoint courant après T043 : le correctif de trace est relu et son oracle ciblé passe. Développement/préparation Implemented, 43/43. Converge1 reste historiquement non convergé ; les artefacts sont figés pour Converge2 ciblé. Aucun verdict de ce prochain passage ni résultat de livraison G8/G9 n’est anticipé.


Checkpoint courant après contre-revue code R1 : verdict BLOCKED, manque confirmé de réconciliation starting/refus durable. T044 est ajoutée après Converge2 pour corriger ce raccord et ses deux E2E. Converge2 reste un verdict historique sur la source précédente ; le développement est rouvert à 43/44, sans passage 3 encore. La préparation de l’audit sur la source pré-R1 est suspendue ; aucun déploiement anticipé.

## Checkpoint final T044 — développement 44/44, avant Converge3

R5 valide la dernière correction worker : **17 tests worker PASS et 4 E2E PASS, zéro échec/ignoré, RC0**, en 58,317 s au total. Compilation : 21,83 s selon ROOT. Le reçu atteste sources_unchanged=true. R4 conserve ses 73 lib PASS et 4 E2E PASS sur la source précédente ; les primitives et le maître sont inchangés selon ROOT. Ces replays ne sont pas additionnés comme des tests distincts et ne remplacent pas la FULL historique en échec.

La lecture du worker final confirme un inventaire groupé des commandes et slots dans reconcile_all, puis un index temporaire des commandes Terminal/Refused. La jointure vérifie l’identité complète et réemploie finish_result_slot. Les commandes actives gardent leur réconciliation propre. Aucun store, flag persistant ni protocole nouveau. Aucun gain de latence mesuré n’est revendiqué.

T044 est cochée après lecture des reçus R4/R5 et de cette correction. État courant **Implemented, 44/44 pour le développement et la préparation**. G8/G9 restent ouverts. Converge3 est à exécuter ; aucun verdict favorable de ce passage, d’audit ou de contre-revue n’est anticipé. Le BLOCKED R1 et Converge2 restent historiques. SHA-256 de tasks.md gelé : 746adf1b544e476ffe21e79131cb9ec2ad0b89c6bd90c0870475ecd242e800ea.
