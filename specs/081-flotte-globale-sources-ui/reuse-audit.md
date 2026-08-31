# Audit de reutilisation de l'existant - SPEC-081 Flotte globale et sources Desktop

## Decision

Statut: PASS  
Date: 2026-08-31  
Feature dir: /home/moi/bridget-referent/.worktrees/session-081-flotte-globale-sources-ui/specs/081-flotte-globale-sources-ui

Conclusion courte: le plan étend six points d'extension vérifiés et ne crée aucun second transport, registre de panneaux, store de profils, ni messagerie. Un nouveau module `fleet.rs` est justifié : il réduit des réponses privées de plusieurs origines et porte des tests purs, une responsabilité absente de l'existant. Aucune duplication évidente non arbitrée ne subsiste.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 8 |
| Items audites | 8 |
| Reutilisations deja prevues | 6 |
| Existants potentiellement pertinents | 2 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 3 |
| Specs existantes applicables | 4 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Lecture HTTP par tunnel | `request_relay_json` | `apps/bridget-desktop/src-tauri/src/lib.rs:103` | Étendre les lectures à snapshot et projets, sans client HTTP nouveau. |
| Construction URL authentifiée | `desktop_relay_url` et `percent_encode` | `apps/bridget-desktop/src-tauri/src/connection.rs:388` | Étendre la fonction ou son voisinage pour le mode compact et l'agent ciblé, sans URL en IPC. |
| Session par origine | `DesktopState.sessions` | `apps/bridget-desktop/src-tauri/src/lib.rs:49` | Source des connexions actives, copiée avant I/O. |
| Panneau unique | `PanelRegistry` | `apps/bridget-desktop/src-tauri/src/panels.rs:15` | Conserver `MAXIMUM_OPEN_PANELS = 1`, modifier seulement position et taille. |
| Préférences locales atomiques | `PreferencesStore` | `apps/bridget-desktop/src-tauri/src/preferences_store.rs:61` | Ajouter épingles et critères compatibles au document existant. |
| Projection source | `UiSnapshotV1` et `UiProjectListV1` | `crates/bridget-daemon/src/ui.rs:551`, `crates/bridget-daemon/src/ui.rs:932` | Consommer les routes déjà publiées et supprimer `canonical_path` avant IPC. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Liste locale d'agents | Styles de colonne déjà embarqués dans Desktop | `apps/bridget-desktop/ui/theme.css:20` | Réutiliser les jetons visuels et styles compatibles, sans réutiliser une structure qui n'est pas rendue par l'index Desktop actuel. |
| Pré-épinglage coordinateur | `AgentLinkUiProjection.role` | `crates/bridget-transport/src/protocol.rs:2815` | Consommer seulement la valeur exacte `coordinator`. Ne pas créer une seconde identité de coordinateur. |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucun | Aucun | Recherche large sur Desktop, daemon et specs | Aucune |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/Users/moi/.speckit/constitution.md` | Réutilisation avant création, état minimal, complexité explicite. | Transport et stores existants sont étendus. Projection O(s + a + p), tri O(a log a). |
| `/Users/moi/.speckit/ref/standards-tests.md` | Tests comportementaux et traçables. | Tests nommés SPEC-081 pour projection, origine, tri et UI compacte. |
| `/Users/moi/.speckit/ref/code-quality-details.md` | Contrats aux frontières et erreur décisionnelle. | Désérialisation stricte des réponses relais et état `unavailable` par source. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC-074 | Profils SSH, tunnel local, WebView distante isolée. | Conserver le confinement des secrets et du contenu distant. |
| SPEC-076 | Barre projet, Toute la flotte et identité de projet. | Transformer la navigation Desktop en Sources sans créer de seconde autorité projet. |
| SPEC-078 | Profils et attention séparés des identités routables. | Afficher une clé composite sans renommer l'agent. |
| SPEC-080 | Préférences Desktop atomiques et réglages contrôlés. | Étendre le store existant et garder le gestionnaire de serveurs atteignable. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg "DesktopFleet|fleet_snapshot|request_relay_json|desktop_relay_url"` | Desktop, daemon, specs 074, 076, 080 | Aucun agrégateur Desktop existant. Transport et URL existants réutilisables. |
| `rg "PanelRegistry|MAXIMUM_OPEN_PANELS|panel_open"` | Desktop | Registre unique et limite à un panneau confirmés. |
| `rg "DesktopPreferences|preferences_store"` | Desktop | Store local atomique unique confirmé. |
| `rg "canonical_path|UiProjectListV1"` | daemon UI | Chemin canonique présent dans la route administrative, à éliminer avant Tauri. |
| `rg "coordinator|AgentLinkUiProjection"` | transport et daemon | Aucun signal coordonnateur fiable hors rôle explicite. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Projection multi-sources | créer `fleet.rs` | Aucune projection équivalente, responsabilité distincte et testable. | 2026-08-31 |
| Transport | reutiliser `request_relay_json` | Évite un second client HTTP et conserve les délais existants. | 2026-08-31 |
| Conversation | reutiliser le panneau unique | Préserve l'isolation et le correctif de recouvrement. | 2026-08-31 |
| Coordinateur | consommer seulement le rôle explicite | Évite la fausse classification par nom. | 2026-08-31 |
| Chemin projet | réduire avant IPC | Le chemin n'est pas nécessaire à la flotte globale. | 2026-08-31 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Rejeu après implémentation

Date : 2026-08-31

| Élément créé ou étendu | Recherche et décision | Preuve finale |
|---|---|---|
| Projection de frontière | CRÉER fleet.rs : aucune projection Desktop existante, nécessité de retirer chemin et secret avant IPC. | apps/bridget-desktop/src-tauri/src/fleet.rs |
| Présentation pure | CRÉER fleet-presentation.js : aucun calcul partagé hors DOM, cinq règles de filtre/tri/groupe/épingle sont testées sans WebView. | apps/bridget-desktop/ui/fleet-presentation.js, fleet-presentation.test.mjs |
| Source locale | ÉTENDRE connection.rs : endpoint versionné déjà existant, aucune reprise du profil local historique. | discover_local_endpoint, commande constante bridget ui endpoint --json |
| Lecture source | RÉUTILISER request_relay_json : mêmes délais loopback, aucun client HTTP nouveau. | read_fleet_target dans src/lib.rs |
| Panneau conversation | RÉUTILISER PanelRegistry : une seule WebView, simplement décalée de 520 px. | MAXIMUM_OPEN_PANELS = 1, arrange_panels |
| Préférences | ÉTENDRE PreferencesStore : mêmes écritures atomiques et lecture rétrocompatible. | pinned_agent_keys, tris, groupes repliés |

Aucun wrapper de transport, store parallèle, endpoint réseau, dépendance ou autorité métier supplémentaire n'a été créé. Les deux modules nouveaux sont des frontières minimales avec usage réel et tests dédiés.
