# Archives techniques de l’adaptateur T3

Document d’origine : [specs/081-conversation-renderer/reuse-audit.md](../../../../../../specs/081-conversation-renderer/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Projection de tours | `projectTimeline` | `crates/bridget-daemon/assets/ui/app.js:5177` | Extraire une projection de présentation au-dessus des faits déjà corrélés, sans toucher au journal. |
| Markdown sûr | `sanitizeMessageHtml` et `renderMessageMarkdown` | `crates/bridget-daemon/assets/ui/app.js:5638`, `crates/bridget-daemon/assets/ui/app.js:5669` | Étendre par classification et nœuds contrôlés, jamais en supprimant l'assainissement. |
| Préférences navigateur | Store du centre de contrôle | `crates/bridget-daemon/assets/ui/app.js:5822`, `crates/bridget-daemon/assets/ui/app.js:5851`, `crates/bridget-daemon/assets/ui/app.js:5885` | Version suivante du document existant, pas une nouvelle clé de stockage concurrente. |
| Préférences macOS | `DesktopPreferences` et `PreferencesStore` | `apps/bridget-desktop/src-tauri/src/preferences_store.rs:17`, `apps/bridget-desktop/src-tauri/src/preferences_store.rs:70` | Ajouter les booléens de contenu au store atomique existant. |
| Injection dans le panneau | WebView relayé isolé | `apps/bridget-desktop/src-tauri/src/lib.rs:597` | Ajouter un script d'initialisation lecture seule au WebView, sans modifier sa capability. |
| Fichiers projet | `ProjectRootPolicy` | `crates/bridget-daemon/src/project_policy.rs:50`, `crates/bridget-daemon/src/project_policy.rs:68` | Canonicaliser et comparer aux racines existantes, sans seconde allowlist. |
| Routes relay | Routeur UI tokenisé | `crates/bridget-daemon/src/ui.rs:1369`, `crates/bridget-daemon/src/ui.rs:1381` | Ajouter un GET spécialisé avec les mêmes bornes de corps et d'authentification. |
| Assets UI | Vendor markdown et sommes SHA | `crates/bridget-daemon/assets/ui/vendor/NOTICE.md:1`, `crates/bridget-daemon/assets/ui/vendor/SHA256SUMS:3` | Ajouter le colorateur et la notice T3 dans l'inventaire existant. |

<a id="extrait-2"></a>

## Extrait 2

| Requete | Portee | Resultat |
|---|---|---|
| `rg projectTimeline|renderMessageMarkdown|preferences` | `crates/bridget-daemon/assets/ui/app.js` | Projection, Markdown assaini et store local présents. |
| `rg PreferencesStore|WebviewBuilder|preferences_save` | `apps/bridget-desktop/src-tauri/src/` | Store local atomique et panneau externe isolé présents. |
| `rg ProjectRootPolicy|file-preview|content/file` | `crates/bridget-daemon/src/` | Politique de racines présente, aucun preview de fichier existant. |
| `rg marked|DOMPurify|SHA256` | `assets/ui/` | Vendor, licences et hashes déjà structurés. |
| `rg` sur `specs/069`, `074`, `076`, `080` | `specs/` | Dépendances fonctionnelles et limites existantes vérifiées. |
| Lecture T3 Code | `/Users/moi/11.Repositories/t3code` | Timeline, ChatMarkdown, licence MIT et dépendances relevées. |

<a id="extrait-3"></a>

## Extrait 3

Les choix de ce parcours relèvent du contrat Bridget : identité attestée,
état durable, budgets bornés et vérification avant annonce de réussite.
Les anciennes comparaisons de produits ont été retirées le25octobre2026.
