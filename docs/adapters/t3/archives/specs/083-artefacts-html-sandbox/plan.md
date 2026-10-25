# Archives techniques de l’adaptateur T3

Document d’origine : [specs/083-artefacts-html-sandbox/plan.md](../../../../../../specs/083-artefacts-html-sandbox/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Élément | Décision confirmée |
|---|---|
| Coque Desktop | Tauri 2 et WKWebView sur macOS, sans embarquer Chromium ni Chrome |
| Browser | enfant WebView dédié, profil local propre et isolé des navigateurs personnels |
| HTML non fiable | document local enveloppé, iframe `sandbox="allow-scripts"`, sans `allow-same-origin`, réseau et navigation bloqués |
| Autorité de données | Bridget daemon et magasin canonique de SPEC-082 |
| Communication | contrat de messages borné, validé et non privilégié entre cadre et coque de rendu |
| Panneau droit | onglets Browser, Artefacts, Fichiers, Liens, Activité ; portée projet par défaut |
| Contrôles de disposition | reprise des deux contrôles MIT de T3 : `Maximize2Icon` / `Minimize2Icon` et `PanelRightIcon` dans `PanelLayoutControls.tsx` |
| Navigation | geste humain explicite, cible validée, ouverte dans Browser Bridget et jamais par le renderer Markdown |
| Persistance Browser | profil local séparé, effacement explicite via l'API Tauri ; jamais lu par un agent ni exporté vers le daemon |
| Réglages | paramètres globaux de l'application, portée visible `Ce Mac` |
| Tests | Rust Desktop, tests de renderer local, tests de contrats, recette macOS de confinement et navigation |

<a id="extrait-2"></a>

## Extrait 2

- Étendre `apps/bridget-desktop/src-tauri/src/lib.rs` et
  `apps/bridget-desktop/src-tauri/src/panels.rs` : les enfants WebView, le
  registre de panneau et l'interception `bridget-open:` existent déjà. La
  destination ne devra plus lancer le navigateur système : elle sera remise au
  Browser du panneau droit par ce registre unique.
- Étendre `apps/bridget-desktop/src-tauri/src/preferences_store.rs` plutôt que
  créer un second magasin de réglages pour état du panneau, profil Browser et
  politique locale.
- Réutiliser `projectTimeline` et ses projections de tours dans
  `crates/bridget-daemon/assets/ui/app.js` : une référence d'artefact est une
  entrée de présentation dérivée, pas une seconde conversation.
- Réutiliser le registre et les routes de lecture de SPEC-082. Le panneau ne
  maintient ni index de sources ni cache parallèle.
- Reprendre la forme des contrôles MIT de T3, observée dans
  `/Users/moi/11.Repositories/t3code/apps/web/src/components/chat/PanelLayoutControls.tsx` :
  même paire icône état agrandi/restauré et bouton de panneau droit, adaptée au
  système de composants Bridget, sans copier le composant ou son CSS.

<a id="extrait-3"></a>

## Extrait 3

1. Étendre le registre de panneau droit existant et ses états persistés dans le
   store de préférences existant, sans créer une seconde régie de WebViews.
2. Créer les WebViews enfants distincts `browser-*` et `artifact-frame-*` avec
   labels, capacités et sources de données séparés.
3. Remplacer le lancement externe existant pour `bridget-open:` par une remise
   typée au Browser interne, après validation de destination.
4. Ajouter le contrôle agrandir/restaurer et le toggle de panneau selon les
   sources T3 identifiées, avec clavier et libellés accessibles.

<a id="extrait-4"></a>

## Extrait 4

1. Ajouter des fixtures malveillantes : fetch, WebSocket, iframe, popup,
   formulaire, tentative de parent/top, URL `file:`, contenu très haut et
   message trop volumineux.
2. Ajouter les tests Desktop de navigation, absence de capability et effacement
   de données Browser, ainsi que les tests du renderer de conversation.
3. Exécuter la recette macOS sur pages HTTPS, contenu local publié, artefact
   offline, lien de source, version enfant et panneau masqué/rétabli.
4. Vérifier la licence de tout code ou icône repris comme forme de T3, et
   consigner les références exactes sans copier de fichiers non nécessaires.
