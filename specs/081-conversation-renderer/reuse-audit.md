# Audit de reutilisation de l'existant - SPEC-081 Conversation structurée et rendu technique sûr

## Decision

Statut: PASS
Date: 2026-08-31
Feature dir: `/tmp/bridget-project-nav.JNWHqE/specs/081-conversation-renderer`

Conclusion courte: le plan étend les deux autorités qui existent déjà : la
projection du journal dans `projectTimeline` et les préférences locales du
client. Il réutilise également `ProjectRootPolicy` pour la seule lecture de
fichier prévue. Aucun endpoint de preview, rendu de code ou modèle de tour
équivalent n'existe sous un autre nom. La route d'aperçu reste nouvelle mais
ne duplique pas un explorateur, un shell ou un service de fichiers.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 10 |
| Items audites | 10 |
| Reutilisations deja prevues | 6 |
| Existants potentiellement pertinents | 2 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 7 |
| Specs existantes applicables | 4 |

## Reutilisations correctement identifiees

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

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Aperçu de fichier relayé | Import de projets sous `ProjectRootPolicy` | `crates/bridget-daemon/src/project_workspace.rs:89` | Réutiliser seulement la vérification de racine. Ne pas détourner l'import, qui a une responsabilité de mutation de projet. |
| Synchronisation de préférences vers panneau | Boucle de notification native | `apps/bridget-desktop/src-tauri/src/lib.rs:100` | Réutiliser l'émission ciblée ou l'évaluation de script, mais ne pas étendre les permissions Tauri du panneau externe. |

## Duplications evidentes

Aucune duplication évidente détectée.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `AGENTS.md` | Français, aucun tiret cadratin, code serveur direct et préservation des changements tiers. | Artefacts et communication en français ; aucun écrasement d'UI ou de branche récente. |
| `AGENTS.md` | Diagnostiquer avant correction et tests explicites. | Projection, URL, préférences et route ont chacune des preuves ciblées avant rendu final. |
| `AGENTS.md` | Préférences locales ne traversent pas le tunnel. | Snapshot Tauri lecture seule et fallback navigateur local. |
| `.specify/memory/constitution.md` | Réemploi, sécurité, accessibilité et décision durable. | ADR-022, notices MIT, DOMPurify conservé et contrôles clavier. |
| `.specify/memory/standards.md` | Recherche et baselines de sécurité/test. | Colorateur sous licence vérifiée, tests hostile Markdown et contrôle vendor. |
| `/Users/moi/.speckit/ref/standards-tests.md` | Tests observables et échecs nommés. | Tests purs, Rust, Tauri, Node et quickstart séparés. |
| `/Users/moi/.speckit/research/06-security-compliance.md` | Moindre privilège et défaut sûr. | Aucune capability d'écriture dans le panneau relayé et trois booléens à `false` par défaut. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC-069 | Remise, dédoublonnage et défilement honnête du fil. | Le nouveau rendu conserve les identités de message et la position de lecture. |
| SPEC-074 | Coque Tauri, tunnel SSH, WebView relayé isolé. | Les préférences natives ne sont pas transformées en permission du relais. |
| SPEC-076 | Politique de racines et cycle de vie projet. | Seule cette politique décide qu'un chemin serveur est prévisualisable. |
| SPEC-080 | Centre de contrôle, préférences locales, bordure Mac/serveur. | La catégorie sécurité est ajoutée au même parcours sans serveur de préférences parallèle. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg projectTimeline|renderMessageMarkdown|preferences` | `crates/bridget-daemon/assets/ui/app.js` | Projection, Markdown assaini et store local présents. |
| `rg PreferencesStore|WebviewBuilder|preferences_save` | `apps/bridget-desktop/src-tauri/src/` | Store local atomique et panneau externe isolé présents. |
| `rg ProjectRootPolicy|file-preview|content/file` | `crates/bridget-daemon/src/` | Politique de racines présente, aucun preview de fichier existant. |
| `rg marked|DOMPurify|SHA256` | `assets/ui/` | Vendor, licences et hashes déjà structurés. |
| `rg` sur `specs/069`, `074`, `076`, `080` | `specs/` | Dépendances fonctionnelles et limites existantes vérifiées. |
| Lecture T3 Code | `/Users/moi/11.Repositories/t3code` | Timeline, ChatMarkdown, licence MIT et dépendances relevées. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Tour de conversation | reutiliser etendre | `projectTimeline` est déjà l'autorité factuelle ; la nouveauté est une projection de présentation. | 2026-08-31 |
| Préférences de sécurité | reutiliser etendre | Les deux stores existants deviennent V2 ; pas de stockage alternatif ni de transfert serveur. | 2026-08-31 |
| Fichier lié par agent | creer nouveau | Aucun endpoint de lecture bornée n'existe. La route nouvelle s'appuie exclusivement sur `ProjectRootPolicy`. | 2026-08-31 |
| Coloration de code | creer nouveau sous vendor existant | Aucun moteur statique n'est déjà embarqué. Il sera ajouté seulement avec licence et somme de contrôle. | 2026-08-31 |
| Réemploi T3 | reutiliser principes et helpers ciblés | React et LegendList ne sont pas compatibles avec l'UI statique ; toute reprise directe sera attribuée. | 2026-08-31 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
