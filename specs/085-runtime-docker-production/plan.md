# Plan d'implémentation - SPEC-085 Runtime Docker de production

## Contexte technique confirmé

| Élément | Existant réutilisé |
|---|---|
| Moteur runtime | `crates/bridget-daemon/src/project_runtime.rs` résout politiques, montages, limites et cycle Docker CLI. |
| Liaison projet | `ProjectBindRequest` accepte déjà Host ou Docker lors de l'enregistrement. |
| Cycle local | `ProjectRuntimeOperation` fournit prepare, status, stop, remove, recreate et switch backend. |
| Ressources | SPEC-067 fournit profils résolus et catalogue fermé de mounts/env/secrets. |
| UI | `/v1/projects/runtime` expose seulement le statut dans `crates/bridget-daemon/src/ui.rs`. |
| Service installé | Le unit systemd ne charge actuellement que `--project-root-policy`. |
| Image | `infra/project-runtime/` est explicitement une fixture de SPEC-066, pas une image de production. |

## Lacune structurante

L'enregistrement initial peut lier directement Docker, mais le cycle runtime actuel ne sait pas promouvoir une liaison Host existante vers Docker. `SwitchBackend` retourne simplement l'état d'une liaison Host. Une opération atomique `ActivateDocker` doit donc être ajoutée au contrat, au store et à l'UI.

## Décisions de planification

1. Étendre le moteur SPEC-066 au lieu d'en créer un autre.
2. Publier une image headless Linux amd64 par digest et un catalogue opérateur séparé.
3. Conserver Host comme valeur de migration et rendre le défaut serveur modifiable par le centre de contrôle.
4. Implémenter Host -> Docker comme saga atomique: résolution, préparation, publication de liaison; rollback complet avant visibilité en cas d'échec.
5. Utiliser des chemins absolus identiques pour checkout, git common dir et worktrees afin de respecter Git linked worktrees.
6. Réutiliser SPEC-067 pour secrets et extensions. L'image ne porte que les outils non secrets.
7. Ne fournir ni GUI, ni navigateur, ni orchestrateur externe dans ce lot.

## Constitution check initial

| Gate | Verdict | Justification |
|---|---|---|
| Worktree isolé | PASS | Branche `session-085-runtime-docker-production` issue de `origin/main`. |
| Réutilisation | PASS | Moteur, store, protocoles, profil et ressources existants sont étendus. |
| Minimalisme | PASS | Un Dockerfile de production et des contrats fermés, sans Compose/Kubernetes. |
| Sécurité | PASS sous garde | Digest, non-root, rootfs RO, cap drop, limites et absence de secrets. |
| Réversibilité | PASS sous garde | Host reste rollback explicite et le dépôt reste sur l'hôte. |

## Plan par lots

### Lot 1 - Image et catalogues de production

- Créer `infra/project-runtime/production/` avec Dockerfile multi-stage, versions, SBOM ou inventaire, script de build et test de fumée.
- Épingler la base par digest et vérifier l'identité de l'image résolue.
- Créer des exemples versionnés de runtime policy et resource catalog sans secret.
- Installer un socle de développement neutre et référencer les clients fournisseurs approuvés à versions explicites par couches ou ressources de politique.

### Lot 2 - Configuration serveur

- Ajouter `execution.default_backend` et la politique par défaut au catalogue de contrôle.
- Faire charger les trois fichiers de politique par le service systemd sans changer les projets existants.
- Vérifier l'accès Docker du compte de service sans modifier automatiquement groupe, socket ou permissions.
- Ajouter une projection de capacité Docker: daemon, image, politique et catalogue.
- Refuser `docker` comme défaut si la capacité n'est pas complètement attestée.

### Lot 3 - Activation atomique Host vers Docker

- Étendre `ProjectRuntimeOperation` avec `ActivateDocker` et politique fermée.
- Ajouter au store une transition préparatoire distincte de la liaison active.
- Préparer et attester le conteneur, puis publier la nouvelle génération Docker dans une transaction logique.
- En cas d'échec, retirer l'environnement partiel et conserver la liaison Host originale.
- Conserver les gardes d'agents actifs pour toutes les transitions.

### Lot 4 - Montages Git et réconciliation

- Résoudre `git rev-parse --git-common-dir` de façon bornée ou via lecture de métadonnées validées.
- Monter checkout, common dir et worktrees aux mêmes chemins absolus.
- Détecter les worktrees ajoutés et imposer recreate lorsque la topologie de montage change.
- Réconcilier au démarrage conteneur réel, image résolue, epoch et liaison persistée.

### Lot 5 - UI par projet

- Étendre la projection runtime et ajouter les routes POST typées.
- Afficher backend, politique, état, disponibilité et raison dans le menu/centre projet.
- Afficher le choix de backend à la création/import avec valeur serveur présélectionnée.
- Demander confirmation pour stop, remove, recreate et retour Host, sans état optimiste.

### Lot 6 - Installation et preuves

- Construire l'image, enregistrer son digest et installer les fichiers de politique.
- Mettre à jour le drop-in systemd avec les paramètres runtime/resources.
- Vérifier une migration Host intacte, une création Docker, une activation existante, un rollback et un redémarrage.
- Auditer image, mounts, logs et projections pour les secrets.

## Fichiers principaux prévus

| Surface | Évolution |
|---|---|
| `crates/bridget-transport/src/protocol.rs` | Opération `ActivateDocker` et refus fermés. |
| `crates/bridget-daemon/src/project_runtime.rs` | Préparation, topologie Git et réconciliation. |
| `crates/bridget-daemon/src/store.rs` | Transition atomique de backend et reçus. |
| `crates/bridget-daemon/src/daemon.rs` | Orchestration de saga et capacité runtime. |
| `crates/bridget-daemon/src/control_settings.rs` | Défaut serveur fermé. |
| `crates/bridget-daemon/src/ui.rs` | Projections et actions POST. |
| `crates/bridget-daemon/assets/ui/*` | Choix et pilotage projet. |
| `infra/project-runtime/production/*` | Image, inventaire et validation. |
| `packaging/systemd/*` | Chargement des politiques dans l'installation. |

## Stratégie de test

1. Tests protocol/store de transition Host -> Docker et rejouabilité.
2. Tests de rollback après chaque point de panne simulé.
3. Tests d'intégration Docker isolés et skippables seulement lorsque Docker est absent.
4. Tests de topologie Git avec checkout principal et deux worktrees liés.
5. Scan de l'image et des sorties pour secrets factices.
6. Tests UI des états et confirmations, puis validation manuelle sur serveur.

## Risques et parades

| Risque | Parade |
|---|---|
| Contrôle du daemon Docker équivaut à un fort privilège hôte | Seul le daemon Bridget possède le socket; aucun agent ni UI ne reçoit le socket ou des arguments libres. |
| Image périme vite | Versions et digest explicites, rebuild volontaire et statut de mise à jour informatif. |
| Transition laisse un état hybride | Saga persistée, publication tardive et compensation avant réponse. |
| Worktrees Git cassés | Chemins identiques et montage du common dir, tests réels multi-worktree. |
| Fuite de secret | Ressources SPEC-067, inventaire image et scans de logs/projections. |
| Docker indisponible | Refus exact, aucune exécution Host implicite. |
| État persistant dispersé | Racine hôte unique administrée; aucun volume Docker nommé comme source de vérité. |

## Constitution check post-conception

PASS sous garde de tests d'intégration. La fonctionnalité rend exploitable un moteur déjà présent, conserve la réversibilité Host et ne donne jamais le contrôle Docker aux agents.
