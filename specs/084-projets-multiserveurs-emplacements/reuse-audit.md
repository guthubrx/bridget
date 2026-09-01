# Audit de réutilisation - SPEC-084

## Statut

PASS - Les exigences de SPEC-084 étendent les composants déjà autoritaires. Aucun registre global, service de synchronisation ou second parcours Desktop n'est nécessaire.

## Périmètre audité

- `crates/bridget-daemon/src/project_policy.rs`
- `crates/bridget-daemon/src/project_workspace.rs`
- `crates/bridget-daemon/src/control_settings.rs`
- `crates/bridget-daemon/src/ui.rs`
- `crates/bridget-transport/src/protocol.rs`
- `apps/bridget-desktop/ui/fleet-app.js`
- les artefacts livrés de SPEC-080 et SPEC-081

## Inventaire et décision par exigence

| Exigence | Preuve existante | Décision |
|---|---|---|
| Identité locale d'un projet | Registre projet local dans `store.rs` depuis SPEC-065 | Réutiliser. La portée de `project_id` reste le daemon qui le détient. |
| Composition de plusieurs serveurs | `fleet_snapshot`, `source_id` et `panel_open` dans `fleet-app.js` | Réutiliser. L'identité de vue reste `(source_id, project_id)`. |
| Choix explicite de la source | `requestPanelAction` ouvre déjà `targetDialog` et conserve une source active compatible | Étendre. Le sélecteur SPEC-081 existe déjà: ne pas en créer un second. Ajouter seulement l'affichage durable de la source, le changement dans le dialogue projet et l'invalidation de l'aperçu. |
| Politique de chemins | `ProjectRootPolicy` v1, génération, écritures atomiques et reçu idempotent | Étendre vers le catalogue v2 dans le même document. Ne pas créer de second validateur ou stockage. |
| Création et import | `ProjectPreview::create` et `ProjectPreview::import` dans `project_workspace.rs` | Étendre avec `location_id` et des validateurs distincts création/import. |
| Administration | `control_settings.rs`, preview/apply sous génération | Étendre le descripteur existant vers un catalogue structuré. |
| Relais UI local | Routes projet dans `ui.rs` et contrats `protocol.rs` | Étendre par charges v2 versionnées, en gardant la lecture v1 bornée. |

## Lacunes démontrées

1. `ProjectRootPolicyDocument` est contractuellement v1 et ne contient que `allowed_project_roots`.
2. `validate_requested_root` accepte tout descendant d'une racine et est utilisé par création comme import.
3. Un chemin de checkout peut donc être traité comme parent de création alors qu'il doit seulement être importable exactement.
4. Le dialogue de sélection de source existe, mais il ne contient ni catalogue d'emplacements, ni génération de catalogue, ni prévisualisation liée à cette sélection.
5. Le centre de contrôle ne connaît qu'une valeur plate `project_roots.allowed_roots`.

## Décisions de réutilisation

### EXTEND

- `project_policy.rs`: document v2, compatibilité restrictive v1, capacités et validation canonique unique.
- `project_workspace.rs`: prévisualisation/apply déterminés par `location_id` côté daemon.
- `control_settings.rs`: preview/apply atomique du catalogue, sur le même fichier de politique.
- `protocol.rs` et `ui.rs`: contrats/routes v2 sans casser les lectures historiques.
- `fleet-app.js`: enrichissement du sélecteur de source SPEC-081, sans remplacement du dialogue ni nouvelle autorité distante.

### REUSE

- profils, tunnels, statut de connexion et capacités de source;
- registre local des projets et ses générations;
- écriture atomique et reçu de mutation;
- relais Tauri vers la source confirmée;
- diagnostic Git non mutateur de `project_workspace.rs`.

### À NE PAS CRÉER

- base centrale de projets;
- réplication de dépôts entre serveurs;
- scan global de système de fichiers;
- catalogue dupliqué dans Bridget Desktop;
- nouveau sélecteur de source en parallèle de `targetDialog`.

## Compatibilité et migration

- Un document v1 reste lisible mais ses entrées sont projetées comme `exact_project` héritées: import du chemin exact seulement, jamais création descendante.
- Une promotion en `workspace` est une mutation v2 explicite avec prévisualisation, génération et confirmation.
- Le retrait d'un emplacement ne modifie ni ne rompt une liaison projet persistée.
- L'aperçu de migration inventorie les projets liés sous les racines v1 sans les modifier.

## Ajustement appliqué au plan de travail

Le lot Desktop ne refond plus le mécanisme de choix de source. Il réutilise le `targetDialog` livré par SPEC-081, le rend visible dans le dialogue de création/import et lui associe le catalogue d'emplacements. La portée fonctionnelle ne change pas; cette précision évite la duplication d'interface et de routage.

## Conclusion

La voie conforme est une évolution des autorités existantes. Elle maintient le daemon comme autorité locale, Bridget Desktop comme composition de sources et le catalogue comme unique source de vérité par serveur.
