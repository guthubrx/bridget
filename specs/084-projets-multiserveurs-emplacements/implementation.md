# Implémentation - SPEC-084

## Statut

En cours. Aucun commit ni déploiement n'a été réalisé par cette spécification.

## Preuves

| Étape | Résultat |
|---|---|
| Audit de réutilisation | PASS. Le sélecteur de source SPEC-081 est réutilisé. |
| Contre-revue Gemini | Indisponible: le CLI demande une authentification Google interactive sur le serveur. |
| Catalogue v2 et compatibilité v1 | Implémentés dans `project_policy.rs`. |
| Validation ciblée | `/home/moi/.cargo/bin/cargo test -p bridget-daemon project_policy --lib`: 9 succès. |
| Parcours placement v2 | Prévisualisation par `location_id`, collision, traversée, lien symbolique hors workspace, exact project et `system_only` couverts. |
| Course catalogue | La confirmation v2 relit le catalogue et le reconstruit depuis `location_id`; un changement de génération entre preview et apply est refusé avant toute connexion registre. |
| Centre de contrôle | L'éditeur est passé de racines libres v1 à des emplacements typés v2, avec prévisualisation, reçu d'application et signalement explicite d'une politique historique restrictive. |
| Validation finale ciblée | `cargo fmt --check`, 34 tests `project_`, 2 tests `control_settings`, 72 tests `ui::tests`, 59 tests `bridget-transport::protocol`, 37 tests Desktop et 127 tests Node UI passent. |
| Validation reprise 2026-09-01 | 34 tests projet, 2 réglages, 73 routes UI, 59 transport, 13 tests Tauri Desktop et 127 tests Node passent. `cargo fmt --check` et `git diff --check` passent. |

## Décisions appliquées

- Une politique v1 est lue comme catalogue hérité restrictif: aucune création par `location_id`.
- Un catalogue v2 porte des `workspace` et `exact_project` canoniques, non imbriqués, avec identifiants et défaut de création contrôlés.
- Le futur parcours Desktop étend `targetDialog` de SPEC-081 au lieu de créer un nouveau sélecteur de source.
- Le panneau ouvert depuis Bridget Desktop porte désormais le libellé de la source confirmée dans son URL locale, puis le réaffiche avant le choix d'emplacement. Les sources indisponibles restent visibles dans le sélecteur mais sont désactivées.
- Les projets de même identifiant local sont distingués par la paire `(source_id, project_id)` dans le Desktop. Le relais ne reçoit que la source confirmée.

## Limite restante de validation

- Le parcours visuel manuel de `quickstart.md` reste à valider par l'opérateur sur deux sources réelles avant la clôture de la SPEC. Aucun résultat de test automatisé ne le remplace.
- Les fichiers modifiés par la SPEC sont formatés. `cargo fmt --check` et `git diff --check` sont verts sur la tête validée.

## Parcours de scénarios et migration sans effet

| Scénario Gherkin | Preuve exécutée |
|---|---|
| Création depuis un workspace explicite | `project_workspace::tests::spec_084_previsualisation_v2_limite_creation_import_et_collisions` - succès. |
| `exact_project` non créable | `project_workspace::tests::spec_084_previsualisation_v2_refuse_lien_symbolique_hors_workspace_et_systeme` - succès. |
| Politique v1 restrictive | `project_policy::tests::spec_084_catalogue_v2_separe_creation_import_et_compatibilite_v1` - succès. |
| Deux sources, même identifiant local | `spec_084_les_projets_homonymes_restent_scopes_par_source` - succès dans `desktop_commands`. |

La politique active `/home/moi/.config/bridget/project-root-policy.json` a été copiée sans écriture vers `/tmp/spec084-project-root-policy-copy.json`. Les deux empreintes SHA-256 sont identiques : `dd715eea4d0e3a3c81fca71c38e79820832f01e5d93d0da82dfcd224f682d5e3`. La prévisualisation v2 inventorie les projets hérités sans modifier les liaisons, comme le prouve `ui::tests::spec_084_inventaire_legacy_associe_les_projets_sans_les_modifier`.
