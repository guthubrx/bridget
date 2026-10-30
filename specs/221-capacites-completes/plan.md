# Plan221 — capacités complètes

Statut : implémentation en cours ; programme global non terminé, L1a/L1b livrés ; compléments de guidage et C18 chargés, L1c global reste ouvert. Responsable unique : bridge1016.

## Contexte technique et invariants

Socle Rust2024, trois crates, SQLite, canaux Unix négociés, workers provisionnés, service HTTP humain axum/tokio et Android Kotlin. État initial : base479f2c00 et services220. Après L1a : source86a88e3f, servicesLinux221-drain chargés ; clients et anciens superviseurs sous usage distingués dans son reçu. Réutiliser ces autorités. Ne pas introduire un second orchestrateur, store de tâches, moteur d’admission, gestionnaire de permissions, installateur ou transport de fédération. Les clients graphiques ne détiennent pas l’autorité d’exécution.

Périmètre :11histoires/50critères. Livraison globale jamais déclarée depuis une seule tranche. Chaque lot buildable conserve les cas non terminés dans tasks.md. La présente invocation autorise tous les lots. Aucun GO intermédiaire pour les actions déjà couvertes.

## Architecture retenue

1. Comptes/modèles/options : étendre registry, provider_catalogue220 et les adaptateurs de transport. Le worker est seul observateur de sa machine. Le maître publie les faits corrélés et périmés. Ajouter un parcours humain d’authentification officielle à partir de la configuration isolée choisie, sans copier de secrets. Les différences de protocole restent dans les adaptateurs ; aucune lecture générale des fichiers de tokens.
2. Installation et versions : étendre install.sh, federate-ssh.sh, client_compatibility et updates. Produire un inventaire disque/processus/liaison lisible depuis les clients et un résultat par étape. Ne pas redémarrer un hôte applicatif tiers à l’aveugle.
3. Projets : étendre project_preparation, workspace_git/compact et remote_worker. Pour une revue d’objet Git immuable, ne pas avancer HEAD partagé ni créer un worktree juste pour lire ; borner la lecture à la base admise et afficher son mode. Un fournisseur qui ne peut travailler depuis ce mode reçoit un refus précis ; il ne sera pas déclaré couvert avant un scénario réel.
4. Conversations : réutiliser execution_store, handoff, reprise, ManagedSession et les ports natifs Resume/Fork. Ajouter les états/manifests nécessaires pour switch, import et checkpoints avec preview/CAS, sans injecter une fausse autorité dans un ancien journal.
5. Arrêt et erreurs : corriger remote_drain et sa barrière de propriétaires avant extension du cycle de vie ; enrichir la projection commune des refus plutôt que créer un catalogue d’erreurs parallèle.
6. Messages : étendre QueueEditAction, idempotency/send_delivery, user_input et artifact_service. Références de secret : valeurs uniquement dans la mémoire privée du service humain, durée courte, consommation atomique et liée à une opération ; seules références/tombstones dans les reçus. Aucun secret dans le contexte du modèle.
7. Interface : étendre la passerelle humaine existante mobile/http et mobile_operations pour partager mêmes commandes, reçus, authentification et contrôle des droits. Ajouter un client web responsive installable sur bureau/iOS/Windows, séparé des autorités. Les contrats humains doivent couvrir création de mission, organisation, diff/PR, terminal, navigateur et appareils. Un raccourci web ne sera pas présenté comme un hôte Windows d’exécution native.
8. Outils interactifs : modules d’exécution bornés pour navigateur, appareils, terminal et PR, enregistrés auprès du worker existant. Ils possèdent les sous-processus/sessions, gardent les droits, le propriétaire de contrôle et les preuves. Réutiliser CDP, outils officiels de simulateur/émulateur, PTY et Git/CLI officiel quand disponible ; aucun shell libre exposé comme lecture. Les nouveaux protocoles sont négociés et refusés explicitement par un ancien pair.
9. Automatisation : étendre wake_schedules et reçus pour tâches/webhooks et abonnements PR. Signature, fenêtre temporelle, nonce, limite de corps, débit et autorité d’action figée. Les données externes ne deviennent jamais une autorisation.
10. Usage/prérequis : étendre provider_limits, événements de transport, host_resources, project_preparation et rapports. Prix configurés distincts des mesures. Prérequis lus dans le commit admis, commandes de sonde fermées/versionnées, aucun script projet arbitraire pour vérifier une version.
11. Moteurs/Windows : étendre ManagedSession/ACP avant tout nouvel adaptateur. Client Windows commun aux interfaces web/bureau/iOS, pilotant exclusivement les serveurs Mac/Linux selon le choix utilisateur. Aucun port du moteur d’exécutionWindows.

## Lots ordonnés

L0: preuve de référence et audit d’existant pour50catégories ; L1: arrêt/diagnostic/reprise et baseGit ; L2: comptes/options/prérequis/miseàjour ; L3: conversations/coordination/secrets ; L4: interface humaine et lancement mobile ; L5: outils navigateur/appareils/PR/terminal ; L6: automation/usage ; L7: moteurs et plateformes ; L8: campagne globale, adversaire, convergence, audit et distribution. Tous appartiennent au même périmètre autorisé.

Une exigence non prouvée par220reste une tâche, pas une remise à zéro de220. Les cas déjà couverts reçoivent les tests existants comme preuve après contrôle de leur pertinence. Ne pas dupliquer les suites ni additionner les répétitions.

## Éléments à auditer avant tasks

| Item | Intention | Issue prévue |
| --- | --- | --- |
| Observation de compte/modèle | étendre faits220et options | Réutiliser provider_catalogue220/transport |
| Authentification isolée | parcours officiel par connexion | Étendre registry/profiles/clients humains |
| Préparation guidée et miseàjour | étapes reprenables | Étendre install/federate/updates |
| Inventaire versions chargées | connaître composant ancien | Étendre build_info/communication observations |
| Préparation/projet/prérequis | base,placement,outils | Étendre workspace/project_preparation |
| Lecture de commit immuable | revue sans conflit shared | Étendre Git contrôlé et contrat worker |
| Switch/fork/import/checkpoint | contexte et fichiers sûrs | Étendre execution_store/handoff/managed_session |
| Drainage | arrêt réel même descendant UIDdifférent | Corriger managed_process/wrapper |
| Diagnostic commun | action autonome | Étendre guidance/projections existantes |
| File éditable | texte corrigeable avant remise | Étendre QueueEdit/store transactionnel |
| Questions | réponses corrélées | Réutiliser user_input/human_inbox |
| Secret éphémère | saisie hors modèle | Extension humaine dédiée ; audit de la primitive prévue158B |
| Artefacts | formats,preview,isolation | Étendre artifact_service/mobile |
| Client web humain | expérience commune | Nouveau client, même passerelle et droits |
| Navigateur | partage contrôlé de session | Nouveau module worker sous protocole négocié |
| Appareils | session contrôlée | Nouveau module worker avec outils officiels |
| PR/diff/abonnement | revue et événement | Étendre Git/store/wake sans storePRparallèle |
| Terminal | PTY détenu/reconnectable | Réutiliser attach/pty, nouveau port humain contrôlé |
| Organisation des travaux | recherche/pin/archive | Étendre threads/store/directory |
| Webhook/tâches | déclencheurs bornés | Étendre wake_schedules/receipts |
| Usage/coût | faits et estimation | Étendre provider_limits/events |
| Mobile création/horsligne | mission humaine | Étendre mobile/Android, même admission |
| Moteurs | adaptation officielle | Étendre ACP/ManagedSession |
| Windows | interface complète vers Mac/Linux | Réutiliser le même client web installable, aucun moteur Windows |

## Constitution et maintenance future

Worktree isolé, noms génériques Bridget dans parcours produit, droits revalidés avant effet, aucune mutation de compte autorisée par une seule lecture de catalogue. Chaque nouveau module doit porter une règle d’exécution/sécurité/compatibilité réelle et son test ; pas de simple wrapper passthrough. Aucune nouvelle dépendance n’est admise dans ce plan sans démontrer l’échec de l’existant. Une interface sans boucle réelle utilisateur ne vaut pas livraison. Les noms des clients tiers restent dans contrats d’adaptateur et documents de comparaison.

## Gates

Avant tasks : reuse-audit PASS et contre-revue adverse demandée ; aucun doublon évident non arbitré. Avant code : Analyze exécuté, checklist qualité fermée, contrats testables. Avant livraison : tout le lot modifié testé, self-review et adversaire, Converge ; la livraison globale exige50/50preuves et toutes les tâches closes. Git puis runtime sont prouvés séparément. Chaque plateforme non testable garde son blocage sans empêcher le travail indépendant.


## Amendements retenus après la première contre-revue

La matrice acceptance.md donne une entrée, une sortie et une preuve distinctes pour50catégories. Les décisionsD1–D10de research.md prévalent sur les formulations initiales trop larges. C28réutilise MobileOperation::MessageQueueChange et mobile_queue::replace ; QueueEditAction ne reçoit pas une édition redondante. Les ports exacts de bifurcation sont CodexThreadBootstrap::Fork et le dispatch thread/fork dans codex_app_server.rs:3186–3260 ; ManagedSession expose la frontière commune et la capacité ProviderOperation::Fork, pas une méthode fn fork.

C25est fondé sur le reçu218et deux défauts relus à479f2c00 : checked_pidfd refuse un enfant adopté d’UIDdifférent avant waitid ; RemoteDrainGuard::drop court-circuite la récolte dès que signal_pass échoue. Le test préalable doit reproduire ce cas sur sous-processus privés et garantir la barrière après destruction des Child. Pas d’élargissement du droit de signal ni de SIGKILL.

Chaque lotL1–L7a sa porte de livraison : tâches du lot closes et tests ciblés, convergence du lot contre toutes les exigences qu’il touche, contre-revue/audit utiles, sources intégrées/poussées, binaires distribués et processus vérifiés, clients anciens nommés, nettoyage borné. Le programme global reste InProgresset conserve toutes les tâches des autres lots. Les gates de la skill s’appliquent à chaque unité de livraison cohérente ; aucune tranche ne vaut remplacement complet. Cette organisation applique la consigne permanente d’activation immédiate sans accumuler des corrections inutilisées.

Aucun nouveau store de tâches. AbonnementsPRetwebhooks ajoutent leurs définitions et occurrences aux magasins wake/idempotency existants, avec schéma de source versionné ; les types d’événements et la signature restent distincts. Les modules sessionsoutils sont justifiés par leur contrôle de possession, leurs bornes et leur nettoyage, pas par un emballage d’outil externe. Clientwebintégré au binairepasserelle, mêmes hash/version et règles de bascule. Les recettesWindows/iOSmanquantes ne deviennent pas des restrictions de portée ni un arrêt des autres lots.

Première unité prévue : L1a/C25 drainage, puis projection/refus liés ; sa livraison autonome requiert un sous-ensemble de tâches explicitement figé et toutes ses preuves. Si le gate anti-doublon du plan n’est pasPASS, aucune unité n’est implémentée.

Clarification prioritaire : la décision utilisateur D11remplace les hypothèses de paquets natifs et de backendWindows. Clientcommun webinstallable pour les quatre plateformes ; Androidexistant maintenu par le même contrat. Aucun hôteWindows. Les interfaces ne deviennent pas des autorités. Le client et son contrat de compatibilité sont sous la responsabilité du mainteneur Bridget, pas d’un service externe.
