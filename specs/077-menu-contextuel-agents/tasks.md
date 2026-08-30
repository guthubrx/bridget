# Tâches - SPEC-077 Menu contextuel unifié des agents

## Règles d'exécution

- Worktree exclusif:
  `/home/moi/bridget-referent/.worktrees/session-077-menu-contextuel-agents`.
- Aucun commit, merge, push, déploiement ou redémarrage production automatique.
- Un seul nœud de menu et une seule matrice d'actions pour toutes les ouvertures.
- Aucun endpoint, service ou paquet supplémentaire.
- Une tâche n'est cochée qu'après sa preuve associée.

## Phase 1 - Ligne de base et fondations locales

- [x] **T001** Exécuter les tests Node existants, tenter les tests Rust UI et
  consigner le commit de départ et les résultats dans
  `specs/077-menu-contextuel-agents/evidence/baseline.md`.
- [x] **T002** Ajouter dans `crates/bridget-daemon/assets/ui/app.js` les tests
  SPEC-077 de normalisation des préférences v1: valeur absente, JSON corrompu,
  version inconnue, types invalides, déduplication et volume borné.
- [x] **T003** Implémenter dans `crates/bridget-daemon/assets/ui/app.js` la
  normalisation et la persistance tolérante de `pinned`, `hidden` et
  `readThrough`, sans donnée de message ni secret.
- [x] **T004** Ajouter dans `crates/bridget-daemon/assets/ui/app.js` les tests
  purs de projection actif, arrêté, masqué, épinglé et de la matrice complète
  des actions disponible ou indisponible avec raison.
- [x] **T005** Implémenter `agentSidebarProjection` et
  `agentContextMenuItems` dans `crates/bridget-daemon/assets/ui/app.js`,
  avec ordre stable et complexité au plus O(n log n).

## Phase 2 - User Story 1 - Un menu par trois voies

Objectif: les trois points, le clic droit et le clavier ouvrent exactement le
même menu global.

- [x] **T006** Adapter les tests Node SPEC-073 et ajouter les tests SPEC-077
  prouvant `role=menu`, `aria-haspopup=menu`, un seul nœud global,
  `contextmenu`, touche Menu, Maj+F10, focus initial et fermeture.
- [x] **T007** Transformer dans
  `crates/bridget-daemon/assets/ui/app.js` la fiche globale existante en menu
  compact avec groupes et éléments `role=menuitem`, sans second composant.
- [x] **T008** Câbler dans `renderAgentButton` les trois points, le clic droit
  et les raccourcis clavier vers le même `openIdentityCard`, avec ancrage au
  bouton, à la ligne ou au pointeur.
- [x] **T009** Implémenter la navigation Flèche haut/bas, Début, Fin, Échap et
  Tab, en laissant les actions indisponibles parcourables pour annoncer leur
  raison, tout en bloquant leur activation et en restaurant le focus au
  déclencheur quand requis.

## Phase 3 - User Story 2 - Organiser et lire la flotte

Objectif: ouvrir, épingler, marquer comme lu, masquer et restaurer sans reload
ni saut de défilement.

- [x] **T010** Ajouter dans
  `crates/bridget-daemon/assets/ui/index.html` la section repliée
  `Agents masqués` et déclarer ses nœuds dans `UI_NODE_IDS`.
- [x] **T011** Ajouter les tests SPEC-077 des actions locales: sélection sans
  reload, ordre épinglé stable, curseur lu persistant, masquage récupérable,
  stockage indisponible et préférences sans ligne fantôme.
- [x] **T012** Implémenter dans `crates/bridget-daemon/assets/ui/app.js` les
  actions ouvrir, épingler/désépingler, marquer comme lu et
  masquer/restaurer, avec persistance locale commune.
- [x] **T013** Adapter `renderAgents` pour projeter les trois groupes, garder
  le vrai compteur actif, conserver sélection, scroll et menu ouvert, et
  fermer proprement le menu d'un agent disparu.

## Phase 4 - User Story 3 - Cycle de vie exact

Objectif: exposer arrêter, relancer et décommissionner sans logique parallèle.

- [x] **T014** Étendre les tests SPEC-075 pour vérifier que les trois commandes
  sont toujours visibles, que seule l'action éligible est activée et que toute
  indisponibilité possède une raison accessible.
- [x] **T015** Raccorder les éléments lifecycle du menu à
  `agentLifecycleEligibility`, `openStopConfirmation`,
  `submitAgentStop` et `agentLifecycleFeedback`, sans changer les routes,
  confirmations ni verdicts existants.

## Phase 5 - User Story 4 - Identité compacte et présentation

Objectif: identifier l'exécution sans transformer le menu en panneau de détail.

- [x] **T016** Réutiliser `identityCardData` pour rendre nom, présence, logo,
  fournisseur, mode et seulement les faits techniques attestés dans l'en-tête
  non interactif du menu.
- [x] **T017** Adapter `crates/bridget-daemon/assets/ui/theme.css` pour le menu,
  ses séparateurs, états focus/désactivé/destructif et la section masquée,
  sans bordure ou fond parasite autour des logos.

## Phase 6 - Convergence et preuves

- [x] **T018** Exécuter `node --test crates/bridget-daemon/assets/ui/app.js`,
  les tests Rust UI avec le bon environnement, `git diff --check` et les
  contrôles source d'absence d'endpoint ou dépendance nouvelle.
- [x] **T019** Exécuter Converge exigence par exigence, la contre-revue
  cross-provider si une capacité existe, l'audit final de simplicité et mettre
  à jour `spec.md`, `tasks.md`, `implementation.md` et `audit.md`.

## Dépendances

```text
T001 -> T002-T005 -> T006-T009 -> T010-T013 -> T014-T015 -> T016-T017
                                                         -> T018 -> T019
```

## Opportunités de parallélisme

- T002 et T004 sont des tests purs distincts mais touchent le même fichier:
  ils restent séquentiels pour éviter une collision.
- T010 touche uniquement le HTML et peut être préparée après figement de
  `UI_NODE_IDS`.
- T017 touche uniquement le CSS, mais son intégration attend la stabilisation
  des classes DOM de T007 et T016.
- Aucun sous-agent n'est requis: la surface productive est limitée à trois
  assets étroitement couplés.

## Vérification Article XX

- T007 transforme la fiche globale existante et interdit un second composant.
- T015 réutilise le moteur lifecycle existant et interdit toute route parallèle.
- T003 concentre les préférences dans un seul contrat local versionné.
- T005 et T013 séparent calcul pur et rendu sans abstraction générique.
- T018-T019 prouvent la régression, la simplicité et l'absence de charge cachée.
