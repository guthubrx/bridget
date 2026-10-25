# Archives techniques de l’adaptateur T3

Document d’origine : [specs/083-artefacts-html-sandbox/tasks.md](../../../../../../specs/083-artefacts-html-sandbox/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [x] T020 [US3] Étendre `apps/bridget-desktop/src-tauri/src/panels.rs` et `lib.rs` pour créer le WebView `browser-*` sans capability Tauri, avec profil séparé, navigation contrôlée et réutilisation de la disposition existante.
- [x] T021 [US3] Implémenter dans `apps/bridget-desktop/src-tauri/src/lib.rs` la remise typée de `bridget-open:` au Browser interne au lieu de `Command::new("open")`, après validation HTTPS ou référence publiée.
- [x] T022 [US3] Ajouter l'effacement explicite des données Browser dans `apps/bridget-desktop/src-tauri/src/lib.rs` et `preferences_store.rs`, via l'API WebView puis incrément de génération de profil.
- [x] T023 [US3] Ajouter dans `crates/bridget-daemon/assets/ui/app.js` les boutons de disposition en reprenant la forme et les états accessibles de `PanelRightIcon`, `Maximize2Icon` et `Minimize2Icon` documentés depuis T3, sans copier son composant.
- [x] T024 [US3] Ajouter dans `crates/bridget-daemon/assets/ui/theme.css` les états panneau masqué/visible/maximisé et le focus clavier, sans détruire la conversation ou le Browser lors d'un toggle.
- [x] T025 [US3] Écrire les tests Desktop de navigation, URL interdite, redirection, ouverture locale, persistance de panneau et effacement de profil dans `apps/bridget-desktop/src-tauri/tests/browser_panel_test.rs`.

<a id="extrait-2"></a>

## Extrait 2

- [x] T035 Vérifier les capabilities finales et leur absence de privilège dans `apps/bridget-desktop/src-tauri/capabilities/` avec un test négatif par label de WebView.
- [x] T036 Vérifier les licences, attribution et limites de réutilisation des contrôles T3 dans `docs/decisions/023-artefacts-structures-et-sandbox.md`, sans importer de code T3 inutile.
- [x] T037 Jouer la recette `specs/083-artefacts-html-sandbox/quickstart.md` sur macOS et instance distante, puis consigner les résultats dans `specs/083-artefacts-html-sandbox/implementation.md`.
- [x] T038 Documenter l'exclusion explicite des actions DOM et d'automatisation agent dans `docs/decisions/023-artefacts-structures-et-sandbox.md` afin qu'aucune évolution ne les introduise par défaut.
- [x] T039 Raccorder les agents Codex gérés au serveur MCP Bridget déclaré statiquement avant `app-server` dans `crates/bridget-daemon/src/wrapper.rs`, puis vérifier que le catalogue fournisseur contient `bridget_publish_artifact` et son contrat `kind: html`.
