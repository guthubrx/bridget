# Archives techniques de l’adaptateur T3

Document d’origine : [specs/083-artefacts-html-sandbox/reuse-audit.md](../../../../../../specs/083-artefacts-html-sandbox/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Requete | Portee | Resultat |
|---|---|---|
| `rg WebviewBuilder|bridget-open|DesktopPreferences|RemoteBrowser` | `apps/bridget-desktop/` | enfants WebView, ouverture externe et préférences existants ; RemoteBrowser non implémenté. |
| `find capabilities -type f` | Desktop capabilities | main privilégié, panneaux relayés sans core Tauri. |
| `rg projectTimeline|renderMessageMarkdown|content_security` | renderer relayé | timeline factuelle et Markdown assaini confirmés. |
| inspection T3 `PanelLayoutControls.tsx` | `/Users/moi/11.Repositories/t3code` | icônes et états exacts pour panneau/agrandissement observés sous MIT. |
