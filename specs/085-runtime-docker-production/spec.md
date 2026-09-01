# Feature Specification: Runtime Docker de production et pilotage par projet

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 085-runtime-docker-production
Titre: Runtime Docker de production et pilotage par projet
Statut: Ready for implementation
Priorité: P1
Tâches: 0/39 (0%)

Résumé:
- Contexte: SPEC-066 et SPEC-067 ont créé le moteur, les politiques et les ressources, mais l'installation active ne charge aucune politique de runtime et l'image fournie reste une fixture de test.
- Objectif: rendre Docker sélectionnable, préparé, observable et réversible par projet avec une image Linux de production et des réglages fermés.
- Dépendances: SPEC-066, SPEC-067, SPEC-080, SPEC-084

Fichiers:
- spec.md: ✓
- tasks.md: ✓
- plan.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-085-runtime-docker-production`
**Created**: 2026-08-31
**Status**: Ready for implementation
**Priority**: P1
**Dependencies**: SPEC-066, SPEC-067, SPEC-080, SPEC-084

## Contexte

Le daemon possède déjà des liaisons Host/Docker, une politique de runtime, une projection d'état, un cycle `prepare/stop/remove/recreate/switch-backend`, des limites de ressources, une racine de fichiers en lecture seule et des montages par projet. Il possède aussi un catalogue de ressources approuvées pour extensions et secrets. Ce socle n'est pas utilisable comme produit installé: le service ne charge ni politique runtime ni catalogue de ressources, aucune image de développement de production n'est publiée, et l'interface n'offre pas un parcours complet d'activation Host vers Docker.

Cette spécification termine le produit existant. Elle n'introduit ni Kubernetes, ni Compose, ni Vault, ni daemon central, ni conteneur par agent.

## Objectifs

- Fournir une image Linux amd64 de développement versionnée, reproductible et référencée par digest.
- Charger les catalogues runtime et ressources au démarrage normal du daemon.
- Définir un backend par défaut au niveau du serveur, appliqué seulement aux nouveaux projets.
- Permettre un choix explicite et une activation Docker par projet.
- Exposer état, politique, opérations sûres et raisons de refus dans l'interface.
- Conserver le retour Host comme rollback explicite, sans fallback silencieux.
- Garder un conteneur partagé par projet pour tous ses agents.

## Hors périmètre

- Un conteneur par agent.
- Kubernetes, Docker Compose, gVisor, microVM ou orchestrateur externe.
- Une interface graphique, VNC, bureau Linux ou navigateur intégré.
- La copie de secrets dans l'image ou dans le dépôt.
- L'édition libre d'une image, d'une commande Docker, d'un chemin hôte ou d'une variable d'environnement depuis l'UI.
- La compilation d'un artefact pour une autre machine depuis le conteneur courant.
- Le déploiement automatique d'un binaire construit par un agent.
- Le dogfooding Bridget, traité par SPEC-086.

## User Story 1 - Configurer Docker au niveau du serveur - P1

Comme exploitant, je veux choisir Host ou Docker comme valeur par défaut des nouveaux projets et administrer un catalogue fermé afin d'activer progressivement le runtime.

### Scénarios d'acceptation

1. Une installation existante migre avec `Host` comme défaut et aucun projet existant n'est modifié.
2. Une installation peut sélectionner `Docker` comme défaut uniquement si Docker, l'image et la politique sont attestés.
3. Le service charge explicitement la politique runtime et le catalogue de ressources depuis des fichiers opérateur validés.
4. Une configuration absente ou invalide rend Docker indisponible avec une raison, sans fallback automatique lors d'une demande Docker.
5. La politique expose des identifiants fermés, jamais des arguments Docker libres.

## User Story 2 - Activer et piloter Docker par projet - P1

Comme opérateur, je veux choisir le backend d'un projet et piloter son environnement afin d'isoler les agents sans perdre la possibilité de revenir sur l'hôte.

### Scénarios d'acceptation

1. À la création/import, le backend proposé vient du serveur mais reste explicitement visible et modifiable.
2. Un projet Host existant peut être activé en Docker par une opération typée qui lie une politique et prépare l'environnement.
3. L'activation échoue sans changer la liaison si la préparation échoue.
4. Arrêt, suppression, recréation et retour Host sont refusés lorsqu'un agent du projet est actif.
5. Le retour Host retire l'environnement selon le contrat et ne supprime jamais le dépôt ou les worktrees.

## User Story 3 - Disposer d'un environnement de développement utile - P1

Comme agent d'un projet Docker, je veux retrouver Git et les chaînes de développement approuvées afin de lire, modifier, tester et construire le projet dans le conteneur partagé.

### Scénarios d'acceptation

1. L'image contient un socle versionné: shell, Git, certificats, curl, jq, ripgrep, outils de compilation, Rust/Cargo, Node/pnpm et Python.
2. Les clients fournisseurs approuvés sont installés à versions maîtrisées ou montés via le catalogue SPEC-067.
3. Les identifiants et secrets ne sont jamais présents dans l'image et sont injectés uniquement par ressources approuvées.
4. Le dépôt et ses worktrees sont montés aux mêmes chemins absolus que sur l'hôte afin de préserver les métadonnées Git liées.
5. Les caches et états persistants vivent sous la racine d'état du projet, pas dans l'image.

## User Story 4 - Comprendre l'état et les refus - P1

Comme opérateur, je veux connaître backend, disponibilité, politique, état et blocage afin de distinguer une configuration manquante d'un conteneur arrêté ou occupé.

### Scénarios d'acceptation

1. La fiche projet distingue Host, Docker absent, préparation, prêt, arrêté, dégradé et politique indisponible.
2. Les opérations possibles sont dérivées de l'état confirmé, jamais anticipées optimistement.
3. Une erreur Docker est convertie en raison fermée et corrélée, sans exposer commande complète, secret ou identifiant de conteneur.
4. Après redémarrage du daemon, l'état réel est réconcilié avec Docker avant d'être présenté.

## Exigences fonctionnelles

- FR-08501: Le daemon DOIT conserver le modèle d'un conteneur partagé par projet.
- FR-08502: Une image de production Linux amd64 DOIT être construite depuis une base épinglée par digest.
- FR-08503: L'image et chaque client fournisseur installé DOIVENT avoir une version explicite et une provenance documentée.
- FR-08504: Aucun secret, jeton, clé SSH privée ou donnée opérateur NE DOIT être incorporé dans l'image.
- FR-08505: L'image DOIT fournir Bash, Git, CA, curl, jq, ripgrep, build-essential, Rust/Cargo, Node/pnpm et Python.
- FR-08506: Les outils supplémentaires DOIVENT provenir d'une nouvelle version d'image ou du catalogue approuvé SPEC-067, jamais d'un téléchargement implicite de l'agent.
- FR-08507: Le service installé DOIT charger une politique racine, une politique runtime et un catalogue de ressources par chemins explicites.
- FR-08508: Le centre de contrôle DOIT exposer `execution.default_backend` avec les valeurs fermées `host` et `docker`.
- FR-08509: Une migration d'installation DOIT conserver `host` par défaut.
- FR-08510: Modifier le défaut NE DOIT changer aucun projet déjà lié.
- FR-08511: La création/import DOIT afficher backend et politique résolue avant confirmation.
- FR-08512: Un projet Host existant DOIT pouvoir demander une activation Docker atomique avec politique explicite.
- FR-08513: L'activation Host vers Docker DOIT préparer l'environnement avant de publier la nouvelle liaison active ou revenir entièrement à Host en cas d'échec.
- FR-08514: Aucune activation Docker NE DOIT accepter d'image, commande, chemin, montage ou argument libre fourni par le client.
- FR-08515: `prepare`, `status`, `stop`, `remove`, `recreate`, `activate_docker` et `switch_to_host` DOIVENT être des opérations typées, versionnées et idempotentes.
- FR-08516: Les opérations destructrices du runtime DOIVENT être refusées si un agent du projet est actif.
- FR-08517: Retirer ou recréer un conteneur NE DOIT jamais supprimer le dépôt, les worktrees ou leur historique Git.
- FR-08518: Un échec Docker demandé explicitement NE DOIT jamais lancer les agents sur Host par fallback silencieux.
- FR-08519: Le dépôt principal, le répertoire Git commun et les worktrees liés DOIVENT être visibles aux mêmes chemins absolus dans le conteneur.
- FR-08520: Les caches, HOME et états fournisseurs DOIVENT être persistés sous la racine d'état propre au projet.
- FR-08521: Le conteneur DOIT conserver rootfs en lecture seule, utilisateur non root, `cap-drop=ALL`, `no-new-privileges`, limites CPU/mémoire/PIDs et tmpfs bornés.
- FR-08522: Les accès réseau DOIVENT suivre une politique fermée du catalogue; l'état initial peut utiliser le bridge Docker existant sans exposer de port entrant.
- FR-08523: Secrets et extensions DOIVENT réutiliser le catalogue et les approbations SPEC-067.
- FR-08524: L'UI DOIT exposer backend, état, politique, version/digest, dernière raison et actions autorisées.
- FR-08525: L'UI NE DOIT pas annoncer le succès avant l'état confirmé par le daemon.
- FR-08526: Le daemon DOIT réconcilier sa liaison persistée avec l'état Docker après redémarrage.
- FR-08527: Les erreurs DOIVENT être fermées, corrélées et purgées des commandes, chemins non nécessaires et secrets.
- FR-08528: Aucun Docker Compose, Kubernetes, GUI, VNC ou navigateur NE DOIT être ajouté.
- FR-08529: Une architecture hôte différente de Linux amd64 DOIT être déclarée non supportée par cette version, sans tentative d'émulation implicite.
- FR-08530: Les données persistantes DOIVENT utiliser une racine d'état hôte administrée et des bind mounts; aucun volume Docker nommé opaque ne doit devenir une seconde source de vérité.
- FR-08531: L'installation DOIT vérifier l'accès du compte de service au daemon Docker et NE DOIT jamais modifier automatiquement groupes, permissions ou socket.
- FR-08532: L'image de base DOIT rester neutre vis-à-vis des fournisseurs; les clients Codex, Claude, Cursor/ACP ou Gemini DOIVENT être des composants versionnés approuvés par politique, sans credential embarqué.

## Exigences non fonctionnelles

- NFR-08501: Le statut d'un projet doit répondre en moins de 500 ms hors indisponibilité Docker.
- NFR-08502: Une commande rejouée avec le même identifiant et la même enveloppe doit produire le même reçu.
- NFR-08503: Le build d'image doit être reproductible à versions et digests identiques.
- NFR-08504: Les journaux ne doivent contenir aucun secret ni commande Docker complète.
- NFR-08505: Le rollback Host doit rester possible sans reconstruction du dépôt.

## Entités clés

- **Réglage serveur d'exécution**: backend par défaut et génération de configuration.
- **Politique runtime**: image, digest, identité Unix, ressources, réseau et exécutables approuvés.
- **Liaison projet**: backend Host ou Docker, génération et politique résolue.
- **Environnement projet**: conteneur partagé, état, epoch et raison fermée.
- **Catalogue de ressources**: extensions et secrets approuvés issus de SPEC-067.

## Cas limites

- Docker devient indisponible entre prévisualisation et activation.
- L'image par tag ne correspond plus au digest attendu.
- Un agent démarre pendant une demande d'arrêt.
- Le daemon redémarre pendant la préparation.
- Un worktree est ajouté après la création du conteneur.
- Une politique est retirée alors qu'un projet y est lié.
- Le chemin Git commun se trouve hors du checkout visible.

## Critères de succès

- SC-08501: Une installation neuve configurée peut créer un projet Docker et lancer deux agents successifs dans le même conteneur de projet.
- SC-08502: Une installation existante reste en Host tant qu'aucune décision explicite n'est prise.
- SC-08503: Cent pour cent des échecs de préparation laissent la liaison précédente intacte et ne déclenchent aucun agent Host.
- SC-08504: Cent pour cent des opérations stop/remove/recreate/switch sont refusées avec un agent actif.
- SC-08505: Les tests prouvent l'absence de secrets dans image, inspect, logs et projections UI.
- SC-08506: Un dépôt multi-worktree reste utilisable depuis Host et Docker grâce aux chemins absolus identiques.

## Décisions actées

- Un conteneur est partagé par tous les agents d'un projet.
- L'image est headless, Linux amd64, sans interface graphique.
- Host reste le défaut de migration et le rollback explicite.
- Les builds destinés à une autre machine sont refaits depuis Git sur cette machine cible.
