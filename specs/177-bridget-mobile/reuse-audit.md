# Audit de reutilisation de l'existant — Bridget Mobile 177

## Decision

Statut: PASS
Date: 2026-10-24
Feature dir: /home/moi/10.Scripts/64.bridget/.worktrees/177-bridget-mobile/specs/177-bridget-mobile

Conclusion courte: Le plan réutilise le moteur, les reçus et la fédération existants. Les nouveautés portent sur l'identité d'appareil, la façade HTTP et Android. La migration impose d'abord une preuve de portabilité ; aucune bascule n'est déduite de l'existence d'un tunnel.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 14 |
| Items audites | 14 |
| Reutilisations deja prevues | 10 |
| Existants potentiellement pertinents | 2 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 8 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Namespace privé | Namespace | crates/bridget-daemon/src/environment.rs:18 | Étendre les fichiers connus, pas un second gestionnaire de chemins |
| Client interne | DaemonConnection | crates/bridget-daemon/src/communication/client.rs:1039 | Budgets et lecture bornée existants |
| Annuaire | Router::list_agents | crates/bridget-core/src/router.rs:151 | Projeter puis filtrer la portée ; pas de cache métier mobile concurrent |
| Projet | resolve_communication_project | crates/bridget-daemon/src/communication.rs:81 | Réutiliser l'identité canonique, pas le nom de dossier |
| Envoi humain et reçu | handle_idempotent_send_with_result_authority | crates/bridget-daemon/src/daemon.rs:12092 | Le client humain nu est déjà distingué de l'agent ; réutiliser |
| Réconciliation | IdempotencyStore::lookup | crates/bridget-daemon/src/idempotency.rs:1250 | Lire le reçu d'origine |
| Journal | lineage_journal_page | crates/bridget-daemon/src/attach.rs:34 | Réutiliser pagination et séquences après adaptation de l'autorité |
| Tâches et annulation | handle_delegation_cancel / human_cancel | crates/bridget-daemon/src/daemon.rs:13341 | Garder les droits, clé et états d'annulation |
| Questions humaines | human_inbox::resolve | crates/bridget-daemon/src/human_inbox.rs:252 | Réutiliser options et acquittement |
| Installation et entretien | federate.rs, scripts/federate-ssh.sh, scripts/build.py | docs/federation-services.md:14 | Un seul raccordement et un seul entretien Rust |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Autorité humaine mobile | human_principal_actor et human_authority_guard | referent_control.rs:31 ; daemon.rs:18115 | Créer une identité d'appareil ; réutiliser les opérations mais ne pas usurper le libellé ou la conversation |
| Migration du maître | reattest dépend du runtime d'origine | t3code_mcp.rs:39 ; native_delegation.rs:4017 | Prouver une adaptation liée à la machine d'origine avant bascule ; aucun relâchement de validation |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucun | Aucun | Recherche ciblée ci-dessous | Garder le moteur natif unique |

## Items nouveaux audités

| Item du plan | Recherche | Verdict | Justification |
|---|---|---|---|
| Associations, appareils et sessions | pairing / mobile / device / authorization dans crates et scripts | CREER | Pas de principal appareil ni d'association mobile dans le noyau |
| Façade HTTP intégrée | TcpListener / axum / server HTTP dans crates et manifestes | CREER | minreq est un client ; tungstenite n'est pas un serveur HTTP applicatif |
| Application Android et stockage local | android / mobile / Gradle dans les fichiers suivis | CREER | Aucun module Android existant |
| Notifications de téléphone et APK | notifications / push / signing / Android | CREER | Le canal humain existant peut notifier un service, pas gérer l'identité et l'installation Android |

Ces quatre lignes complètent les dix responsabilités réutilisées : quatorze items du plan audités. Les deux voisins d'autorité ne sont pas des doublons, leurs contrats ne couvrent pas l'appareil autonome.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| AGENTS.md | Travail serveur et worktree dédié | Code sur Cartae, main préservé |
| AGENTS.md, builds | Wrapper Cargo obligatoire et cache réutilisé | Pas de Cargo direct ni target par test |
| Constitution XIX/XX | Réemploi et charge future | Aucun moteur ou orchestrateur mobile parallèle |
| Constitution VII | ADR des choix structurants | Décisions enregistrées avant le code |
| my-specify-all explicite | Pipeline complet et gates | Pas de commit de livraison partielle présenté comme complet |
| Choix utilisateur | Identité Bridget et documentation bilingue | Implémentation propre et aucune filiation produit revendiquée |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 089 | Identité et noyau | Aucun agent synthétique pour le téléphone |
| 099 | Reçu durable | Réconciliation sur la clé d'origine |
| 138 | Projet canonique | Portée appliquée avant exposition |
| 145–147 | Vues humaines et séquences | Réutiliser les lectures, adapter la garde |
| 148–150 | Droits et tâches | Pas de permission héritée inventée |
| 155 | Questions et décision humaine | Réutiliser les choix et acquittements |
| 162 / 168 | Worker et admission distante | Préserver les contrats au déplacement |
| 173–176 | Confidentialité et entretien | Docs EN/FR, builds bornés, nettoyage prudent |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| rg mobile / gateway / http / android | crates, scripts, docs et manifestes | Aucun accès mobile ni module Android |
| rg HumanPrincipal / human_authority_guard / control_client_actor | daemon.rs, protocol.rs | Deux gardes humaines existantes, responsabilités différentes |
| rg reattest / loopback / validate_endpoint | t3code_mcp.rs, native_delegation.rs | Validation locale du runtime et du port |
| rg handle_idempotent_send / human_from_bare_client | daemon.rs | Envoi humain natif réutilisable sans identité d'agent |
| rg human_cancel / resolve / lookup | delegation.rs, human_inbox.rs, idempotency.rs | Puits existants à réutiliser |
| lecture federation-services.md | docs | Un maître, pas de réplication |
| inventaire PATH et emplacements Android | Cartae | Outils Android absents des emplacements usuels |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Principal humain | creer nouveau | L'appareil est une autorité d'accès distincte, les puits métier sont réutilisés | 2026-10-24 |
| Maître permanent | approfondir avant bascule | La portabilité des attestations est une tâche bloquante d'architecture, jamais une validation relâchée | 2026-10-24 |
| HTTP et Android | creer nouveau | Aucune interface de ce type n'existe ; bibliothèque et versions à justifier avant ajout | 2026-10-24 |
| Réconciliation | reutiliser | Le reçu natif reste l'autorité, sans seconde file d'exécution | 2026-10-24 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Contre-revue

Deux demandes ont été refusées avant lancement ; aucune revue n'est acquise.
- bridget-mobile177-plan-review-20261024-01 : auto_admission_invalid. Lecture de native_delegation.rs:1915 : agent_type est interdit en V2. Aucun reçu réservé avant ce refus.
- bridget-mobile177-plan-review-20261024-02 : expected_base_commit_mismatch. La branche source main a avancé concurremment jusqu'à 949d0c781ace70988aeddabf5596a423ae3f5680 ; le worktree de la session reste sur la base figée be00ace5.
La délégation est arrêtée à ce refus. Aucun autre moteur ni lancement de remplacement.

## Arbitrage d’implémentation T003

CREER dépendances HTTP Axum/Tokio : aucun serveur HTTP dans les manifestes ; le nouveau besoin exige bornes et autorisation et justifie une bibliothèque reconnue, au lieu d’un parseur maison. Réutiliser serde/rusqlite/uuid/sha2 et le client Unix ; aucun second client HTTP sortant. Les fonctionnalités et versions seront verrouillées dans Cargo.toml/Cargo.lock.

## Arbitrage d’implémentation T006

- REUTILISER la connexion Unix négociée pour l’administration sur le maître ; ne jamais ouvrir le magasin d’autorisation d’un client fédéré comme autorité.
- CREER une capacité MobileAdministrationV1 et un contrat strict limité à configurer, associer, lister et révoquer. Justification : ControlStateV1 contrôle le moteur, pas les appareils ; aucune identité d’agent n’est réutilisée.
- CREER la dépendance qrcode 0.14.1 : recherche par nom et responsabilité négative ; encoder le QR demandé est une responsabilité réelle. Rendu sans bibliothèque d’image ni outil externe.

## Arbitrage T007

CREER la frontière Axum avec un adaptateur Listener limité : absence de serveur HTTP ou de borne TCP existante vérifiée ; ce wrapper porte les limites de connexion avant décodage et la durée totale. Réutiliser DaemonConnection, le magasin et le protocole. Une connexion en attente ne peut pas ouvrir une file illimitée de travaux bloquants. CREER la vérification du maître local avant service : une socket transférée ne prouve pas la résidence de l’autorité.

## Arbitrage Android T018–T021

CREER le module Android, absent du dépôt et des manifestes ; réutiliser le protocole mobile, les clés et reçus. Kotlin/Compose/Activity et coroutines portent le cycle de vie natif. REUTILISER HttpsURLConnection, SQLite et Android Keystore, sans nouveau client HTTP ni chiffrement maison. CREER la dépendance ZXing embarquée pour scanner le QR dans cet APK ; aucun scanner ni service Google séparé requis pour l’association. Les versions AGP 8.9.2, Gradle 8.11.1 et JDK17 suivent la matrice officielle API35.

## Arbitrage d'envoi natif T010

REUTILISER handle_idempotent_send et handle_idempotency_lookup, reçus et commandes différées existants. CREER une opération mobile fermée négociée sur Unix : le contrôle HTTP seul peut devenir périmé si un agent change de projet avant la remise. Le daemon doit relire les droits du téléphone puis vérifier l'identité, la machine et le projet sous son verrou, immédiatement avant le puits natif. Aucun relais générique de trames, aucun label humain existant usurpé. Les clés de reçu sont séparées par appareil, pas par connexion TCP. Le magasin ne valide que son répertoire et ses trois fichiers SQLite connus, sans parcourir tout le namespace à chaque authentification ; le bootstrap de namespace existant garde son préflight global.

## Arbitrage journal T011

REUTILISER lineage_journal_page et la lecture Worker JournalPage existante. Extraire seulement la lecture froide de journal de la garde de conversation de bureau : deux autorités distinctes appellent le même lecteur, sans fabriquer de contexte de bureau. Les gardes d'origine restent autour du lecteur sur le parcours existant. Pour une identité mobile, recontrôler appareil et identité après I/O. Tant que l'historique natif n'a pas de partition de projet prouvée, une autorisation partielle ne permet pas la lecture d'un historique global ; le refus est explicite et n'élargit pas les droits.

## Liaison des données Android à leur origine

REUTILISER LocalStore et les reçus natifs. La référence relue, composerDraftStore.ts:1482 et :2600, associe les brouillons à environnement et fil. Bridget lie ses brouillons à origine HTTPS, appareil et agent ; un changement de clé du même appareil ne change pas cette identité. Les commandes et renouvellements figent aussi leur origine et refusent une nouvelle association avant toute I/O réseau. Aucun second moteur de reçu ni table métier.

## Suivi HTTP T007/T011

REUTILISER attach::lineage_journal_follow (attach.rs:151) après une première page autorisée via le daemon. Le suivi natif reconstruit les séquences et lacunes, puis utilise le flux wrapper ; aucune boucle de requêtes de journal distante n’est ajoutée. La référence relue borne les éléments et leur taille dans ThreadLiveEventCoalescer, et sépare abonnement de fil et instantané. CREER seulement la borne de souscriptions HTTP par appareil et le repère de reprise lié à l’appareil/agent : nouvelle frontière humaine, distincte du contexte de conversation de bureau. Chaque flux conserve le permis global et son arrêt après 25 s ; le contrôle d’autorisation ne lit que le magasin d’appareils, jamais le journal métier, à chaque contrôle. Aucun broker ni ledger supplémentaire.

## Tâches et annulation T015

REUTILISER lineage_page_versioned, lineage_show_versioned, human_cancel et la table de reçus native. La référence relue exige un fil existant et émet une demande d’interruption corrélée à commandId ; elle ne possède pas cet arbre de tâches distant. Bridget conserve son reçu d’annulation et sa réconciliation, sans fabriquer de parent fournisseur. La lecture mobile de l’arbre exige TaskRead et une autorisation complète tant que la partition historique par projet n’est pas attestée. Les portées partielles reçoivent un refus fermé explicite. CREER le seul lecteur de reçu d’annulation par clé préfixée appareil/opération : cette lecture manque, et doit vérifier l’enveloppe entre racines. Un index non unique sur request_id évite un scan. Pas de nouvelle table métier ni duplication d’état. Reprise et suspension ne sont pas promises par une capacité fournisseur absente : l’interface distingue une instruction au parent d’une action native confirmée.

## Artefacts T012

REUTILISER ArtifactService, ArtifactStore et le magasin SHA-256. La référence relue AttachmentUpload.ts et attachmentPaths.ts borne les claims, octets et chemins avant publication. Bridget réutilise son stockage canonique et son reçu, sans URL de disque ni upload externe. CREER seulement la projection et la garde du principal mobile dans mobile_artifacts.rs. Les octets sont encodés par le codec base64 existant ; 8 Mio reste sous la trame native de 16 Mio. Deux uploads HTTP simultanés, autorisation avant corps puis avant publication. CREER un lecteur de reçu par clé appareil et un index sur la table native : pas de table ni état métier parallèle. Etendre le lecteur de références par un curseur complet (date, référence) pour ne pas perdre les fichiers publiés à la même seconde. Les types actifs (HTML, exécutables) ne sont pas servis ; PDF, PNG, JPEG et texte UTF-8 seulement, avec nom sans chemin et signature vérifiée à l’entrée.

## Adaptation des tâches T015

CREER mobile_tasks.rs pour isoler la garde du principal humain mobile des gardes de fil de bureau. REUTILISER la projection lineage V4 et human_cancel ; rendre le prédicat de disponibilité natif visible au module frère. La lecture d’annulation vérifie son reçu natif par clé d’appareil, et refuse une autre racine/tâche sous cette clé. Aucun état ni reçu secondaire. Les reprises et pauses sont des instructions au parent, explicitement différentes d’une continuation fournisseur ou d’une pause confirmée. La référence relue exige aussi un fil existant pour interrompre ; aucun droit fournisseur de parent n’est produit.

Lecture durable T012 : REUTILISER le lecteur de références, avec une requête contrainte sur l’identité de conversation native de l’agent. Une autorisation complète de fédération permet les fichiers historiques, même si l’agent n’est plus connecté ; l’identité de projet vient de la version immuable. Les portées partielles restent refusées. Les publications exigent toujours la route courante et sa portée, avant et après ingestion.

## Questions T016

REUTILISER la boîte humaine, resolve, les options offertes, la décision et son ACK producteur. La référence relue decider.ts:1192 et ProviderService.ts:1676 vérifie le fil existant et corrèle la réponse au requestId ; Bridget garde aussi l’item humain et ne transmet aucun droit de fournisseur inventé. CREER mobile_questions.rs : garde d’appareil complète TaskRead/TaskRespond, projection bornée des faits155 et du texte cité comme non fiable, puis choix natif. L’acteur mobile fige appareil/opération/commande ; un lecteur indexé de la décision native détecte le rejeu ou un autre item/choix. Aucun nouveau reçu ou stockage métier. La pagination réutilise SELECT_ITEM et row_to_frame, avec un repère complet date/id. L’acquittement reste réservé au producteur ; un choix enregistré n’est jamais annoncé comme appliqué avant son ACK.

Fichiers Android : REUTILISER la clé Android Keystore, AES-GCM et la file LocalStore existantes. CREER seulement le fichier privé chiffré et son écriture AtomicFile Android : la limite de ligne SQLite de 128 Kio ne convient pas aux 8 Mio autorisés. L’enveloppe reste dans la même file ; origine/appareil/commande servent de données authentifiées. Aucun nouveau gestionnaire de chiffrement ni bibliothèque externe. Le budget tient compte aussi des fichiers sans ligne après une panne ; aucun fichier reçu par un chemin libre.

Récupération d’association : la référence relue ConnectionsSettings.tsx:2390 et client-runtime/state/connections.ts:103 enlève explicitement une connexion enregistrée via le catalogue, sans action automatique de modèle. Bridget doit aussi distinguer l’oubli local d’une clé et sa révocation serveur. REUTILISER LocalStore, en retirant seulement session/pairing/renewal après confirmation ; garder commandes, fichiers et preuves, liés à leur origine. Cette récupération est nécessaire pour une autorisation déjà révoquée ou une association expirée, qui ne peut plus s’auto-révoquer par réseau. Aucun accès ancien n’est envoyé avec une nouvelle association.

Reprise de processus Android : REUTILISER LocalStore et les tests ProtectedStoreTest. CREER un seul test instrumenté à deux étapes, piloté par adb, car aucun test existant ne garde sa fixture entre deux processus. Son alias, sa base et ses fichiers sont distincts des données de l’application. L’arrêt est limité au paquet Bridget ; aucune mission ni autre application n’est touchée.

## Notifications — décision avant implémentation

Recherche fcm/firebase/notifications dans le module mobile, le manifest Android et les dépendances : aucun canal mobile existant. Réutiliser le magasin privé des appareils, minreq HTTPS et les mutations natives (guichet humain, projection de lignée, fin de tour) ; aucun balayage de tâches ni nouvelle file métier. Créer mobile/notifications.rs pour un signal coalescé et un expéditeur FCM séparé de la boucle du daemon. La file éphémère contient un seul signal, sans contenu.

Le SDK Firebase Messaging est nécessaire pour le transport Android en arrière-plan ; aucun Analytics, Firestore ou service payant. ring 0.17.14 est déjà dans Cargo.lock via TLS et sera utilisé directement pour la signature OAuth RS256. base64 0.22.1 évite une implémentation locale du codec JWT/PEM. La configuration de compte serveur est un fichier privé hors Git. Le téléphone ne reçoit que les identifiants publics de l’application Firebase. La référence consultée McpSessionRegistry de l’autre produit ne couvre pas FCM ; aucun mécanisme équivalent n’est à réutiliser pour ce canal.

## Relais d’origine — réemploi avant implémentation

Le réattesteur existant valide le runtime loopback sur sa machine. Les preuves ParentProof/ParentSourceProof utilisent le worker authentifié, mais exigent une commande déjà relâchée ; elles ne couvrent pas un parent de bureau avant sa première Task. Réutiliser le même canal worker, l’époque, la négociation et la corrélation ; ajouter une enveloppe de lecture d’origine distincte, sans fabriquer de Task ou d’agent. Les seules opérations sont attestation fournisseur existante et vérification native des sources du parent. Aucun endpoint ni chemin de shell libre. Les lectures McpSessionRegistry/McpHttpServer dans la référence locale confirment la liaison stricte du jeton à l’instance et au fil ; elles ne proposent pas de relais multi-machine équivalent. Une origine absente doit être refusée, jamais remplacée par le disque du coordinateur.

## Préparation depuis une origine distante — décision T026

Réutiliser intégralement project_preparation::prepare et prepare-project.py : sélection du mapping, verrou, inspection SSH, bundle différentiel, import et reçus restent dans leur propriétaire existant. Le worker d'origine utilise son propre chemin de registre, jamais un chemin donné dans la requête. Ajouter une opération fermée de préparation au canal d'origine pour transporter parent, source et machine cible ; aucune commande ou script libre. Elle contrôle les droits et la base avant et après et ne lance aucun fournisseur. Le coordinateur relit son registre publié avant les sondes. Le cas n'existe pas dans les mécanismes locaux de la référence relus ; le détour provient de la séparation des disques et autorités de machine.

Réutiliser stop_owned_child pour borner le helper et ses enfants détenus. Un délai inconnu conserve les reçus et ne provoque pas de rejeu métier. Les lectures d'origine et la préparation ont des permis séparés ; une préparation lente ne doit pas bloquer l'attestation ou Release. Réutiliser le worker présent sur le coordinateur pour les sources étrangères ; aucun moteur local bis.

## Canal de mise à jour Android T035

La référence relue DesktopUpdates.ts:394 et ElectronUpdater.ts:126 sépare vérification, téléchargement et installation ; une erreur de vérification ne prouve pas une version courante. Réutiliser ce principe et le canal de releases GitHub existant. Le client mobile lit uniquement les métadonnées publiques bornées de ce dépôt ; aucune clé d'appareil, connexion associée ou jeton Firebase n'entre dans ce client. Créer le lecteur Android fermé MobileRelease, absent du module. Le téléchargement et la confirmation d'installation sont confiés au navigateur et à l'installateur Android, qui vérifie la signature. Aucun moteur de mise à jour de fichiers ni permission d'installation supplémentaire. La version Android suit la version du workspace ; la signature de production et sa sauvegarde restent privées.

## Contre-revue T026 — corrections ciblées

La référence locale GitVcsDriverCore.ts:839-854 borne une opération Git par Effect.timeoutOption ; createWorktree:2881 transmet un délai unique. Elle ne couvre ni origine distante ni préparation multi-machine. Réutiliser la même règle de durée au niveau du passage complet : 595 s partagées entre machines et transmises au helper existant, avec 5 s de marge sous le plafond worker de 600 s. Un lien absent donne project_preparation_link_missing, jamais un succès. La liaison de machine est relue après préparation. Ces corrections ne créent ni service ni table.

## Raccordement au maître serveur T027 — décision avant code

REUTILISER federation-ssh.sh, son runner autonome, sa configuration inerte, ses reçus et ses services. Le transfert inverse existant ne crée pas une socket cliente sur le Mac quand le maître est distant. L’essai réel de direct-streamlocal a été refusé par PermitOpen. Ne pas ouvrir PermitOpen any : ajouter une direction to-master qui utilise socat Unix/EXEC et SSH stdin/stdout, sans TCP ni nouvelle autorité. La documentation socat confirme fork, max-children et le transport stdin/stdout ; les tests doivent vérifier les chemins avec espaces, les sockets privées et la fermeture. Le même contrôle de socket stale sera réutilisé localement et à distance. Aucun nouveau superviseur, fichier de clé, protocole métier ou table. La référence locale de gestion des connexions ne couvre pas ce transport de daemon distant ; sa séparation connexion/runtime reste appliquée.

Périodes silencieuses US7 : la recherche quiet/silence dans le module Android ne trouve pas de réglage. Réutiliser le magasin chiffré et la liaison appareil/origine pour un horaire local séparé du consentement push. Le service ignore l’affichage pendant ce créneau ; les messages restent consultables. Aucun minuteur, permission système, requête serveur ou état métier ajouté.

## Revue sécurité HTTP — séparation des budgets

La référence locale McpHttpServer.ts:68-89 résout le jeton avant le traitement ; elle ne possède pas de quota bootstrap global. Réutiliser ce principe : un renouvellement obtient son sujet depuis le magasin existant, y compris le reçu de rotation valable, avant son quota par appareil. Une clé inconnue ne crée aucun compteur. L’association sans identité utilise l’adresse transmise par le proxy loopback, qui doit remplacer intégralement X-Bridget-Client-IP par remote_host. Deux index mémoire bornés remplacent le compteur global : 1024 adresses et 256 appareils, durée 60 s. Aucun stockage d’adresse durable, nouvelle table ou identité empruntée. Les relectures natives restent nécessaires à leur frontière ; la suggestion de transmettre seulement un Device depuis HTTP est rejetée, car elle supprimerait leur authentification indépendante.

## Journal d’un agent de bureau après migration

Constat concret : mobile_operations::agent_journal lit root/sessions pour un agent sans Task. Après déplacement du maître, ce journal reste sur la machine d’origine. Réutiliser read_journal_excerpt et le canal natif Attach/Subscribe, déjà capables de lire l’instantané du vrai wrapper distant. Aucune nouvelle opération de protocole, réplication de journal ou endpoint d’origine. Créer seulement la projection paginée bornée vers lineage_journal_result : 50 événements et 16 Kio, lacunes et reprise conservées. Le chemin local reste utilisé pour une origine locale ; une origine distante indisponible est refusée, jamais remplacée par un fichier homonyme. La référence locale ThreadLiveEventCoalescer.ts:194-197 conserve aussi la frontière synchronized après les événements et borne la mémoire. Ajouter la socket de configuration immuable à DaemonState évite une dépendance à l’environnement global et tout accès accidentel au maître réel depuis une fixture.

## Incident réel de projet distant, 24 octobre

Lecture courte de T3 : apps/server/src/project/T3ProjectFileLoader.ts charge le projet sur la machine qui possède son workspace ; apps/server/src/vcs/GitVcsDriverCore.ts transmet explicitement cwd au lecteur Git. Aucun mécanisme de projet fédéré équivalent dans ces fichiers. Bridget réutilise son resolve_communication_project existant sur le worker provisionné, avec le canal OriginProof déjà introduit dans ce lot. Il ne recrée pas une règle lexicale de projet et ne transforme pas le chemin Mac en chemin Linux. La garde humaine conserve le binding T3 et l'égalité du projet validé.

Pour le relais SSH duplex, les mêmes sources T3 ne couvrent pas socat/MSG_PEEK. Bridget conserve son runner natif et son contrat de flux SSH ; le correctif évite de consommer puis réinjecter le premier octet. La preuve macOS utilise deux échanges espacés, pas seulement un écho immédiat.

Intégration du main 4118ff2b : conservation des corrections concurrentes et de minreq 3 / serde-saphyr. minreq 3 retourne un statut HTTP u16 ; la frontière FCM existante i32 conserve son contrat par conversion sans perte. Référence T3 OpenCodeAdapter.test.ts, cas session_message 404 : conservation du statut fournisseur, pas de réinterprétation du code. T3 n’utilise pas ce client Rust et ne fournit pas d’équivalent FCM ; seule la règle de conservation du code est réutilisable.

Recette Pixel : T3 ChatView.tsx, scrollToEnd et bouton de retour au direct (4663+, 8008+), maintient le suivi de fin jusqu’à un geste de lecture antérieure. Le client Compose reprend ce comportement sans réutiliser son code web : en-tête compact, boutons secondaires dans un menu et saisie ancrée. Pour l’historique vide, le lecteur Attach natif de Bridget connaissait déjà SnapshotCaughtUp même sans through_seq ; le relais mobile perdait ce fait en testant seulement is_some. Un booléen interne non sérialisé conserve maintenant l’événement explicite. Le protocole et les limites restent inchangés. T3 ne possède pas ce relais Unix distant ; le lecteur natif existant est la référence appropriée.

### Reassociation and expired QR
The user reused the consumed first QR after forgetting local access. The server refused correctly; Android incorrectly showed a generic unconfirmed response. Dedicated EN/FR guidance now explains expiry, one-use pairing and the explicit new-association action. No pending request or saved work is discarded automatically. T3 ConnectionsSettings.tsx computes expiry from expiresAt and displays an expiration label; the temporary private QR page follows the same expiry principle without copying its implementation. Lost-response reconciliation is unchanged.

### Recent history, project labels and keyboard (physical feedback)
T3 ChatView.tsx retains follow-to-end until a real user scroll, and observes composer resize. Android uses its existing LazyListState with viewport-size changes and stable message keys. T3 ProjectIconPickerDialog uses a compact project icon; Bridget uses a local monogram and a short name, with host/parent/digest only for homonyms. Native host/root identities remain unchanged. Recent history reuses Attach Tail(1) to locate the end, then the existing bounded forward page. The optional tail flag defaults to false for previous native requests; workers need no new operation. Older messages are loaded explicitly without moving the live cursor.

### Concurrent deployment observation
The active core was replaced by the native-project fix while physical testing was ongoing. The r8 archive used by that build predates the empty-snapshot amendment, although the original r8 binary included it. This explains the restored stopped-follow symptom. The next candidate must include both the native-project correction d5af84f8 and the empty-snapshot amendment. No rollback to an older binary is permitted.

### Guidage et activité mobile (24 octobre)

Lecture préalable : runtime Codex sendTurn (turn/start et conservation du tour
actif), adapter Claude sendTurn (tour SDK déjà actif), actions du composeur et
contrats des outils du dépôt de référence local. Différence nécessaire : le
mobile utilise l’autorité humaine limitée au téléphone et Bridget conserve son
journal et ses reçus. Aucun appel direct de l’application Android à ce bureau.
Réemploi Bridget : SteerCurrent + steer-or-queue déjà gérés par Codex natif,
Claude et ACP ; ajout du même repli au connecteur de bureau. Sa liste de
fournisseurs reste prudente : Claude immédiat, autres fournisseurs en file.
Le connecteur publie des métadonnées d’outil depuis sa projection existante ;
aucun argument ni sortie brute copiés et aucun fait d’écriture supplémentaire.
Complexité : passage O(n) sur la file bornée à 64 ; outils O(items × profondeur
bornée à 64), même filtre de fil principal que les messages.
