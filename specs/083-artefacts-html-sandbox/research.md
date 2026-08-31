# Recherche - SPEC-083 Artefacts HTML sandboxés et navigateur latéral

**Date**: 2026-08-31

## Questions et décisions

| Question | Décision | Motif et source |
|---|---|---|
| Faut-il un moteur Chromium embarqué ? | Non | Tauri sur macOS s'appuie sur WKWebView. La documentation Tauri expose directement les WebViews enfants et leurs options, sans dépendance Chrome. [Tauri WebviewBuilder](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewBuilder.html) |
| Peut-on persister puis purger un Browser dédié ? | Oui, avec un profil WebView séparé et un effacement depuis la coque | Tauri documente `data_directory` selon plate-forme, `data_store_identifier` pour WKWebView récent et `clear_all_browsing_data`. La compatibilité macOS doit être testée. [WebviewBuilder](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewBuilder.html), [Webview](https://docs.rs/tauri/latest/tauri/webview/struct.Webview.html) |
| Le HTML peut-il avoir `allow-scripts` et `allow-same-origin` ? | Non | MDN explique que cette combinaison sur contenu de même origine permet à un cadre de retirer sa sandbox. La sandbox doit rester à origine opaque. [MDN iframe sandbox](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/iframe) |
| Peut-on laisser le HTML contacter les sources ? | Non | Toute collecte doit être passée à Bridget afin d'appliquer provenance et règles SSRF. OWASP recommande validation de destination et contrôle des redirections. [OWASP SSRF](https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html) |
| Le Browser partage-t-il ses données avec Safari, Chrome ou l'agent ? | Non | Le profil WebView est dédié à Bridget Desktop et n'est jamais exposé par le contrat agent. Le principe de moindre privilège Tauri impose des scopes explicites. [Tauri permissions](https://v2.tauri.app/learn/security/using-plugin-permissions/) |
| Comment servir un HTML publié ? | route locale à ticket court, pas `file:` et pas URL directe arbitraire | Le document de confiance construit la sandbox autour d'octets canoniques et évite d'accorder des capacités de fichier ou d'origine partagée. [Tauri CSP](https://v2.tauri.app/security/csp/) |
| Quels boutons reprendre de T3 ? | le toggle de panneau droit et l'agrandir/restaurer | T3 utilise `PanelRightIcon`, `Maximize2Icon` et `Minimize2Icon`, avec `Toggle`, infobulle et état accessible. Source MIT inspectée : `/Users/moi/11.Repositories/t3code/apps/web/src/components/chat/PanelLayoutControls.tsx`. |
| T3 rend-il un fichier HTML généré inline ? | Non | T3 reconnait HTML comme prévisualisable dans Browser, pas comme Markdown exécuté en ligne. Sources inspectées : `/Users/moi/11.Repositories/t3code/apps/web/src/browser/openFileInPreview.ts`, `/Users/moi/11.Repositories/t3code/packages/shared/src/filePreview.ts`, `/Users/moi/11.Repositories/t3code/apps/web/src/components/ChatMarkdown.tsx`. |

## Enseignements utiles

1. Un Browser WebView est bien un navigateur utilisable, mais il n'est pas un
   nouveau privilège. Son profil peut contenir des sessions de l'opérateur ; il
   ne peut donc jamais devenir la surface de collecte automatique de l'agent.
2. Une iframe n'est pas une frontière complète par elle-même. La combinaison
   origine opaque, CSP envoyée par l'hôte, absence de bridge Tauri, tickets
   courts et validation de messages limite les chemins de sortie.
3. La CSP ne doit pas seulement être déclarée dans la page principale. Elle est
   fixe et contrôlée par Bridget pour chaque enveloppe de sandbox, sans fusion
   avec une CSP réclamée par l'artefact.
4. Les contrôles observés dans T3 sont une bonne référence ergonomique, pas une
   raison de copier un composant React dans l'UI JavaScript de Bridget. Leur
   comportement et leurs icônes sont les éléments à réutiliser.
5. Les navigateurs et artefacts liés à des conversations se retrouvent par les
   reçus et l'index canonique de SPEC-082. Il n'est pas nécessaire de scanner
   les chemins utilisateur ni de stocker des doublons dans le Browser.

## Risques à vérifier pendant l'implémentation

| Risque | Mesure prévue |
|---|---|
| `data_directory` non disponible en WKWebView sur macOS cible | test d'intégration de `data_store_identifier`, puis repli documenté ou contrainte de version macOS avant livraison |
| Insertion HTML qui casse l'enveloppe par `</script>` | transmettre les octets encodés et décoder dans un bootstrap de confiance, jamais interpoler du HTML brut dans une balise script |
| `postMessage` utilisé comme IPC implicite | schéma de message fermé, taille max, origine/instance validées, aucune commande native dérivée de texte libre |
| Redirection vers protocole dangereux | navigateur accepte HTTPS, route locale Bridget et cibles explicitement autorisées seulement ; intercept avant ouverture |
| État interactif très volumineux | plafond contractuel et création de version seulement après validation Bridget |
| Profil Browser confondu avec cache d'artefacts | répertoires et opérations de purge distincts ; l'effacement Browser ne touche jamais le magasin canonique |

## Baseline de sécurité

- Content Security Policy restrictive, aucune dépendance distante ni CDN.
- Capabilities Tauri réduites par label de WebView et testées négativement.
- Validation côté daemon des sources externes, y compris destinations et
  redirections, avant téléchargement ou restauration.
- Les actions Browser suivantes restent hors scope : sélection DOM assistée,
  clic ou saisie par agent, formulaires, téléchargement et publication.
