# Plan d'implémentation - Pilotage des rondes par projet dans l'interface

**Branche**: `session-081-pilotage-rondes-projet-ui`
**Date**: 2026-08-31
**Spec**: `specs/081-pilotage-rondes-projet-ui/spec.md`

## Résumé

Étendre la projection locale des projets avec la politique de ronde déjà autoritaire, puis ajouter une mutation versionnée dans le relais UI. Le menu contextuel établi par la SPEC-080 affiche l'état confirmé, permet l'activation ou la désactivation et conserve la dernière valeur en cas de refus. Le store existant reçoit uniquement les trois faits nécessaires à l'observabilité du dernier dispatch; aucun second registre, scheduler ou timer n'est créé.

## Contexte technique

- Langage backend et transport: Rust 2024, `serde`, `rusqlite` et socket Unix existante.
- Interface: JavaScript sans framework, HTML et CSS embarqués dans le binaire.
- Tests: tests Rust unitaires et intégration du relais, programme Node intégré à `app.js`, scénario Gherkin métier.
- Stockage: SQLite local existant, table `project_round_policies` étendue par migration additive.
- Performance: lecture O(p) et jointure O(p), où p est le nombre de projets liés; mutation O(1).
- Dépendances nouvelles: aucune.

## Constitution Check

| Gate | Statut | Application |
|---|---|---|
| Cycle SpecKit complet | PASS | spec, plan, audit de réutilisation, tâches, analyse, implémentation et audit sont produits dans ce lot. |
| Worktree isolé | PASS | travail dans `/home/moi/bridget-referent/.worktrees/session-081-pilotage-rondes-projet-ui`. |
| Réutiliser avant de créer | PASS | extension de `ProjectRoundPolicyV1`, du relais `/v1/projects` et du menu projet existant. |
| Minimalisme | PASS | pas de commande globale, cadence, timer, dépendance, écran ou store supplémentaire. |
| Observabilité | PASS | dernier passage persisté avec instant et résultat fermé, sans contenu ni secret. |
| Complexité | PASS | une jointure par table de hachage O(p), aucune recherche quadratique. |
| Tests avant code | PASS | contrats et scénarios sont définis avant les changements productifs. |
| Responsabilité future | PASS | contrat versionné, états fermés, migration lisible et vérifications reproductibles. |
| Sécurité et confidentialité | PASS | seulement identifiant opaque, génération, booléens, révision et instants. |

Aucune décision architecturale nouvelle ne justifie un ADR : cette SPEC étend explicitement les choix acceptés par les SPEC-079 et 080 sans changer de technologie, frontière ou autorité.

## Recherche et décisions

Les décisions détaillées sont dans `research.md`. Elles reposent sur le code et les contrats déjà livrés :

1. Réutiliser `ProjectRoundRequest`, `ProjectRoundProjection` et `ProjectRoundPolicyV1`.
2. Enrichir `GET /v1/projects` afin que la ligne et son menu partagent une seule projection confirmée.
3. Ajouter une route de mutation étroite, non générique, liée à `project_id + binding_generation`.
4. Persister le dernier résultat de dispatch dans la ligne de politique courante, sans nouveau journal.
5. Décrire le prochain passage comme « au prochain cycle global, au plus sept minutes », car le relais ne connaît pas l'instant exact de réveil du timer.

## Réutilisation de l'existant

| Besoin | Existant | Décision |
|---|---|---|
| Autorité de politique | `ProjectRoundRequest` et `ProjectRoundProjection` dans `crates/bridget-transport/src/protocol.rs:526` | Étendre, ne pas dupliquer. |
| Cadence | `PROJECT_ROUND_INTERVAL_SECS` dans `crates/bridget-transport/src/protocol.rs:584` | Projeter 420 secondes, ne pas recalculer une cadence. |
| Persistance | `project_round_policies` dans `crates/bridget-daemon/src/store.rs:712` | Migration additive de trois colonnes optionnelles. |
| Mutation sûre | `apply_project_round_mutation` dans `crates/bridget-daemon/src/store.rs:1449` | Appeler via le contrat existant. |
| Projection projets | `read_projects` dans `crates/bridget-daemon/src/ui.rs:2363` | Joindre les politiques par identifiant en O(p). |
| Menu projet | `openProjectContextMenu` dans `crates/bridget-daemon/assets/ui/app.js:7639` | Ajouter un item à case d'état et son détail. |
| Rafraîchissement | `refreshProjects` dans `crates/bridget-daemon/assets/ui/app.js:7804` | Rafraîchir après reçu; aucun état optimiste. |
| Style menu | `.project-context-menu` dans `crates/bridget-daemon/assets/ui/theme.css:2429` | Ajouter seulement les sous-éléments nécessaires. |

## Architecture cible

### Lecture

1. Le relais ouvre le registre projet existant et obtient les liaisons.
2. Il ouvre une session client locale avec la capacité `ProjectRoundPolicyV1` et demande la liste des politiques.
3. Il indexe les politiques par `project_id`, puis produit chaque ligne avec sa génération et son état de ronde.
4. Si la capacité de ronde est absente ou la réponse invalide, la liste n'invente pas `disabled`; elle renvoie une indisponibilité actionnable.

### Mutation

1. Le navigateur envoie `version`, `command_id`, `project_id`, `binding_generation` et l'état souhaité.
2. Le relais valide strictement l'enveloppe et la traduit en `Enable` ou `Disable`.
3. Le daemon applique la commande idempotente existante.
4. Le relais retourne la projection confirmée ou un refus fermé.
5. Le navigateur rafraîchit la liste uniquement après succès; en cas d'échec il garde l'objet projet inchangé.

### Dernier passage

Après que le socle idempotent a rendu son issue, le daemon classe le dispatch en `deposited`, `refused` ou `indeterminate`, puis met à jour la politique de la génération exacte. L'écriture est monotone par occurrence : un passage plus ancien ne remplace jamais un passage récent. Le texte du message, le fournisseur, la racine et le secret ne sont pas stockés.

### Génération et rebind

La projection est toujours recherchée par `project_id + binding_generation`. Une nouvelle génération sans ligne apparaît `configured=false`, `enabled=false` et sans dernier passage. L'ancienne ligne reste historique mais n'est jamais jointe à la nouvelle liaison.

## Modèle de données

Voir `data-model.md`. La seule migration persistante ajoute à `project_round_policies` :

- `last_occurrence_at`, optionnel;
- `last_dispatch_state`, optionnel et fermé;
- `last_dispatch_observed_at`, optionnel.

Aucune table ni index nouveau n'est nécessaire. La clé primaire actuelle fournit déjà l'accès O(1) par projet et génération.

## Contrat du relais

Voir `contracts/project-round-ui-v1.md`.

- `GET /v1/projects` ajoute `binding_generation` et `round` à chaque projet.
- `POST /v1/projects/round` applique une activation ou désactivation.
- Les erreurs sont fermées et ne révèlent aucune donnée hôte.

## Fichiers concernés

| Fichier | Modification |
|---|---|
| `crates/bridget-transport/src/protocol.rs` | état fermé du dispatch et champs optionnels de projection. |
| `crates/bridget-daemon/src/store.rs` | migration additive, lecture et écriture monotone du dernier passage. |
| `crates/bridget-daemon/src/daemon.rs` | classement et persistance du résultat après dispatch. |
| `crates/bridget-daemon/src/ui.rs` | négociation de capacité, jointure de lecture et route de mutation. |
| `crates/bridget-daemon/assets/ui/app.js` | projection pure, menu, mutation confirmée et tests Node. |
| `crates/bridget-daemon/assets/ui/theme.css` | état visuel compact et accessible dans le menu. |
| `tests/features/081-pilotage-rondes-projet-ui.feature` | acceptation métier. |

Aucun changement n'est prévu dans le script de dispatch, le timer global, Maicie, les wrappers fournisseur ou Bridget Desktop.

## Plan de tests

1. Transport : sérialisation des nouveaux champs et compatibilité des champs absents.
2. Store : migration d'une base existante, état absent, écriture monotone et isolation par génération.
3. Daemon : un dispatch déposé ou refusé produit le résultat fermé attendu sans modifier la sélection des projets.
4. Relais : capacité exigée, jointure de liste, mutation confirmée, corps inconnu, génération divergente et projet inactif.
5. Node : libellés, accessibilité, absence d'état optimiste, rebind non configuré et texte de prochain passage.
6. Intégration : programme Node complet, tests ciblés Rust, relais UI, formatage, clippy applicable et `git diff --check`.

## Ordre d'implémentation

1. Écrire les tests de contrat transport et store.
2. Étendre le type et la migration, puis faire passer les tests ciblés.
3. Ajouter la persistance du résultat de dispatch.
4. Écrire les tests du relais et ajouter la lecture puis la mutation.
5. Écrire les tests Node et intégrer le menu et ses styles.
6. Exécuter les suites complètes applicables et le scénario de validation manuelle documenté.

## Risques et parades

| Risque | Parade |
|---|---|
| État UI mensonger pendant la requête | aucune mutation locale avant reçu, contrôle occupé et rafraîchissement confirmé. |
| Politique d'une ancienne génération réutilisée | génération obligatoire dans la ligne UI et dans la mutation. |
| Liste projets rendue quadratique | indexation des politiques dans une table de hachage avant la projection. |
| Ancien daemon sans capacité | erreur explicite, jamais de faux état désactivé. |
| Heure exacte inventée | délai maximal constant, pas de timestamp du prochain timer. |
| Migration destructive | colonnes optionnelles ajoutées sans réécriture des lignes historiques. |
| Scope étendu au scheduler | aucun changement du timer, du script ou de sa cadence. |

## Déploiement et réversibilité

Le changement n'est ni committé, ni mergé, ni déployé par ce pipeline. Une livraison ultérieure devra compiler et installer le même binaire daemon/CLI avant redémarrage. Le rollback du binaire ignore les colonnes additives; les décisions de politique existantes restent valides.

## Constitution Check post-conception

PASS. La conception garde une autorité unique, ajoute trois faits optionnels sur la ligne existante et une route métier fermée. Elle évite une API générique, un nouveau store, un nouvel écran et toute promesse temporelle non attestée. Un mainteneur peut inspecter la politique, reproduire la mutation par la CLI existante, vérifier la migration et supprimer la présentation UI sans toucher au scheduler.
