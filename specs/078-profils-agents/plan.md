# Plan d'implémentation - SPEC-078 Profils d'agents et notifications

## Résumé

Donner une identité humaine durable aux agents sans changer leur adresse de
routage, rendre cette identité commune au Web et à Bridget Desktop, puis ajouter
un journal d'attention réellement sémantique. Les réglages de profil deviennent
actifs au prochain démarrage contrôlé du fournisseur. Les préférences de
notification restent distinctes par client et ne provoquent jamais de bruit
pour les outils ou le streaming.

## Résultat visé

- Tout agent visible possède un nom affiché unique, des labels indépendants et
  une apparence commune à tous les clients du même relais.
- Ni l'ID opaque ni le nom de routage ne s'affichent dans les surfaces normales.
- Les instructions individuelles possèdent une révision et un verdict honnête:
  appliquées après une naissance fournisseur ou en attente de relance.
- Les événements `human_input_needed`, `task_completed` et `terminal_failure`
  alimentent un centre d'activité, un badge et, après autorisation locale, une
  notification système unique par occurrence.
- Le clic sur la bouille ouvre le panneau de profil. Les trois points gardent
  exactement les actions rapides SPEC-077.

## Contexte technique

| Sujet | Choix |
|---|---|
| Backend | workspace Rust Bridget, SQLite du daemon, Axum et protocoles existants |
| UI conversation | HTML, CSS et JavaScript embarqués dans `crates/bridget-daemon/assets/ui` |
| Client Desktop | Tauri v2 macOS, WebView enfant relayée par tunnel SSH |
| UI partagée | le relais reste la source unique: aucune seconde conversation Desktop |
| Identité interne | UUID profil dans SQLite, jamais un remplacement de `name` technique |
| Notification Web | API Notification et centre d'activité relayé |
| Notification Desktop | plugin officiel Tauri, capacités macOS de la coque seulement |
| Fournisseurs | Codex, Claude, GLM, DeepSeek, Cursor/ACP |
| Service externe | aucun |
| Clarifications ouvertes | aucune: les limites d'application en session sont désormais explicites |

## Constitution Check

| Gate | Décision |
|---|---|
| SpecKit complet | SPEC, recherche, plan, audit de réutilisation, tâches, analyse, implémentation, revue et audit final |
| Worktree isolé | `session-078-profils-agents` créé depuis `main` synchronisé |
| Persistance sûre | migration SQLite additive, transactionnelle, idempotente, sans réécriture du ledger |
| Minimalisme | un registre de profils et un journal sémantique, pas de service de notification ni de seconde UI |
| Sécurité | jeton de relais existant, aucune clé, consigne ou UUID dans les logs et surfaces utilisateur |
| Réutilisation | store SQLite, snapshot/watch, panneau droit, renderer avatar, tunnel Desktop et actions SPEC-077 étendus |
| Accessibilité | focus stable, rôle status ARIA poli, clavier, 200 %, 1280 x 720 |
| Responsabilité future | contrats versionnés, états d'application explicites, tests de migration et de déduplication |
| Article XIX/XX | complexité justifiée par la séparation indispensable routage/profil/client; chaque nouvelle table a un invariant et un test |

Aucune violation n'est admise. Le seul package nouveau est le plugin officiel
Tauri de notification: il remplace une intégration native ad hoc et n'ajoute ni
service externe ni privilège à la WebView relayée.

## Architecture cible

```text
nom de routage / ledger / flotte
             │ alias de routage
             ▼
AgentIdentity UUID ───── AgentProfile partagé
             │                    │
             │                    ├── display_name, labels, avatar
             │                    └── instructions + révision + application
             ▼
AttentionEvent sémantique ───── ClientPreference / ClientAttentionState
             │                                    │
             ▼                                    ▼
 snapshot + watch + API profil             Web / Desktop, badge, activité,
                                           notification locale
```

### Frontières de propriété

| Zone | Responsabilité SPEC-078 | Réutilisation / exclusion |
|---|---|---|
| `crates/bridget-daemon/src/store.rs` | identité, profil, instructions, événements et états client | étend le ledger SQLite sans modifier les lignes de messages |
| `crates/bridget-daemon/src/ui.rs` | projection, mutations authentifiées, flux d'attention | étend snapshot/watch et l'authentification existante |
| `crates/bridget-daemon/src/wrapper.rs` | snapshot d'instructions au spawn/reprise, verdict | point unique de lancement et relance fournisseur |
| `crates/bridget-transport` | primitive d'amorçage uniquement là où le protocole l'autorise | aucun détournement de steer, interrupt ou session active |
| `assets/ui` | panneau profil, activité, préférences, rendu par display name | réemploie détail, avatar, menu SPEC-077 et Notification Web |
| `apps/bridget-desktop/src-tauri` | client_id stable, livraison macOS native et ouverture de l'événement | la WebView ne reçoit pas de capacité Tauri |
| personas | hors périmètre | aucun catalogue Entropie ou Libraire |

## Conception détaillée

### 1. Registre d'identités et profils

Ajouter les tables `agent_identities`, `agent_routing_aliases`,
`agent_profiles`, `agent_profile_labels` et `agent_profile_applications` dans
la SQLite du daemon. La migration importe les noms de la flotte, de l'annuaire
vivant et du ledger. Elle peut être rejouée et ne réécrit jamais les tables de
messages.

La projection résout d'abord un alias technique vers `agent_id`, puis le profil.
Si le store est indisponible ou un profil absent, elle montre le libellé historique
comme fallback de lecture mais n'empêche ni conversation ni gestion de flotte.
Une fois la migration achevée, toute surface d'écriture et d'affichage normal
utilise exclusivement `display_name`.

### 2. Contrat de profil et contrôle de concurrence

`UiAgentRowV1` reçoit une sous-projection profile avec référence opaque,
`display_name`, labels, avatar et état d'instructions. La lecture détaillée ne
retourne le texte des instructions qu'au panneau explicitement ouvert. Une
mutation `PATCH` contient la révision connue et remplace nom, labels, avatar et
instructions dans une seule transaction. Les collisions de nom et conflits de
révision ont des codes stables sans fuite d'identité.

Le renderer ne doit jamais copier l'UUID dans un attribut HTML. Il le conserve
seulement dans l'état JavaScript nécessaire aux appels API.

### 3. Insertion fournisseur sûre

Le daemon charge un snapshot d'instructions à chaque spawn et reprise. Il
conserve `instructions_revision` et le verdict d'application sans journaliser le
texte. L'ordre de contexte est fixe:

1. invariants Bridget, sécurité et permissions;
2. mandat, politiques et contexte de travail;
3. bloc délimité d'instructions individuelles;
4. travail reçu.

Toutes les familles reçoivent une carte interne Bridget contrôlée avant le
premier travail d une nouvelle session. Un mécanisme fournisseur natif ne sera
ajouté qu après preuve qu il ne place jamais le texte dans les arguments de
processus, les logs ou un profil fournisseur global. Aucun transport ne modifie
une session active. Une écriture pendant un tour passe donc en `pending_restart`.

### 4. Événements et préférences de notification

Le daemon dérive les seuls événements éligibles depuis les faits d'exécution
existants, avec une clé d'occurrence idempotente. Il conserve
`AttentionEvent`, puis chaque client possède ses préférences et ses marqueurs de
lecture. Un événement non sélectionné ne devient ni badge ni notification pour
ce client. Le centre d'activité est une projection d'événements persistants,
pas une capture de toast.

Le navigateur garde son client_id local. Bridget Desktop possède le sien dans
le répertoire de données de l'application, indépendant du port de tunnel. Les
préférences suivent le client au travers du contrat autorisé et ne sont jamais
partagées avec un second client. La coque Desktop lit les événements du relais
via son tunnel, applique ses préférences et envoie l'alerte macOS. Le contenu
relayé reste incapable d'invoquer Tauri.

### 5. Panneau et activité

Généraliser le panneau droit existant avec deux modes explicites:
`peerExchange` et `agentProfile`. Le bouton bouille de l'en-tête ouvre ce dernier
sans changer le fil. Il contient nom affiché, labels multi-pastilles, apparence,
consignes avec statut et réglages locaux d'attention. Les trois points conservent
le menu contextuel et toutes les actions SPEC-077.

Le centre d'activité est ouvert par un bouton à badge. Il est navigable au
clavier, annonce les mises à jour dans une région ARIA polie, ouvre l'agent et
marque une entrée lue sans voler le focus. Les libellés sont brefs et n'utilisent
que le display name.

## Phases d'implémentation

### P1 - Fondations de profil

1. Créer migrations, modèles et opérations atomiques du store.
2. Importer les noms de la flotte, annuaire et ledger de façon idempotente.
3. Ajouter les tests de migration, alias, unicité et dégradation de lecture.

### P2 - Relais et projection

1. Étendre les DTO snapshot/watch avec le profil non secret.
2. Ajouter lecture et mutation de profil token-authenticated avec validation.
3. Remplacer les libellés utilisateur de projection par display name et couvrir
   les conversations, recherches et erreurs.

### P3 - Consignes effectives

1. Définir le snapshot de contexte initial et la table d'application.
2. Câbler lancement/reprise du wrapper et chaque famille de transport.
3. Tester états applied, pending_restart, unsupported et absence de fuite dans
   les arguments ou diagnostics.

### P4 - Journal sémantique et clients

1. Produire et dédupliquer les trois types d'événements depuis l'exécution.
2. Ajouter lecture du flux, préférences par client, activité et marqueurs lus.
3. Tester l'absence totale d'événement pour outils, commandes, chunks et états
   routiniers.

### P5 - Interface Web partagée

1. Migrer l'avatar local vers l'apparence projetée.
2. Ajouter panneau profil, éditeur de labels et état de consignes.
3. Ajouter recherche par display name et labels, centre d'activité, badge,
   notification navigateur et comportements clavier.
4. Préserver le menu contextuel SPEC-077 sans duplication.

### P6 - Bridget Desktop et validation croisée

1. Ajouter client_id stable et préférences Desktop non secrètes.
2. Ajouter le plugin/capacité Tauri de notification et le suivi relayé borné.
3. Ouvrir l'agent concerné à l'activation d'une notification native.
4. Vérifier que deux clients voient le même profil mais des préférences et
   lectures différentes.

### P7 - Durcissement et preuves

1. Exécuter tests Rust daemon, transport, UI Node et Desktop.
2. Tester les scénarios Web/Desktop, reconnexion, conflits, permission refusée,
   redémarrage, fournisseurs et accessibilité.
3. Vérifier avec recherche de sources et tests que noms techniques, UUID et
   instructions n'apparaissent ni dans les surfaces, ni dans logs de test.

## Fichiers prévus

| Fichier ou zone | Modification |
|---|---|
| `crates/bridget-daemon/src/store.rs` | schéma SQLite, registre, profil, événements et préférences |
| `crates/bridget-daemon/src/ui.rs` | DTO, routes, projection et validation |
| `crates/bridget-daemon/src/wrapper.rs` | snapshot d'instructions et verdict de spawn |
| `crates/bridget-daemon/src/daemon.rs` ou `fleet.rs` | propagation des faits de cycle de vie vers attention |
| `crates/bridget-transport/src/claude_stream_json.rs` | contexte Claude hors arguments journalisés |
| `crates/bridget-transport/src/codex_app_server.rs` | amorçage contrôlé au nouveau thread |
| `crates/bridget-transport/src/acp.rs` | amorçage contrôlé au nouveau session/new |
| `crates/bridget-daemon/assets/ui/index.html` | panneau profil, activité et live region |
| `crates/bridget-daemon/assets/ui/app.js` | état profil, rendu, préférences et notifications Web |
| `crates/bridget-daemon/assets/ui/theme.css` | pastilles, panneau, activité et états accessibles |
| `apps/bridget-desktop/src-tauri/*` | client_id, préférences, notifications macOS et activation |
| `apps/bridget-desktop/ui/*` | seulement si une commande de coque minimale est requise |
| `docs/decisions/020-profils-agents-et-attention-client.md` | ADR de séparation identité, profil et client |
| `specs/078-profils-agents/*` | traçabilité, tâches et preuves |

## Stratégie de tests

- Store SQLite: migration vide et existante, idempotence, aliases, collisions,
  rollback transactionnel, absence de réécriture du ledger.
- Relais: contrat snapshot/watch, PATCH valide/invalide, authentification,
  projection sans nom technique/UUID, recherche et compatibilité fallback.
- Fournisseurs: tests de traces Codex, Claude/GLM/DeepSeek et ACP/Cursor avec
  révision appliquée ou en attente, sans commande active détournée.
- Événements: déduplication, ordre, reprise après reconnexion, préférences de
  deux clients et zéro événement pour les faits exclus.
- UI Node: parsing labels, rendu de pastilles, panneau/menus distincts,
  non-lus, ARIA, notification dédoublonnée et masquage des IDs.
- Desktop: store de client_id, tunnel, autorisation Tauri, notification native,
  activation, port loopback variable et séparation des WebViews.
- Manuel: Web et Desktop simultanés, 200 %, 1280 x 720, lecteur d'écran de la
  région status, permission refusée et ouverture depuis une notification.

## Déploiement

Aucun commit, merge, push ou redémarrage n'est inclus dans cette session
SpecKit. Une livraison ultérieure devra construire le daemon, empaqueter
Bridget Desktop sur macOS, vérifier les migrations sur une copie de données et
redémarrer seulement après validation explicite.
