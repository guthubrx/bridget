# Audit de réutilisation de l'existant - SPEC-082

## Decision

Statut: PASS
Date: 2026-08-31
Feature dir: `/private/tmp/bridget-project-nav.JNWHqE/specs/082-artefacts-natifs-durables`

Conclusion courte: aucun registre d'artefacts versionnés, sourcés et durables
n'existe dans Bridget. Le plan ne recrée pas les frontières déjà en place : il
étend le store SQLite, le registre MCP, le journal, le renderer de conversation,
les préférences Desktop et les routes relayées. Les objets proches,
notamment `idempotency` et les références de contenu Markdown, n'ont pas la
même responsabilité et restent explicitement séparés.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 10 |
| Items audites | 10 |
| Reutilisations deja prevues | 7 |
| Existants potentiellement pertinents | 3 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 5 |
| Specs existantes applicables | 5 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Métadonnées et migrations d'artefacts | `Store` SQLite | `crates/bridget-daemon/src/store.rs:514`, `:528` | Ajouter les tables additive dans le store existant, sans seconde base. |
| Publication unique | routeur MCP et validation stricte | `crates/bridget-daemon/src/mcp.rs:628`, `:1557` | Ajouter `bridget_publish_artifact` à ce registre unique. |
| Rattachement conversation | projection de tours | `crates/bridget-daemon/assets/ui/app.js:5733` | Référence d'artefact dérivée du journal, aucune nouvelle timeline. |
| Markdown et code ordinaires | renderer assaini existant | `crates/bridget-daemon/assets/ui/app.js:6440` | Les artefacts restent hors Markdown ; Markdown continue via DOMPurify et highlight.js. |
| Réglages cache/contenu | `DesktopPreferences` atomique | `apps/bridget-desktop/src-tauri/src/preferences_store.rs:34`, `:163` | Étendre le document local versionné, sans localStorage autoritaire. |
| Panneau Artefacts | panneaux WebView locaux | `apps/bridget-desktop/src-tauri/src/lib.rs:809` | Étendre le relai et le panneau droit existants. |
| Activité de restauration | journal/activité de travail | `crates/bridget-daemon/src/ui.rs:65`, `crates/bridget-daemon/assets/ui/app.js:5733` | Réutiliser les actes existants plutôt qu'un journal d'artefact parallèle. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Blobs canoniques | `idempotency` stocke des octets de remise | `crates/bridget-daemon/src/idempotency.rs:510` | Garder séparé : remise transport et contenu d'artefact n'ont ni durée ni références communes. Réutiliser seulement les pratiques d'écriture atomique. |
| Cache et répertoire daemon | daemon actuel sous `.cache/bridget` | `crates/bridget-daemon/src/daemon.rs:595`, `:612` | Créer une racine durable distincte pour les blobs canoniques, car un cache ne garantit pas 30 jours, pinning et restauration. |
| Images Markdown | référence chargée sur geste | `crates/bridget-daemon/assets/ui/app.js:6737` | Remplacer le fetch renderer par collecte Bridget seulement pour les artefacts ; conserver le comportement Markdown actuel hors scope. |

## Duplications evidentes

Aucune duplication évidente détectée.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/Users/moi/.speckit/constitution.md` | Réutiliser avant de créer | Store, MCP, projection, préférences et relay sont étendus avec preuves. |
| `/Users/moi/.speckit/constitution.md` | Durabilité et immutabilité | Manifeste/version canonique distincts du cache, restauration en version enfant. |
| `/Users/moi/.speckit/constitution.md` | Sécurité et provenance | Sources vérifiables, collecte Bridget, refus de fait non sourcé. |
| `/private/tmp/bridget-project-nav.JNWHqE/AGENTS.md` | Français, chemins, tests et code Desktop | Contrats et recettes détaillent tests Rust, renderer et Desktop. |
| `/private/tmp/bridget-project-nav.JNWHqE/.specify/memory/standards.md` | Baseline de recherche et licences | ECharts/Tabulator locaux, audit de licence et hashes prévus. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| `specs/081-conversation-renderer` | `projectTimeline`, Markdown assaini, références de contenu, scroll stable | Ajouter une projection d'artefact au-dessus de ces faits sans altérer l'historique. |
| `specs/080-centre-controle-bridget` | préférences globales Desktop et Commande-virgule | Les seuils de cache/rétention rejoignent le centre de contrôle global. |
| `specs/074-bridget-desktop` | relai loopback, panneaux enfants sans permissions | Le panneau Artefacts respecte cette frontière. |
| `specs/072-registre-fournisseurs` | provenance déclarative exacte | Provenance d'artefact est explicite et ne déduit rien du texte. |
| `specs/067-profils-extensions-secrets-projet` | sources et secrets isolés par projet | Visibilité d'artefact et collecte ne contournent pas cette isolation. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg artifact|manifest|blob|provenance|source_ref|restore|cache|retention|quota|mcp` | `crates/`, `apps/`, `specs/` | Aucun registre d'artefacts métier existant ; store, idempotence et provenance fournisseur identifiés. |
| `rg projectTimeline|renderMessageMarkdown|bridget-open` | renderer et Desktop | Projection de tours, rendu Markdown assaini et ouverture encadrée confirmés. |
| `rg DesktopPreferences|preferences_store` | Desktop | Magasin local atomique unique confirmé. |
| `rg CREATE TABLE|init_schema` | daemon | Migration SQLite additive dans `Store` confirmée. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Métadonnées d'artefact | etendre `Store` | Une même base garde cohérence transactionnelle avec projets et journal. | 2026-08-31 |
| Blobs canoniques | creer un magasin dédié | Le cache/idempotence n'offre pas les règles de conservation et références nécessaires. | 2026-08-31 |
| Sources distantes | creer un collecteur métier étroit | Le renderer Markdown ne peut devenir un canal réseau autoritaire. | 2026-08-31 |
| Rendu dans le fil | etendre `projectTimeline` | Préserve ancrage, ordre attesté et dédoublonnage déjà corrigés. | 2026-08-31 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md reutilise les composants existants ou justifie les divergences
