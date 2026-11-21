# Spécification 308 : Google Antigravity et connexion facilitée multi-fournisseurs

**Statut** : En cours
**Branche** : `main` (décision branche unique du 10 novembre 2026)
**Date** : 2026-11-21
**Dépendances** : specs 168/220 (routing et comptes isolés), 208/209 (compatibilité clients), transport ACP (`crates/bridget-transport/src/acp.rs`).

## Demande initiale et cadrage

**Demande reformulée :**
1. Intégrer **Google Antigravity (`agy`)** dans Bridget comme moteur de premier rang au même titre que Claude et Codex : protocole `acp`, isolement de profil par compte (`antigravity_home` -> `--gemini-dir`), sélection de modèle/effort/mode de permissions, contrôle de compatibilité client (`agy --version`), délégation native et wrapper CLI `bridget agy`.
2. Installer `agy` sur les serveurs Linux joignables (`cartae`, `sysle2`, `ksstore`) et enregistrer les connexions Antigravity sur les 4 machines (`monordinateur`, `cartae`, `sysle2`, `ksstore`).
3. Fournir une expérience simple pour connecter, reconnecter ou changer de compte sur presque tous les fournisseurs (Claude, Codex, Antigravity) et sur chaque machine depuis `monordinateur`, y compris la création et la connexion à distance d'un compte isolé supplémentaire :
   - en CLI (`bridget accounts status`, `bridget accounts login`),
   - et via une petite interface graphique locale (`bridget accounts ui`, fenêtre web strictement locale `127.0.0.1` protégée par jeton de session + lanceur macOS `/Users/moi/Applications/Bridget Comptes.app`).

**Exclusions et garanties de sécurité :**
- Aucun token, clé API ou cookie n'est stocké dans Bridget (`bridget.db`, `agents.json`, journaux) : chaque fournisseur écrit uniquement dans son dossier privé (`0700`/`0600`) ou son trousseau système sur la machine cible.
- Aucune exposition réseau de l'interface `bridget accounts ui` : écoute exclusive sur `127.0.0.1` avec jeton aléatoire de session.
- `macbook-m1` reste hors périmètre tant qu'il est hors ligne.

## Exigences fonctionnelles

### US1 (P1) - Protocole ACP complet pour Google Antigravity (`bridget-transport`)
- **FR-001** : `AcpOptions` accepte des options rétrocompatibles : `auth_method_id`, `client_info`, `model`, `effort`, `session_mode`.
- **FR-002** : Lors de l'initialisation ACP, `AcpTransport` envoie `clientInfo` si fourni, appelle `authenticate` avec `auth_method_id` avant `session/new` si configuré, puis applique `session/set_mode` et `session/set_config_option` (ou `session/set_model`) pour le modèle et le niveau de réflexion demandés.

### US2 (P1) - Moteur `antigravity` de premier rang dans Bridget (`bridget-daemon`)
- **FR-003** : `AgentDefinition` et `ResolvedAgentDefinition` ajoutent `antigravity_home: Option<String>` (`#[serde(default, skip_serializing_if = "Option::is_none")]`) sans changer le digest des définitions existantes qui n'utilisent pas ce champ.
- **FR-004** : `registry.rs` reconnaît le couple `("antigravity", "acp")` (routage, validation, `capabilities.account_profiles`, définition native `agy`) et interdit la fuite de variables globales d'authentification (`GEMINI_API_KEY`, `GOOGLE_API_KEY`, `AGY_ADC_AUTH`) lorsque `antigravity_home` est défini.
- **FR-005** : `lifecycle.rs` et `wrapper.rs` injectent `--gemini-dir <antigravity_home>` et `--app-data-dir antigravity-bridget` pour Antigravity, propagent le modèle, l'effort et le mode de permission vers `AcpOptions`, et le CLI accepte `bridget agy [ARGS...]`.
- **FR-006** : `client_compatibility.rs` reconnaît le binaire `agy` (`engine = "antigravity"`, `protocol = "acp"`, lecture de `agy --version`, minimum `1.5.0`) tout en conservant la compatibilité ascendante pour les machines qui n'ont que Claude et Codex.
- **FR-007** : `provider_accounts.rs` prend en charge `("antigravity", "acp")` dans `bridget accounts prepare` (création du dossier privé `provider-accounts/<id>/antigravity` en `0700` et remplissage de `antigravity_home`).

### US3 (P1) - Connexion et changement de compte centralisés en CLI (`bridget accounts status` & `login`)
- **FR-008** : `bridget accounts status [--json]` inspecte localement et via SSH non interactif (`BatchMode=yes`, parallèle, délai borné) toutes les connexions déclarées sur `monordinateur`, `cartae`, `sysle2` et `ksstore`, et renvoie leur état réel (`authenticated`, `unauthenticated`, `binary_missing`, `machine_unreachable`) sans jamais lire ni afficher de secret.
- **FR-009** : `bridget accounts login` permet de connecter ou reconnecter une connexion existante (`--connection <id> [--logout-first]`) ou de créer, connecter et activer un nouveau compte isolé en une passe (`--from <conn> --prepare-id <nom>`), localement ou sur une machine distante via SSH interactif (avec tunnel `-L 1455:127.0.0.1:1455` pour Codex distant et flux OAuth natif pour Claude et Antigravity).

### US4 (P1) - Fenêtre graphique locale `bridget accounts ui` et lanceur macOS
- **FR-010** : `bridget accounts ui` démarre un serveur HTTP strictement local (`127.0.0.1:<port éphémère>`) protégé par jeton de session aléatoire et ouvre automatiquement le navigateur sur l'interface graphique.
- **FR-011** : L'interface affiche chaque machine et chaque connexion avec son état d'authentification en direct, un bouton « Se connecter / Changer de compte » (ouverture auto de l'URL OAuth + zone de saisie du code de validation si demandé par le fournisseur distant + journal de session en direct), un formulaire « + Nouveau compte isolé » (local ou distant), et un bouton d'actualisation.
- **FR-012** : Un lanceur macOS léger `/Users/moi/Applications/Bridget Comptes.app` ouvre directement `bridget accounts ui` en un clic ou via Spotlight.

### US5 (P1) - Déploiement d'`agy` et de `311-antigravity-comptes-r1` sur les 4 machines
- **FR-013** : `agy` est installé et vérifié (`agy --version`) sur `cartae`, `sysle2` et `ksstore` (en plus de `monordinateur`).
- **FR-014** : Les connexions `antigravity-mac`, `antigravity-cartae`, `antigravity-sysle2`, `antigravity-ksstore` sont ajoutées proprement aux registres `agents.json` (avec sauvegarde préalable) et activées avec la version `311-antigravity-comptes-r1` sur les 4 machines joignables.

## Critères de réussite mesurables

- **SC-001** : `agy --version` répond sur les 4 machines (`monordinateur`, `cartae`, `sysle2`, `ksstore`).
- **SC-002** : Tous les tests unitaires ACP, `registry`, `provider_accounts`, `client_compatibility` et `accounts_connect` passent avec `make lint` à 0 avertissement.
- **SC-003** : `bridget accounts status` depuis `monordinateur` renvoie l'état de toutes les connexions Claude, Codex et Antigravity sur les 4 machines en moins de 8 secondes.
- **SC-004** : `bridget accounts ui` démarre sur `127.0.0.1`, sert l'interface protégée par jeton et expose les API JSON `/api/status`, `/api/login/start`, `/api/login/input`, `/api/prepare`.
- **SC-005** : Version `311-antigravity-comptes-r1` compilée (macOS ARM64 + Debian 12 glibc <= 2.36), testée en compatibilité bidirectionnelle avec `310-purge-unreachable-r1`, poussée sur `origin/main` et active sur les 4 machines.
