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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/081-conversation-renderer/reuse-audit.md#extrait-1).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/081-conversation-renderer/reuse-audit.md#extrait-2).

## Arbitrages

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/081-conversation-renderer/reuse-audit.md#extrait-3).

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
