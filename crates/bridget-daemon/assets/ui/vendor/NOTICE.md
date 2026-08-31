# Vendor UI - Markdown, rendu structuré et assainissement

Embarqué pour servir hors-ligne via le relais local (pas de CDN).

| Fichier | Paquet | Version | Licence | Taille |
|---|---|---|---|---|
| `marked.min.js` | marked | 15.0.12 | MIT | ~39 Ko |
| `purify.min.js` | DOMPurify | 3.2.6 | MPL-2.0 OR Apache-2.0 | ~22 Ko |
| `highlight.min.js` | highlight.js | 11.12.0 | BSD-3-Clause | ~126 Ko |
| `highlight-github-*.min.css` | highlight.js | 11.12.0 | BSD-3-Clause | ~3 Ko |
| `echarts.min.js` | Apache ECharts | 6.1.0 | Apache-2.0 | ~1,1 Mo |
| `tabulator.min.js` | Tabulator | 6.5.2 | MIT | ~0,4 Mo |
| `tabulator.min.css` | Tabulator | 6.5.2 | MIT | ~0,1 Mo |

Licences complètes : `LICENSE.marked.md`, `LICENSE.dompurify.txt`,
`LICENSE.highlightjs.txt`, `LICENSE.echarts.txt`, `NOTICE.echarts.txt`,
`LICENSE.tabulator.txt` et `LICENSE.t3code.MIT.txt`.

Pourquoi ce couple : marked transforme le Markdown en HTML (GFM, tableaux) ;
DOMPurify assainit le HTML avant insertion. marked seul = trou XSS.
`highlight.js` est chargé localement et ne reçoit aucune donnée réseau. Son
absence laisse le code lisible en texte brut.

Apache ECharts rend les graphiques déclaratifs. Son option `aria` est activée
par le renderer Bridget et une table sémantique dérivée reste disponible.
Tabulator n'est chargé que pour les tables qui dépassent le seuil documenté;
la table HTML reste le repli accessible. Ces trois fichiers proviennent des
versions figées dans `../package-lock.json`, sont copiés par
`npm run vendor:sync` et sont vérifiés par `npm run vendor:check`. Leur somme
SHA-256 est portée par `manifest.json`; aucun script ou style n'est servi par
CDN.

Réemploi T3 Code : les portions explicitement adaptées sont consignées dans
`specs/081-conversation-renderer/implementation.md`. Le code source T3 Code
est sous licence MIT, copyright 2026 T3 Tools Inc.

Total JS ajouté : ~1,7 Mo (hors licences).
