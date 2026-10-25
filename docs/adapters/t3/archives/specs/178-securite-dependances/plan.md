# Archives techniques de l’adaptateur T3

Document d’origine : [specs/178-securite-dependances/plan.md](../../../../../../specs/178-securite-dependances/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Protection WebView : T3 BrowserDocumentFrame.tsx et HtmlRenderFrame.tsx lus. T3 isole les documents en iframe, avec origine opaque ; pas de garde pour moteur Android système. Bridget conserve son rendu statique plus strict et ajoute une garde avant construction du WebView, partagée par messages et fichiers. Baseline officielle WebView Stable156.0.8078.48 relevée sur ChromiumDash le25octobre ; les versions inconnues/anciennes restent lisibles en code source. Le paquet moteur est relu à la reprise de l’écran, pour permettre la mise à jour GooglePlay sans figer le refus. La mise à jour du moteur reste externe ; le risque de rendu est neutralisé dans Bridget.
