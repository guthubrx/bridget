# Archives techniques de l’adaptateur T3

Document d’origine : [specs/083-artefacts-html-sandbox/research.md](../../../../../../specs/083-artefacts-html-sandbox/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

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

<a id="extrait-2"></a>

## Extrait 2

Les choix de ce parcours relèvent du contrat Bridget : identité attestée,
état durable, budgets bornés et vérification avant annonce de réussite.
Les anciennes comparaisons de produits ont été retirées le25octobre2026.
