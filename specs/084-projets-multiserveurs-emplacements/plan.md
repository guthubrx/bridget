# Plan d'implémentation - SPEC-084 Projets multi-serveurs et catalogue d'emplacements

## Contexte technique confirmé

| Élément | Existant réutilisé |
|---|---|
| Flotte multi-serveurs | Bridget Desktop compose déjà les profils et sources dans `apps/bridget-desktop/ui/fleet-app.js` et ouvre le panneau cible via `panel_open` dans `apps/bridget-desktop/src-tauri/src/lib.rs`. |
| Registre de projets | SPEC-065 fournit identité opaque, génération et liaison serveur locale. |
| Création/import | `crates/bridget-daemon/src/project_workspace.rs` et les routes `/v1/projects/*` de `crates/bridget-daemon/src/ui.rs`. |
| Politique racine | `ProjectRootPolicy` v1 et son remplacement atomique dans `crates/bridget-daemon/src/project_policy.rs`. |
| Centre de contrôle | Catalogue fermé, prévisualisation et application de SPEC-080 dans `crates/bridget-daemon/src/control_settings.rs`. |

## Décisions de planification

1. La fédération reste dans Bridget Desktop. Aucun registre global n'est ajouté.
2. Le dialogue create/import reçoit un `source_id` présélectionné, affiche toujours sa source et permet de la changer avant confirmation.
3. Le daemon reste ignorant des autres serveurs. Il ne reçoit qu'une commande locale relative à son propre catalogue.
4. `ProjectRootPolicy` évolue vers un document v2 d'emplacements typés. La v1 reste décodable mais ses entrées sont importables au chemin exact seulement.
5. `workspace` et `exact_project` portent les permissions. `system_only` et `default_creation` sont des marqueurs fermés.
6. La création autorise un enfant direct d'un workspace. L'import autorise un descendant de workspace ou le chemin exact d'un exact project.
7. Le checkout Bridget de l'installation sera migré plus tard par SPEC-086 en `exact_project + system_only`.

## Constitution check initial

| Gate | Verdict | Justification |
|---|---|---|
| Worktree isolé | PASS | Branche `session-084-projets-multiserveurs-emplacements` issue de `origin/main`. |
| Réutilisation | PASS | Extension du Desktop multi-source, du registre, de la politique racine et du centre de contrôle. |
| Minimalisme | PASS | Aucun serveur central, scan de filesystem, transfert de dépôt ou nouvelle dépendance. |
| Sécurité | PASS sous garde | Compatibilité v1 restrictive, chemins canoniques et mutation générationnelle. |
| Observabilité | PASS | Commandes et refus corrélés par source, emplacement et génération sans contenu projet. |

## Plan par lots

### Lot 1 - Contrat d'emplacements v2

- Ajouter les types `ProjectLocation`, `ProjectLocationKind` et `ProjectLocationCatalogV2` dans `project_policy.rs`.
- Décoder v1 et v2, produire une projection v2 unique, puis tester les invariants d'identifiants, chemins, unicité, imbrication et défaut.
- Séparer `validate_creation_parent` et `validate_import_root` afin que les capacités ne puissent plus être confondues.
- Conserver écriture atomique, génération et comportement fail-closed existants.

### Lot 2 - Création et import guidés

- Faire consommer un `location_id` aux requêtes de prévisualisation et d'application dans `project_workspace.rs`.
- Construire le chemin enfant exclusivement côté daemon à partir du workspace confirmé.
- Conserver les routes v1 en lecture/compatibilité et versionner les nouvelles charges v2.
- Étendre les projections UI avec emplacement, capacités et raison d'indisponibilité.

### Lot 3 - Parcours Desktop multi-serveurs

- Étendre le sélecteur de source `targetDialog` livré par SPEC-081 pour afficher la source même présélectionnée dans create/import; le panneau distant ne reçoit aucune autorité inter-serveurs.
- Charger les emplacements depuis la source choisie et invalider la prévisualisation quand la source change.
- Composer les identités de lignes par `(source_id, project_id)` sans modifier le contrat du daemon.
- Couvrir clavier, source déconnectée, capacités absentes, doubles libellés et état concurrent.

### Lot 4 - Administration et migration

- Remplacer la ligne `project_roots.allowed_roots` du centre de contrôle par un éditeur de catalogue structuré.
- Présenter le delta v1 vers v2 et exiger une promotion explicite vers `workspace`.
- Inventorier dans la prévisualisation les liaisons existantes sous chaque racine historique, sans les modifier.
- Livrer un outil de validation/migration sans effet et une procédure opérateur.
- Ne jamais modifier ou retirer automatiquement une liaison de projet existante.

### Lot 5 - Validation

- Tests Rust de politique, workspace, routes et idempotence.
- Tests JavaScript/Tauri du routage de source et du dialogue.
- Vérification de migration sur une copie de la politique de production.
- Preuve manuelle que le checkout Bridget n'est plus proposé comme parent après classification exacte.

## Fichiers principaux prévus

| Fichier | Évolution |
|---|---|
| `crates/bridget-daemon/src/project_policy.rs` | Catalogue v2, compatibilité v1 et validations séparées. |
| `crates/bridget-daemon/src/project_workspace.rs` | Prévisualisation/applique par `location_id`. |
| `crates/bridget-daemon/src/control_settings.rs` | Descripteur et mutation structurée du catalogue. |
| `crates/bridget-daemon/src/ui.rs` | Routes et projections v2 compatibles. |
| `apps/bridget-desktop/ui/fleet-app.js` | Extension du choix de source SPEC-081, emplacement explicite et identité composée. |
| `apps/bridget-desktop/src-tauri/src/lib.rs` | Routage vers le profil confirmé sans nouvelle autorité. |

## Stratégie de test

1. Tester le décodage v1 et v2 avant la migration.
2. Tester séparément création et import sur chaque type.
3. Tester les courses entre prévisualisation, génération et changement de source.
4. Tester l'accessibilité du dialogue et les capacités absentes.
5. Exécuter `cargo fmt --check`, les tests ciblés daemon/Desktop, les tests Node UI et `git diff --check`.

## Risques et parades

| Risque | Parade |
|---|---|
| Migration v1 élargit les droits | Toutes les entrées v1 deviennent exactes tant qu'un opérateur ne les promeut pas. |
| Confusion de deux projets homonymes | Identité Desktop composée et source toujours visible. |
| TOCTOU sur le chemin | Canonicalisation et revalidation sous génération au moment de l'effet. |
| Projet existant rendu inaccessible | Le retrait du catalogue n'altère pas la liaison persistée. |
| Dialogue trop complexe | Une source et un emplacement présélectionnés restent visibles, modifiables et expliqués. |

## Constitution check post-conception

PASS. Le plan remplace une ambiguïté d'autorité par deux capacités fermées et étend les composants qui portent déjà la persistance, le tunnel et l'UI. Aucune infrastructure globale n'est créée.
