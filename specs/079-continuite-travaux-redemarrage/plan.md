# Plan d'implémentation - SPEC-079 Continuité des travaux après redémarrage

## Résumé

Compléter le modèle d'exécution SPEC-064 au point où le chemin UI idempotent le
contourne encore. Une remise `trigger_turn` sera liée à une exécution avant
injection. Lors d'un redémarrage complet, Bridget créera une continuation
`reconstructed`, puis utilisera la saga de remise idempotente existante pour
livrer le message exact.

Ajouter séparément une politique de ronde par projet. Le timer reste global et
ne porte aucune autorité projet. Il demande au daemon la liste des projets
activés et émet une occurrence idempotente par projet.

## Constitution Check

| Gate | Décision |
|---|---|
| Travail serveur | worktree isolé sur cartae.app |
| Réutilisation | WorkSubmission, Execution, DeliveryExecutionLink, ContinuationMode et ProjectBinding |
| Identité projet | `ProjectReference` 065, aucune inférence par chemin |
| Test avant code | tests store, protocole, wrapper et daemon ajoutés avant les branches productives |
| Complexité | O(e + p), e exécutions actives et p projets liés |
| Réversibilité | ronde désactivée par défaut, scheduler unique, aucun timer par projet |
| Fournisseurs | contrat commun au wrapper, aucune condition Claude/Codex/Cursor |

## Architecture réutilisée

1. `ExecutionStore` conserve déjà le message exact, les exécutions et les continuations.
2. `IdempotencyStore` conserve les remises et possède déjà `send_delivery_execution_links`.
3. `DeliverExecution` prouve le binding wrapper pour une remise non idempotente.
4. `DeliverIdempotent` possède déjà les garanties de rejeu par `delivery_id`.
5. `ProjectBinding` est l'autorité Bridget de l'identité et de la génération projet.
6. `bridget-ronde.py` reste l'observateur passif existant.

## Conception cible

### 1. Admission UI

Le relais UI pose `MessageOrigin::Human` et `MessageIntent::TriggerTurn`. Le
daemon admet le message exact dans `ExecutionStore`, puis crée la remise et son
`DeliveryExecutionLink` dans une transaction `IdempotencyStore`. La projection
du lien est ajoutée à `DeliverIdempotent` comme champ optionnel compatible avec
les anciennes enveloppes.

### 2. Binding wrapper

Le wrapper traite le contexte d'exécution avant l'appel au tracker idempotent.
Une remise nouvelle crée le binding puis injecte. Un rejeu déjà accusé est
supprimé par le tracker et ne crée pas un second tour. Un lien absent conserve
le comportement historique.

### 3. Réconciliation au Register

Après enregistrement d'un wrapper:

1. si `turn_in_progress=true`, ne rien reconstruire;
2. rejouer d'abord toute remise `dispatching` existante;
3. chercher une exécution active sans remise en vol pour le nom attesté;
4. terminaliser le parent avec `daemon_restart`;
5. créer dans la même transaction un enfant sur la même soumission et le même projet;
6. enregistrer `ContinuationMode::Reconstructed`;
7. créer une remise idempotente interne et son lien causal dans une transaction;
8. la livrer par le chemin `DeliverIdempotent` normal.

Une enveloppe absente ou plusieurs exécutions actives concurrentes produisent
un diagnostic fermé. Aucun prompt n'est inventé.

### 4. Politique de ronde

Créer un registre local `project_round_policies` distinct du registre métier.
Chaque ligne est épinglée à une génération de `ProjectBinding`. Une commande
idempotente active ou désactive la politique. Une politique absente ou devenue
stale après rebind est désactivée.

Un contrat client local expose `list`, `status`, `enable` et `disable`. La
liste des cibles ne retourne que les projets actifs et activés.

### 5. Scheduler unique

Le timer systemd demeure unique. Un dispatcher source-controlled calcule
`occurrence = floor(now / interval) * interval`, liste les projets activés et
envoie un message `origin=routine`, `intent=trigger_turn` avec une clé stable
par `(project_id, binding_generation, occurrence)`. Un second lancement du même
tick obtient un rejeu idempotent.

La désactivation filtre seulement les occurrences futures. Elle n'appelle
aucune commande de contrôle d'exécution ou de cycle de vie.

## Fichiers prévus

| Fichier | Responsabilité |
|---|---|
| crates/bridget-transport/src/protocol.rs | contextes de remise et contrat de politique |
| crates/bridget-daemon/src/execution_store.rs | reconstruction transactionnelle |
| crates/bridget-daemon/src/idempotency.rs | lien remise-exécution atomique et lookup |
| crates/bridget-daemon/src/store.rs | politique de ronde par projet |
| crates/bridget-daemon/src/daemon.rs | admission, Register et commandes de politique |
| crates/bridget-daemon/src/wrapper.rs | binding des remises idempotentes liées |
| crates/bridget-daemon/src/ui.rs | intention explicite des messages humains |
| crates/bridget-daemon/src/cli.rs | pilotage et dispatch de ronde |
| scripts/bridget-ronde-dispatch.py | occurrence globale et émission par projet |
| tests/features/079-continuite-travaux-redemarrage.feature | acceptation métier |

## Ordre d'implémentation

1. Ajouter les tests du store d'exécution et du lien de remise.
2. Rendre atomique le lien de remise et ajouter sa projection protocolaire.
3. Raccorder UI, daemon et wrapper.
4. Implémenter et tester la reconstruction au Register.
5. Ajouter la politique de ronde par projet et son contrat client.
6. Ajouter le dispatcher unique et ses tests sans toucher au timer de production.
7. Exécuter les suites ciblées puis workspace.
8. Converger exigence par exigence et auditer les frontières de crash.

## Risques

| Risque | Atténuation |
|---|---|
| Double injection | saga idempotente existante et lien durable par delivery_id |
| Travail perdu entre deux stores | enfant actif récupérable au redémarrage et lien atomique avec la remise |
| Reprise d'un tour vivant | garde `turn_in_progress` avant reconstruction |
| Prompt inventé | `message_json` obligatoire, défaut terminal visible |
| Ronde confondue avec reprise | stores, contrats et tests séparés |
| Rebind trans-projet | génération obligatoire et politique stale |
| Explosion systemd | un timer et une itération O(p) |

## Déploiement

La skill ne committe et ne déploie rien automatiquement. Après validation et
intégration explicites, le binaire daemon et le dispatcher devront être installés
ensemble avant activation volontaire des politiques projet.
