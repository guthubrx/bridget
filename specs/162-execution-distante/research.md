# Recherche SPEC-162

**Date :** 2026-10-13. Lectures ciblées et synthèses des agents placement162, lineage162, code161 et reuse161. Aucun build, test, provisionnement ou service lancé par cette recherche.

## Autorité unique et réemploi

La fédération 095/096/161 transporte une socket vers le maître ; elle ne place pas les processus. managed_process fournit bootstrap bloqué, marqueur et contrôle de naissance. Fleet, idempotency et desired_state gardent l’autorité métier. Le worker réemploie ces primitives, sans seconde base métier. Un SSH exec par mission créerait une autre voie sans les garanties de reprise ; cette option est rejetée.

registry.rs:85/120/779/960 fournit structure, chargement et gardes du registre privé. L’ancien parser ignore les champs inconnus : une capacité distante doit précéder toute mutation. IdentityCredential, protocol.rs:2359, masque les valeurs secrètes. Le générateur actuel dans daemon.rs:8588 peut être extrait, pas remplacé par un service neuf. Le rôle Service existant ne constitue pas à lui seul une autorisation worker.

## Permissions et préparation

native_permissions.rs:464–630 relit aujourd’hui les contextes parent et enfant localement. native_launch_registry et admit_delegate canonicalisent aussi leurs cwd sur le maître. La séparation origine/destination est donc indispensable. Un parent déjà distant doit être relu sur son hôte attesté. La projection de ses droits doit rester restrictive.

delegate_worktree, native_delegation.rs:864, fournit le modèle probe sous verrou, préflight hors verrou, réserve sous verrou, création hors verrou, commit revalidé. lifecycle::launch_preflight et build_environment sont réutilisables. Le worker doit garder le MarkedManagedChild jusqu’à un RELEASE distinct ; handle_managed_command libère trop tôt pour ce contrat.

Documentation primaire consultée : [réglages gérés Claude](https://code.claude.com/docs/en/managed-settings), [réglages](https://code.claude.com/docs/en/settings), [permissions](https://code.claude.com/docs/en/permissions), et synthèse des [réglages gérés par serveur](https://code.claude.com/docs/en/server-managed-settings). Linux emploie /etc/claude-code, avec fichiers et fragments gérés. Capturer les sources et leur révision ; ne pas reproduire leur fusion. Ces documents ne prouvent pas la version ni les droits réels du CLI installé. Les règles non traduisibles restent refusées.

## Résultat durable : manque réel identifié

wrapper.rs:6724 construit la réponse puis appelle send_wrapper_message. Cette fonction, vers 5531, ne fournit ni reçu sortant durable ni confirmation et perd le writer après erreur. Les événements sont consommés en mémoire. Le journal fournisseur est asynchrone et son flush ne fournit pas un fsync. Il ne suffit donc pas pour promettre une publication reprise après crash.

Le maître appelle capture_reply avant son ACK (daemon.rs:21821). save_projected persiste payload et projection dans une transaction avant notification. Cependant capture_reply absorbe aussi des messages obsolètes sans vérifier leur canon : son ACK legacy ne prouve pas toujours l’égalité du résultat.

Décision R2 limitée au distant 162 : store technique distinct du ReceiptStore à deux lignes. Le worker écrit seul ses métadonnées ; le wrapper écrit seul un blob immutable dans un slot préalloué. Réemployer fsutil avant réseau, avec contrôle du namespace et absence de remplacement. Ajouter capture_remote_result sous autorité de la connexion worker et du placement durable, même après mort du wrapper. RetainedResultAuthority intervient seulement ensuite, pour la notification sortante. Disposition persistée dans la transaction métier : accepted, discarded_terminal, rejected_conflict ou deferred. Quota_hold est temporaire, pas une fin. Réserve comptant l’enveloppe sérialisée de 2 Mio, les métadonnées et temporaires, dans le plafond de 16 Mio. Pas de framework d’outbox ni second store métier. L’audit de réemploi est actualisé après cette correction.

## Journal et contact

Journal durable sur le worker ; Attach à la demande via le maître autorisant chaque lecture. Pas de lecture froide des sessions locales pour une tâche distante. Pas de miroir permanent. Worker perdu : indisponibilité explicite du journal, état et résultat déjà reçus conservés.

Contact fondé sur le transport, jamais sur task.updated_at. Heartbeat existe dans protocol.rs:3646 ; son traitement daemon.rs:22181 vise actuellement présence.conn_instances. Étendre ce traitement au worker authentifié et à son époque, sans nouvelle famille ping. Une trame toutes les 10 secondes, seuil de 30 secondes. Horodatage volatile, notifications seulement lors des transitions, séquence modifiée seulement aux transitions de contact pour invalider les curseurs ; last_contact_at observé par page, sans gel global ni invalidation à chaque heartbeat.

## Compatibilité adaptateur de bureau et Git

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/research.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/research.md#extrait-2).

## Limites et workflow

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/research.md#extrait-3).

Sélection 162 vérifiée. setup-plan tenté : RC127, absent. Templates et extensions absents. Fallback documentaire des skills sans installation de toolkit. Les artefacts restent Draft jusqu’à audit de réemploi et contre-revue. Aucun tasks.md créé.

## Corrections vérifiées après contre-revue R1

ReceiptStore impose un handle unique et un format Seen/Acked ; il ne doit pas être ouvert par deux producteurs pour stocker un résultat. Le FD managed-status est fermé au succès du démarrage et ne fournit pas un IPC durable de résultat. D’où les responsabilités séparées metadata/blob, sans nouveau canal.

capture_reply dépend d’une connexion enfant vivante. Son mutation métier peut être extraite, mais le relais worker nécessite une garde d’entrée distincte. Le quota temporaire, l’annulation, l’ancienne ronde et le conflit de résultat exigent des dispositions explicites. Le résultat retenu pour notification au parent n’est pas une autorité pour la capture initiale.

Attach lit des fichiers locaux : étendre le contrat avec journal_page et exécuter le lecteur sur le worker, racine issue du reçu. Le flux vivant réemploie LiveJournalFragment et AttachView, daemon.rs:21484/21507 ; aucune nouvelle famille follow/stop. Un gap se reprend par page froide autorisée. Ajouter parent_proof pour la relecture sur l’hôte réel. Garder la profondeur existante de 8, sans retirer les parents distants du périmètre. Toute E/S réseau reste hors verrou DaemonState.

La reconnexion authentifiée remplace l’ancienne connexion fantôme avec une nouvelle époque. Chaque commande reste liée au placement durable de ce worker. Aucun handshake ne ressuscite un permis expiré ou un lancement ambigu.

## Supervision vérifiée

ROOT a observé systemd --user joignable, Linger=yes et aucune unité worker existante. Credential prévu en 0600 sous /home/moi/.config/bridget ; données en 0700 sous /home/moi/.local/share/bridget-worker/cartae. Tunnel 095 conservé au Mac.

La [documentation systemd.kill](https://raw.githubusercontent.com/systemd/systemd/main/man/systemd.kill.xml) précise que KillMode=process laisse les enfants survivre et déconseille cet usage général. Le choix est ici explicite : préserver les wrappers attestés lors d’une relance worker, puis les réconcilier. SendSIGKILL=no et SIGTERM évitent un arrêt forcé implicite. La [documentation systemd.service](https://raw.githubusercontent.com/systemd/systemd/main/man/systemd.service.xml) encadre Restart=on-failure et les délais de reprise. La recette doit vérifier ce comportement avec un enfant synthétique. Aucun changement d’infrastructure ou privilège root.

## Limite de confiance sous un UID commun

Le worker, les wrappers et les fournisseurs sous le même UID partagent un domaine de confiance. Les modes 0600/0700 ne les isolent pas entre eux. Un fournisseur full-access compromis peut lire le credential ou modifier l’état ; l’authentification prouve la possession du credential, pas l’intégrité du processus. À Prepare et avant RELEASE, refuser tout chevauchement canonique dans les deux sens entre cwd, mappings, writableRoots ou additional_directories et le credential, son répertoire privé ou l’état worker. Une résolution impossible entraîne un refus. Cette garde prévient l’exposition explicite ou accidentelle, pas un accès global dangerFullAccess/bypassPermissions. Le wrapper d’infrastructure peut publier uniquement dans son slot préalloué ; ce slot n’est jamais ajouté aux droits du fournisseur. Un UID séparé ou une sandbox OS restent hors périmètre. Journaliser les remplacements de connexion avec machine, worker et anciennes/nouvelles époques, jamais le secret. Le credential est chargé depuis le fichier privé dans la seule mémoire du worker. Il ne passe jamais par l’environnement du wrapper ou du fournisseur. build_environment/pass_env ne constitue pas un filtre générique de secrets ; ne pas créer une variable d’environnement contenant ce credential.

## A, B et C après la ronde 2

Task tolère les champs additionnels, mais sa requête NativeDelegationRequest et les types DesiredEquipier/Fleet sont stricts. Choix : Task.remote_placement et DesiredEquipier.remote_placement avec valeur par défaut, Fleet schéma 5 explicite. Garder les champs de la requête stockée. Aucun ancien binaire sur état 162 ; rollback par sauvegarde cohérente jointe seulement sans perte post-activation.

Send et SendIdempotent appellent capture_reply ; Ok(false) laisse continuer vers prepare_dispatch. Pour le distant, l’erreur remote_result_requires_worker doit donc devenir Nack dans les deux appelants. Le wrapper n’émet que son blob ; seul le worker publie final_result.

Les rondes et la reprise quota ne repassent pas toutes par start. Ajouter reserve_result_slot : après begin_round avant send_as mission_pending, et après sauvegarde resume_execution_id avant deliver_reconstructed. Le canon d’admission est connu avant exécution ; le digest du résultat final ne l’est pas. Rejeu même slot, refus avant dispatch, clôture d’un slot caduc seulement sur preuve durable.

## Complément de déploiement G8 — prérequis Git et sandbox Linux

Deux refus réels provenaient des prérequis de l’hôte. La garde existante de /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/workspace_git.rs exige Git ≥ 2.44 ; le système fournissait 2.43. La correction de déploiement conserve le Git système et place /home/moi/.local/lib/bridget-tools/git-2.56.0/bin en tête du PATH du worker. Le reçu /Users/moi/.cache/bridget-delivery162-20261013/git162-validation.json atteste Git 2.56.0 et les contrôles version/identité/HEAD sur Bridget, Roadie et Opus2D. Son champ worker_modified=false décrit cette validation avant raccord du PATH, pas une preuve de l’environnement final du service.

La [documentation primaire OpenAI](https://learn.chatgpt.com/docs/sandboxing#prerequisites), relue le 13 octobre 2026, recommande bubblewrap et le profil AppArmor Ubuntu ciblé lorsque la création des espaces utilisateurs est restreinte. Pour Ubuntu 24.04, elle indique le profil supplémentaire bwrap-userns-restrict. La variante qui désactive la restriction globale n’est pas retenue.

Le profil provient du paquet Ubuntu apparmor-profiles 4.0.1really4.0.1-0ubuntu0.24.04.8, extrait et vérifié avant installation ciblée. Le reçu /Users/moi/.cache/bridget-delivery162-20261013/sandbox162-profile-installed.json atteste le fichier /etc/apparmor.d/bwrap-userns-restrict, SHA256 11d39094f044f0cda0febb3ad517b830301da6b2ce929664af09ee9e4dd264f9, bwrap/unpriv_bwrap en enforce et la restriction globale conservée à 1. Il s’agit d’un prérequis hôte de la sandbox fournisseur existante, pas d’un nouveau moteur d’isolation Bridget ni d’une séparation des fournisseurs par UID.

La preuve /Users/moi/.cache/bridget-delivery162-20261013/sandbox162-probes-r3-result.json conserve le RC0 et les refus effectifs d’écriture et de réseau. La sonde précédente attendait le refus réseau au connect ; le noyau refusait déjà la création de socket avec EPERM. R3 accepte ce refus explicite à l’une ou l’autre opération. Cette correction d’oracle ne change pas les permissions. Les essais précédents restent historiques ; aucun succès global ni appel réel de modèle n’est déduit de cette sonde.

Le retour arrière est limité au profil ajouté, après quiescence des sandboxes et comparaison d’empreinte : déchargement par /sbin/apparmor_parser -R /etc/apparmor.d/bwrap-userns-restrict, puis retrait du seul fichier attesté absent avant installation. Une modification ultérieure interdit ce retrait automatique. Les sysctl et autres profils restent inchangés. La procédure détaillée figure dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/162-execution-distante/quickstart.md ; aucun retour arrière n’a été exécuté par cette mise à jour documentaire.
