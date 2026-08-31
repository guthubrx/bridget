# Plan d'implémentation - SPEC-083 Artefacts HTML sandboxés et navigateur latéral

**Branche**: `082-artifact-publication` | **Date**: 2026-08-31 | **Spec**: `specs/083-artefacts-html-sandbox/spec.md`
**Dépendance d'implémentation**: SPEC-082 doit fournir le registre, les versions, les blobs et les reçus d'artefact.

## Résumé

Cette spécification ajoute deux surfaces distinctes dans Bridget Desktop :

1. un renderer HTML/JavaScript non fiable, exécuté dans un cadre isolé et
   strictement sans réseau ni privilège local ;
2. un panneau droit d'opérateur, contenant un navigateur Web normal mais isolé
   de Safari/Chrome et des agents.

Le point déterminant est la séparation des autorités. Le navigateur est une
surface de consultation humaine. L'artefact est du contenu publié non fiable.
Le réseau de collecte, les sources, les restaurations et les nouvelles versions
restent des opérations de Bridget, journaliées et soumises aux politiques de
SPEC-082. Aucune page, aucun iframe, aucun script d'artefact ne reçoit une
capacité Tauri, un cookie de navigateur, un chemin local ou un accès direct à
Bridget.

## Contexte technique

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

## Architecture et frontières

~~~text
Agent
  │ bridget_publish_artifact({ kind: "html", ... })
  ▼
Bridget daemon - validation, provenance, versions, blobs canoniques
  │ reçu attesté et routes locales à ticket court
  ├────────────────────────────────────────────────────────────┐
  ▼                                                            ▼
Conversation relayée                                    Panneau droit
  │                                                            │
  ▼                                                            ├─ Browser WebView : navigation humaine HTTPS,
Renderer HTML isolé                                       │  profil séparé, aucun droit agent
  │ iframe sandbox allow-scripts                             ├─ Artefacts, Fichiers, Liens, Activité : projections
  │ CSP sans réseau, sans navigation, sans API Tauri          │  du registre Bridget
  ▼                                                            ▼
Données déclarées de la version                         contrôle Desktop typé
~~~

### Ce qui est autorisé et ce qui ne l'est pas

| Surface | Autorisé | Interdit |
|---|---|---|
| Artefact HTML | calcul local, filtres, événements locaux, rendu de ses données injectées | réseau, fichiers, cookies Browser, Tauri IPC, navigation, popups, formulaires, téléchargements, accès parent |
| Conversation | afficher le reçu et proposer une ouverture | prévisualisation réseau ou exécution HTML hors sandbox |
| Browser | navigation volontaire de l'opérateur, contenu publié et pages HTTPS | accès agent, lecture/export de cookies, automatisation ou actions DOM par agent dans ce lot |
| Bridget daemon | collecter, restaurer, vérifier, journaliser, publier une nouvelle version | fournir une capacité générale au code HTML ou au navigateur |

## Réutilisation de l'existant

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

## Décisions techniques

### Sandbox d'artefact

Le daemon conserve le document, les données et le manifeste tels qu'ils ont été
publiés. La coque de conversation les sert par un ticket local à usage limité,
dans un document enveloppe de confiance. Cet enveloppe crée un iframe avec
`sandbox="allow-scripts"`, sans `allow-same-origin`, `allow-forms`,
`allow-popups`, `allow-downloads` ou permission de navigation. Il applique une
CSP fixe qui refuse tout par défaut, notamment `connect-src 'none'`, les images
distantes, les frames, les formulaires et les bases URL.

L'artefact ne parle jamais à Tauri. Son seul canal facultatif est un protocole
de messages local, versionné, limité à : hauteur de rendu bornée, état
interactif sérialisable borné et demande d'ouverture d'une référence de source.
La coque revalide chaque message. Une ouverture est seulement proposée à
l'opérateur et, après son geste, transmise à Bridget. Enregistrer un état passe
par Bridget et crée une version enfant explicite, conformément à SPEC-082.

### Browser de l'opérateur

Le Browser est un enfant WebView sans capability applicative et sans bridge
Tauri injecté. Sa zone de données est dédiée au profil Browser de Bridget
Desktop. Sur macOS, l'implémentation doit utiliser l'identifiant de data store
Tauri/WKWebView disponible sur la version cible, et documenter le repli si la
plate-forme n'offre pas de répertoire de données. L'effacement invoque
`clear_all_browsing_data` depuis la coque de confiance puis régénère le profil
si nécessaire. Les cookies restent dans ce profil et ne traversent aucun IPC.

Les pages HTML publiées sont ouvertes via une route locale validée, pas via une
URL `file:`. Les pages Internet requièrent HTTPS et une navigation opérateur.
La récupération automatique de contenu externe reste désactivée par défaut ;
si l'opérateur l'active dans les réglages `Ce Mac`, Bridget collecte puis remet
le résultat attesté au renderer, qui ne récupère jamais lui-même l'URL.
Les redirections, nouvelles fenêtres et téléchargements passent par un
intercepteur Desktop qui demande ou applique la politique explicite. La
sélection de DOM et toute action d'agent dans Browser sont hors périmètre.

### Panneau droit et contrôles

Le bouton `PanelRightIcon` affiche ou masque le panneau. Le bouton
`Maximize2Icon` devient `Minimize2Icon` lorsque la surface droite est agrandie,
sans fermer l'état de conversation ou de navigation. Les deux ont infobulle,
libellé accessible et état `aria-pressed`. Aucun raccourci Command-K n'est
introduit. Le panneau s'ouvre avec la portée du projet actif ; une recherche
globale reste une action volontaire de l'opérateur.

## Modèle et contrats

Les entités détaillées sont dans `data-model.md`. Les échanges sont fixés dans :

- `contracts/sandbox-runtime-v1.md` pour le document isolé et ses messages ;
- `contracts/browser-panel-v1.md` pour l'ouverture humaine, les onglets et
  l'effacement du Browser.

Tous les objets référencent `artifact_id` et `version_id` de SPEC-082. Aucun
artefact HTML ne peut exister hors de ce cycle de publication.

## Plan par lots

### Lot 1 - Frontières Desktop et réglages

1. Étendre le registre de panneau droit existant et ses états persistés dans le
   store de préférences existant, sans créer une seconde régie de WebViews.
2. Créer les WebViews enfants distincts `browser-*` et `artifact-frame-*` avec
   labels, capacités et sources de données séparés.
3. Remplacer le lancement externe existant pour `bridget-open:` par une remise
   typée au Browser interne, après validation de destination.
4. Ajouter le contrôle agrandir/restaurer et le toggle de panneau selon les
   sources T3 identifiées, avec clavier et libellés accessibles.

### Lot 2 - Runtime HTML sandboxé

1. Ajouter la route locale à ticket court qui construit l'enveloppe de sandbox
   à partir d'une version canonique de SPEC-082.
2. Appliquer l'iframe sandbox et la CSP fixe, puis refuser toute option
   d'assouplissement fournie par l'artefact.
3. Implémenter le protocole de messages borné et validé, sans exposer d'objet
   global Bridget ou Tauri.
4. Ajouter les états d'erreur explicites, les détails de manifeste et les
   actions consulter, copier, exporter, restaurer et ouvrir une source.

### Lot 3 - Browser, contenus liés et recherche

1. Ajouter Browser avec navigation humaine HTTPS, historique local visible,
   interception des nouvelles fenêtres et effacement explicite.
2. Ajouter les onglets Artefacts, Fichiers, Liens et Activité comme projections
   du registre et du journal Bridget, sans scanner le système de fichiers.
3. Ajouter l'ouverture des contenus publiés et liens depuis la conversation
   vers le Browser, sans fetch automatique.
4. Exposer les réglages locaux Browser et sécurité sous Paramètres de
   l'application, explicitement étiquetés `Ce Mac`, avec récupération externe
   désactivée par défaut et activation qui reste relayée par Bridget.

### Lot 4 - Preuves de sécurité et recette

1. Ajouter des fixtures malveillantes : fetch, WebSocket, iframe, popup,
   formulaire, tentative de parent/top, URL `file:`, contenu très haut et
   message trop volumineux.
2. Ajouter les tests Desktop de navigation, absence de capability et effacement
   de données Browser, ainsi que les tests du renderer de conversation.
3. Exécuter la recette macOS sur pages HTTPS, contenu local publié, artefact
   offline, lien de source, version enfant et panneau masqué/rétabli.
4. Vérifier la licence de tout code ou icône repris comme forme de T3, et
   consigner les références exactes sans copier de fichiers non nécessaires.

## Structure projet envisagée

~~~text
apps/bridget-desktop/src-tauri/src/
├── lib.rs                         # extension contrôlée des WebViews et bridget-open
├── panels.rs                       # extension du registre et de la disposition uniques
├── artifact_sandbox.rs            # nouveau, enveloppe, tickets et politique immuable
├── preferences_store.rs           # extension du store atomique existant
└── capabilities/
    ├── browser-panel.json         # nouveau, aucune permission métier/daemon
    └── artifact-frame.json        # nouveau, strictement sans capability

crates/bridget-daemon/
├── src/artifact_store.rs          # fourni par SPEC-082, réutilisé ici
├── src/ui.rs                      # routes locales bornées et projections de panneau
└── assets/ui/
    ├── app.js                     # rendu inline, panneau et références de tour
    ├── artifact-sandbox-host.js   # nouveau, hôte de cadre sans privilège
    └── theme.css                  # états visuels et accessibilité
~~~

## Complexité et performances

- Une navigation est O(1) côté coque plus le coût WebKit du chargement. Elle
  ne déclenche pas de collecte daemon sans action explicite.
- La hauteur inline est bornée à 1 200 px. Les messages de redimensionnement
  sont plafonnés, coalescés par frame et ignorés hors intervalle autorisé.
- Les onglets utilisent les index de SPEC-082, donc pagination SQL et rendu
  progressif, jamais un balayage de blobs ou de disque.
- Browser est conservé quand le panneau est masqué si cela ne dépasse pas la
  politique mémoire. L'artefact développé peut être détruit et reconstruit à
  partir de son manifeste immutable.

## Gate avant implémentation

- [ ] SPEC-082 est implémentée et fournit les contrats de version/blob requis.
- [ ] Les politiques de sandbox ont une preuve de blocage automatisée sur la
  plate-forme macOS supportée.
- [ ] Les choix de data store WKWebView sont validés sur la version macOS cible.
- [ ] Les capabilities de `browser-*` et `artifact-frame-*` sont vérifiées par
  test négatif, pas seulement par revue de configuration.
- [ ] Les conditions de navigation, redirection, téléchargement et effacement
  ont une recette explicite.
