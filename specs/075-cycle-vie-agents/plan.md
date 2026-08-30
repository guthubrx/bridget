# Plan d'implémentation - SPEC-075

## Résumé

Faire de `fleet.json` l'inventaire durable des agents gérés, avec un état de
cycle de vie distinct de la politique de reprise automatique. Réutiliser le
superviseur de processus et la saga de spawn pour implémenter trois commandes
explicites et les exposer de façon identique en protocole, CLI, HTTP et UI.

## Contexte technique

- Backend et daemon: Rust stable, socket Unix, serde JSON.
- Interface: HTML, CSS et JavaScript natifs embarqués.
- Persistance: `fleet.json` atomique plus SQLite existant pour les sagas.
- Processus: superviseur et groupes de processus SPEC-009.
- Dépendance nouvelle: aucune.
- Worktree: `/home/moi/bridget-referent/.worktrees/session-075-cycle-vie-agents`.

## Constitution Check

| Gate | Verdict | Justification |
|---|---|---|
| SpecKit et isolation | PASS dégradé | Worktree et artefacts isolés; sync tenté, CLI `specify` absent |
| Réutiliser avant de créer | PASS | Superviseur, état atomique, saga spawn, fiche et relais réutilisés |
| Minimalisme | PASS | Trois opérations explicites sans nouveau service ni nouvelle base |
| Charge future | PASS | Une source durable et une autorité daemon pour CLI, HTTP et UI |
| Complexité | PASS | Projection et mutations O(n) au pire, aucune requête par ligne |
| Sécurité | PASS | Autorité locale existante, aucun signal depuis le navigateur, aucune purge |
| Accessibilité | PASS | Matrice d'actions, confirmations nommées, focus SPEC-073 conservé |
| Recherche | PASS dégradé | Sources officielles consultées; `mem` absent |

## Architecture cible

```text
CLI ou interface locale
        |
        v
StopOrder | RelaunchOrder | DecommissionOrder
        |
        v
daemon: autorité de cycle de vie
        |
        +--> FleetSupervisor + fleet.json schema 4
        |
        +--> superviseur de groupes SPEC-009
        |
        +--> saga spawn existante pour relaunch
        |
        v
résultat typé et corrélé
        |
        v
snapshot dynamique de la barre latérale
```

## Phases

### Phase 1 - Modèle durable et compatibilité

1. Ajouter `DesiredLifecycleState` et `persistent` à `DesiredEquipier`.
2. Passer le schéma écrit à 4 avec valeurs par défaut pour 1 à 3.
3. Ajouter des mutations ciblées: marquer arrêté, retirer seulement une
   génération possédée, décommissionner vers une tombstone cachée.
4. Persister tous les agents gérés connectés, y compris non persistants.
5. Au démarrage, convertir `running + persistent=false` en `stopped` et filtrer
   les reprises sur `running + persistent=true`.
6. Projeter les entrées arrêtées depuis `fleet.json` dans `who`.

### Phase 2 - Commandes daemon

1. Ajouter les résultats fermés de relance et décommissionnement au protocole.
2. Corriger `StopOrder`: conserver la définition durable arrêtée.
3. Ajouter `RelaunchOrder`: valider l'état, créer une nouvelle saga depuis la
   définition figée, attendre `Connected`, préserver l'ancien état en échec.
4. Ajouter `DecommissionOrder`: arrêter par le chemin commun si nécessaire,
   passer l'entrée en `decommissioned` et retirer la présence seulement après
   succès.
5. Persister les morts spontanées comme `stopped`.
6. Ajouter les logs structurés de transition.

### Phase 3 - Compatibilité héritée

1. Ajouter une adoption explicite pré-redémarrage qui prend la liste arrêtée
   courante et vérifie sa dernière génération gérée SQLite.
2. Ne jamais adopter un nom absent de l'instantané ou sans définition complète.
3. Produire un rapport noms adoptés/refusés sans secret.

### Phase 4 - CLI et relais HTTP

1. Ajouter `bridget relaunch` et `bridget decommission`.
2. Généraliser le parseur et la corrélation sans modifier le comportement de
   `bridget stop` hors nouvelle sémantique durable.
3. Ajouter les deux routes HTTP et leur validation commune.
4. Mapper chaque issue daemon vers un statut et un code stables.

### Phase 5 - Interface

1. Remplacer le bouton unique par la matrice d'actions de la spec.
2. Renommer l'action existante en « Arrêter ».
3. Ajouter les confirmations distinctes relance et décommissionnement.
4. Généraliser l'état en vol et les retours sans mutation optimiste.
5. Conserver le panneau, le focus, les logos et le rafraîchissement incrémental
   existants.
6. Afficher clairement l'état arrêté dans la ligne et la fiche.

### Phase 6 - Tests et preuve

1. Sérialisation protocole et migration schema 1 à 4.
2. Transitions fleet, compensation générationnelle et crash-frontières.
3. Tests daemon stop, relaunch, decommission et courses.
4. Contrats HTTP et matrice JavaScript.
5. Preuve intégrée avec daemon et agent fixture temporaires.
6. Non-régressions SPEC-009, SPEC-071, SPEC-073 et barre dynamique.

## Fichiers prévus

| Fichier | Évolution |
|---|---|
| `crates/bridget-transport/src/protocol.rs` | Ordres, résultats et issues typés |
| `crates/bridget-daemon/src/desired_state.rs` | Schéma 4 et mutations durables |
| `crates/bridget-daemon/src/fleet.rs` | Transitions stop/relaunch/decommission et compensation ciblée |
| `crates/bridget-daemon/src/lifecycle.rs` | Préparation de relance figée |
| `crates/bridget-daemon/src/daemon.rs` | Routage des commandes, projection et événements |
| `crates/bridget-daemon/src/cli.rs` | Commandes CLI |
| `crates/bridget-daemon/src/ui.rs` | Routes HTTP et mapping des issues |
| `crates/bridget-daemon/assets/ui/app.js` | Matrice, dialogues et retours |
| `crates/bridget-daemon/assets/ui/theme.css` | Actions et état arrêté |
| Tests existants associés | Couverture unitaire et intégrée |

## Risques et parades

| Risque | Parade |
|---|---|
| Ressusciter un agent arrêté après crash | Écrire `stopped` avant d'attendre le superviseur |
| Perdre l'agent sur échec de relance | Compensation conditionnée par commande et génération |
| Décommissionner alors que le processus vit | Retrait seulement après verdict d'arrêt positif |
| Mélanger ancien et nouvel historique | Tombstone cachée qui réserve le nom décommissionné |
| Réactiver un mandat clos | Relance sans ancien ownership actif |
| Polluer la liste avec tout l'historique | Adoption bornée à l'instantané arrêté pré-migration |
| Dupliquer le superviseur | Toutes les terminaisons passent par le chemin SPEC-009 |
| Écraser des développements concurrents | Worktree isolé, aucun merge, commit ou déploiement automatique |

## Déploiement prévu mais non exécuté par le pipeline

1. Rebaser ou fusionner prudemment sur la tête de `main` au moment de livrer.
2. Compiler et tester le binaire.
3. Exécuter l'adoption héritée pendant que l'ancien daemon expose encore son
   instantané arrêté.
4. Installer le binaire validé.
5. Redémarrer daemon et relais.
6. Vérifier build-id, agents arrêtés, agents actifs et routes.
