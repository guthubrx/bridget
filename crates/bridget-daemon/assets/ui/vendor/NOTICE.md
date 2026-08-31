# Vendor UI — Markdown + assainissement

Embarqué pour servir hors-ligne via le relais local (pas de CDN).

| Fichier | Paquet | Version | Licence | Taille |
|---|---|---|---|---|
| `marked.min.js` | marked | 15.0.12 | MIT | ~39 Ko |
| `purify.min.js` | DOMPurify | 3.2.6 | MPL-2.0 OR Apache-2.0 | ~22 Ko |
| `highlight.min.js` | highlight.js | 11.12.0 | BSD-3-Clause | ~126 Ko |
| `highlight-github-*.min.css` | highlight.js | 11.12.0 | BSD-3-Clause | ~3 Ko |

Licences complètes : `LICENSE.marked.md`, `LICENSE.dompurify.txt`,
`LICENSE.highlightjs.txt` et `LICENSE.t3code.MIT.txt`.

Pourquoi ce couple : marked transforme le Markdown en HTML (GFM, tableaux) ;
DOMPurify assainit le HTML avant insertion. marked seul = trou XSS.
`highlight.js` est chargé localement et ne reçoit aucune donnée réseau. Son
absence laisse le code lisible en texte brut.

Réemploi T3 Code : les portions explicitement adaptées sont consignées dans
`specs/081-conversation-renderer/implementation.md`. Le code source T3 Code
est sous licence MIT, copyright 2026 T3 Tools Inc.

Total JS ajouté : ~188 Ko (hors licences).
