# Tâches : Renommer un agent Bridget

**Entrées** : artefacts de conception de `/specs/001-renommer-agent/`  
**Prérequis** : `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/rename-agent-cli.md`

## Phase 1 — Fondations

**Objectif** : étendre le protocole et l’annuaire sans modifier leur comportement existant.

- [X] T001 Étendre `crates/bridget-transport/src/protocol.rs` avec les messages `Rename` et `Renamed`, puis tester leur sérialisation et désérialisation.
- [X] T002 Étendre `crates/bridget-core/src/router.rs` avec un renommage atomique par `connection_id`, incluant succès, collision, nom invalide et connexion inconnue.
- [X] T003 Mettre à jour `crates/bridget-daemon/src/daemon.rs` pour traiter `Rename`, synchroniser `conn_names` avec le routeur et retourner `Renamed` ou `Nack`.

**Point de contrôle** : un renommage daemon est atomique et l’ancien nom est absent de l’annuaire.

---

## Phase 2 — Histoire utilisateur 1 : renommer son agent actif (P1) 🎯 MVP

**Objectif** : un agent actif change de nom sans reconnexion et continue à communiquer sous sa nouvelle identité.

**Test indépendant** : avec deux agents connectés, renommer le premier ; vérifier que le second peut lui envoyer un message avec le nouveau nom et que l’ancien est rejeté.

- [X] T004 [US1] Ajouter dans `crates/bridget-daemon/src/wrapper.rs` le fichier d’état local du wrapper, qui porte le nom courant.
- [X] T005 [US1] Modifier `crates/bridget-daemon/src/cli.rs` pour que `bridget rename <nouveau-nom>`, `send` et `reply` consultent le nom courant plutôt que le nom d’environnement figé.
- [X] T006 [US1] Étendre `crates/bridget-daemon/tests/integration_test.rs` avec le parcours renommage sans reconnexion, routage par le nouveau nom et rejet de l’ancien nom.

**Point de contrôle** : l’histoire P1 fonctionne depuis l’interface CLI documentée, y compris pour un message envoyé après le renommage.

---

## Phase 3 — Histoire utilisateur 2 : préserver une identité renommée (P2)

**Objectif** : le nouveau nom confirmé est réutilisé lors de la reprise de l’agent.

**Test indépendant** : renommer, arrêter l’agent puis le reprendre avec le même identifiant de session ; vérifier son nouveau nom.

- [X] T007 [US2] Mettre à jour `crates/bridget-daemon/src/wrapper.rs` pour persister le nouveau nom uniquement après `Renamed`.
- [X] T008 [US2] Ajouter dans `crates/bridget-daemon/tests/integration_test.rs` le test de reprise après renommage et le test de non-persistance après refus.

**Point de contrôle** : la reprise conserve seulement une identité validée par le démon.

---

## Phase 4 — Finalisation transversale

- [X] T009 Mettre à jour l’aide de `crates/bridget-daemon/src/cli.rs` et le contrat `specs/001-renommer-agent/contracts/rename-agent-cli.md` si l’interface effective diffère du contrat prévu.
- [X] T010 Exécuter `cargo test` à la racine du dépôt et rejouer le parcours de `specs/001-renommer-agent/quickstart.md` ; documenter toute limite d’environnement.

## Dépendances et ordre d’exécution

`T001 → T003 → T004 → T005 → T006 → T007 → T008 → T009 → T010`.

T001 et T002 concernent des fichiers distincts et peuvent être préparées en parallèle ; T003 dépend de leurs contrats. T004 est le socle de l’histoire P1. L’histoire P2 dépend de la confirmation de renommage livrée par P1.

## Stratégie d’implémentation

Livrer d’abord les fondations et l’histoire P1, puis valider le parcours de routage sans reconnexion. Ajouter ensuite la persistance, qui est isolée au wrapper. Cette séquence évite d’écrire un état persistant avant que l’identité active soit fiable.

## Validation Article XX

Le canal de contrôle local est la seule complexité ajoutée : il est justifié car il évite une incohérence d’identité des commandes enfant. Les tâches modifient les responsabilités existantes au lieu d’introduire un nouveau service.
