# Vendor UI — Markdown + assainissement

Embarqué pour servir hors-ligne via le relais local (pas de CDN).

| Fichier | Paquet | Version | Licence | Taille |
|---|---|---|---|---|
| `marked.min.js` | marked | 15.0.12 | MIT | ~39 Ko |
| `purify.min.js` | DOMPurify | 3.2.6 | MPL-2.0 OR Apache-2.0 | ~22 Ko |

Licences complètes : `LICENSE.marked.md`, `LICENSE.dompurify.txt`.

Pourquoi ce couple : marked transforme le Markdown en HTML (GFM, tableaux) ;
DOMPurify assainit le HTML avant insertion. marked seul = trou XSS.
Total JS ajouté : ~62 Ko (hors licences).
