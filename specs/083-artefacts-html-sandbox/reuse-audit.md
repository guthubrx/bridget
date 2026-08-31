# Audit de réutilisation de l'existant - SPEC-083

## Decision

Statut: PASS
Date: 2026-08-31
Feature dir: `/private/tmp/bridget-project-nav.JNWHqE/specs/083-artefacts-html-sandbox`

Conclusion courte: l'application dispose déjà de WebViews enfants, d'un registre
de panneau unique, d'un store de préférences, d'une interception de liens et
d'un renderer Markdown assaini. Le plan les étend pour créer deux surfaces
séparées, Browser humain et cadre HTML non fiable. Il n'existe pas de runtime
HTML sandboxé ni de navigateur distant actif à dupliquer ; `RemoteBrowser` est
documentaire seulement.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 9 |
| Items audites | 9 |
| Reutilisations deja prevues | 7 |
| Existants potentiellement pertinents | 2 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 4 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Création et disposition WebView | enfant Tauri de la coque | `apps/bridget-desktop/src-tauri/src/lib.rs:809`, `:826` | Étendre cette création contrôlée, avec labels distincts sans permissions. |
| Panneau droit | `PanelRegistry` et limite de panneau | `apps/bridget-desktop/src-tauri/src/panels.rs:15`, `:52` | Étendre ce registre et `arrange_panels`, pas de second système de panneaux. |
| Ouverture de lien | protocole `bridget-open:` | `apps/bridget-desktop/src-tauri/src/lib.rs:815`, `crates/bridget-daemon/assets/ui/app.js:6747` | Remplacer l'ouverture navigateur système par une remise typée au Browser interne. |
| Réglages Browser/panneau | `DesktopPreferences` | `apps/bridget-desktop/src-tauri/src/preferences_store.rs:34`, `:105` | Ajouter des champs typés, portés `Ce Mac`. |
| Frontière de permissions | capabilities `main` et `panel-*` | `apps/bridget-desktop/src-tauri/capabilities/main.json`, `relay-notification-permission.json` | Créer des capabilities sans droit pour `browser-*` et `artifact-frame-*`. |
| Conversation et références | Markdown assaini et projection | `crates/bridget-daemon/assets/ui/app.js:6440`, `:5733` | Ajouter les cartes/références sans transformer Markdown en exécuteur HTML. |
| Réglages de contenu existants | `ContentSecurityPreferences` | `apps/bridget-desktop/src-tauri/src/preferences_store.rs:40` | Étendre sous une politique Browser dédiée, sans relâcher la sandbox. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Browser distant | `ProfileCapability::RemoteBrowser` documentaire | `apps/bridget-desktop/src-tauri/src/profile.rs:33` | Ne pas l'activer : il concerne un profil distant et ne fournit ni Browser local ni isolation d'artefact. |
| Panneau relayé unique | `MAXIMUM_OPEN_PANELS = 1` | `apps/bridget-desktop/src-tauri/src/panels.rs:5` | Conserver le registre unique et le faire évoluer pour les enfants Browser/cadre, sans ouvrir une flotte de WebViews. |

## Duplications evidentes

Aucune duplication évidente détectée.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/Users/moi/.speckit/constitution.md` | Moindre privilège et secrets | séparation labels/capabilities, aucun cookie ou IPC agent. |
| `/Users/moi/.speckit/constitution.md` | Réutilisation avant création | `PanelRegistry`, Tauri child WebViews et préférences sont étendus. |
| `/private/tmp/bridget-project-nav.JNWHqE/AGENTS.md` | Ne jamais dépasser deux hypothèses sans observation | la recette inclut fixtures hostiles et tests de confinement observables. |
| `/private/tmp/bridget-project-nav.JNWHqE/AGENTS.md` | Préférences d'application globales | Browser/cache/panneau sont explicitement `Ce Mac`, jamais projet/agent. |
| `specs/081-conversation-renderer/contracts/local-content-preferences-v1.md` | navigation locale interceptée par coque | La route est étendue au Browser interne plutôt que contournée. |
| `specs/082-artefacts-natifs-durables/plan.md` | registre/version/source canonique | HTML et onglets lisent ce registre, sans stockage concurrent. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| `specs/074-bridget-desktop` | WebViews enfants loopback et isolation de la coque | Browser/cadre restent enfants limités, aucune capacité métier. |
| `specs/080-centre-controle-bridget` | réglages locaux, centre de contrôle | effacement et politique Browser sont placés dans les réglages globaux. |
| `specs/081-conversation-renderer` | contenu Markdown non exécutable, références sur geste | HTML sandboxé reste une branche de rendu distincte, pas une exception Markdown. |
| `specs/082-artefacts-natifs-durables` | versions, blobs, provenance et panneau Artefacts | prérequis direct et source unique de vérité. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg WebviewBuilder|bridget-open|DesktopPreferences|RemoteBrowser` | `apps/bridget-desktop/` | enfants WebView, ouverture externe et préférences existants ; RemoteBrowser non implémenté. |
| `find capabilities -type f` | Desktop capabilities | main privilégié, panneaux relayés sans core Tauri. |
| `rg projectTimeline|renderMessageMarkdown|content_security` | renderer relayé | timeline factuelle et Markdown assaini confirmés. |
| inspection T3 `PanelLayoutControls.tsx` | `/Users/moi/11.Repositories/t3code` | icônes et états exacts pour panneau/agrandissement observés sous MIT. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Gestion des panneaux | etendre `PanelRegistry` | Évite deux régies de disposition et conserve la borne existante. | 2026-08-31 |
| Browser | creer une surface enfant dédiée mais sans capability | Aucun Browser local réutilisable n'existe ; la séparation est une mesure de sécurité. | 2026-08-31 |
| HTML actif | creer un runtime iframe isolé | Il n'existe aucun exécuteur HTML et Markdown doit rester non exécutable. | 2026-08-31 |
| Ouverture de lien | etendre `bridget-open:` | Le protocole existe et doit cesser de déléguer silencieusement au navigateur système. | 2026-08-31 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md reutilise les composants existants ou justifie les divergences
