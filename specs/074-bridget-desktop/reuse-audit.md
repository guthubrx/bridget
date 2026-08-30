# Audit de reutilisation de l'existant - SPEC-074 Bridget Desktop

## Decision

Statut: PASS
Date: 2026-08-30
Feature dir: `/home/moi/bridget-referent/.worktrees/session-074-bridget-desktop/specs/074-bridget-desktop`

Conclusion courte: le plan ne recrée ni le relais UI, ni son contrat HTTP/SSE, ni un daemon, ni une exposition réseau publique. Il étend la commande UI existante pour une découverte d'endpoint typée, puis crée le premier client macOS dans un périmètre séparé car aucun client desktop, profil de connexion ou paquet Tauri n'existe dans le workspace. Le tunnel client reprend les garanties SSH déjà pratiquées, sans réutiliser le script interactif qui démarre un relais temporaire concurrent. Aucun doublon évident ne reste non arbitré.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 14 |
| Items audites | 14 |
| Reutilisations deja prevues | 5 |
| Existants potentiellement pertinents | 2 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 5 |
| Specs existantes applicables | 6 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Découverte d'endpoint UI | `UiEndpoint` et lecture validée | `crates/bridget-daemon/src/ui.rs:93`, `crates/bridget-daemon/src/ui.rs:184` | Ajouter une sortie CLI étroite au lieu de relire le fichier d'état depuis le client. |
| Point d'extension CLI UI | `cmd_ui` existant | `crates/bridget-daemon/src/cli.rs:125`, `crates/bridget-daemon/src/cli.rs:181` | Étendre `bridget ui` avec une sous-action fermée, sans nouvelle commande racine. |
| Relais et rendu de conversation | Relais HTTP/SSE et assets embarqués | `crates/bridget-daemon/src/ui.rs:63`, `crates/bridget-daemon/src/ui.rs:512` | Le client affiche le relais servi, il ne copie ni ne réimplémente la conversation. |
| Tunnel SSH client | Principe de forward local | `scripts/open-remote-ui.sh:47`, `scripts/open-remote-ui.sh:97` | Réemployer le principe `-L` avec un processus possédé par le client, pas le script. |
| Robustesse tunnel | Paramètres SSH de la fédération | `scripts/federate-ssh.sh:37`, `scripts/federate-ssh.sh:65` | Reprendre `BatchMode`, `ExitOnForwardFailure` et keepalives, sans modifier le tunnel inverse des agents. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Coque desktop et consultation multi-vues | Étude GUI T3 Code seulement documentaire | `docs/decisions/009-gui-plan-de-controle.md:20` | Créer le client minimal propre au dépôt: aucun composant source Bridget réutilisable n'existe. |
| Diagnostics sans secret | Projections redacted et tests de contenu secret | `crates/bridget-transport/src/codex_app_server.rs:2937`, `scripts/test-bridget-idle.sh:1584` | Réutiliser la règle de non-divulgation, créer le format desktop limité à son besoin. |

## Items sans equivalent existant

| Item du plan | Recherche et preuve d'absence | Decision |
|---|---|---|
| Racine `apps/bridget-desktop` et paquet Tauri | `find . -name Cargo.toml -o -name package.json` ne retourne que le workspace Rust et ses crates. | Créer le premier périmètre client séparé. |
| Persistance locale de profils non secrets | `rg -n -i "profile|connection.*state"` dans les crates, assets et specs ne révèle aucun profil de serveur client. | Créer un stockage local minimal, versionné et sans secret. |
| Formulaires d'ajout, édition, retrait et sélection de serveur | Aucun frontend Desktop ou composant de gestion de serveurs dans le workspace. | Créer seulement les formulaires nécessaires aux profils SPEC-074. |
| Construction d'arguments SSH depuis une coque locale typée | Les scripts shell existants couvrent seulement l'opération manuelle. | Créer un backend Tauri fermé, sans commande libre transmise par l'interface. |
| Approbation d'empreinte et `known_hosts` par profil | `rg -n -i "known_hosts|StrictHostKeyChecking"` ne retourne aucun cycle de confiance applicatif. | Créer ce flux de confiance explicite, sans modifier les fichiers SSH globaux par défaut. |
| Supervision de tunnel, reconnexion et nettoyage à la fermeture | `open-remote-ui.sh` reste attaché au terminal et ne gère pas de session Desktop. | Créer la supervision liée à la fenêtre et à la session de profil. |
| Vue à deux panneaux et profil local direct | Aucun panneau multi-origine ni connexion locale sans SSH dans l'UI ou les crates. | Créer la coque locale, sans agréger les timelines métier. |
| Contrat futur de navigateur distant | Recherche `browser session|novnc|playwright` sans code réutilisable Bridget. | Documenter la frontière seulement; ne rien créer dans SPEC-074. |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucun | Aucun | Recherche couvrant workspace, scripts et specs | Aucune |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/home/moi/.speckit/constitution.md` | SpecKit, ADR et tests avant livraison | Conserver SPEC-074, ADR-018, tâches vérifiables et preuves de test. |
| Instructions projet fournies à la session | Développement serveur dans un worktree isolé | Sources dans ce worktree uniquement, packaging macOS comme validation cible séparée. |
| Instructions projet fournies à la session | Secrets hors Git et logs, SSH prudent | Profils non secrets, jeton seulement en mémoire, empreinte explicite, diagnostics redacted. |
| `docs/decisions/017-profils-fournisseurs-compatibles.md` | Le fournisseur est déclaré sans déduction et ses secrets restent isolés | Desktop ne déduit ni ne déplace les secrets de fournisseur; il affiche le relais existant. |
| `docs/decisions/018-bridget-desktop-tunnels-client.md` | Le client possède ses tunnels, le serveur reste loopback | Pas de port public, pas de proxy serveur, pas de tunnel générique anticipé. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC-002 federation SSH | Forward sécurisé, keepalive, échec explicite | Réutiliser les options SSH sans confondre tunnel client et fédération inverse. |
| SPEC-063 interruptions | États d'exécution et contrôle réels | Le contenu rendu par le relais conserve ces comportements, sans logique desktop dupliquée. |
| SPEC-064 contrôle Bridget-Maicie | Autorités et corrélations séparées | Desktop reste une surface de connexion, sans transition métier ni autorité Maicie. |
| SPEC-070 activité et notifications | États honnêtes et notification orientée message | Séparer les états de tunnel de l'état des agents et conserver l'origine du panneau. |
| SPEC-071 provenance fournisseur | Présentation d'identité runtime/fournisseur | L'UI distante est réemployée telle quelle, sans régression d'overlay. |
| SPEC-072 profils fournisseurs | Fournisseurs explicites et secrets isolés | Le profil Desktop est un profil de serveur, pas un profil de fournisseur. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg -n -i "ui-endpoint|UiEndpoint|open-remote-ui"` | workspace et scripts | Endpoint et script de tunnel trouvés; réutilisation partielle décidée. |
| `rg -n -i "ssh|known_hosts|StrictHostKeyChecking|ExitOnForwardFailure"` | scripts, crates, SPEC-002 | Garanties SSH existantes localisées. |
| `find . -name Cargo.toml -o -name package.json` | workspace | Aucun package desktop ou Tauri existant. |
| `rg -n -i "desktop|tauri|webview|browser session|novnc|playwright"` | specs, docs, crates, scripts | Seulement une ADR documentaire et les artefacts SPEC-074. |
| `rg -n -i "redact|secret|token|diagnostic"` | crates, scripts, decisions | Règle de projection sans secret déjà établie. |
| `rg -n -i "profile|connection.*state|reconnect|notification.*origin"` | UI et SPEC-063 à SPEC-073 | États de relais existants à consommer, pas de profils de serveur existants. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Commande de découverte | reutiliser | `UiEndpoint` contient déjà le contrat et la validation; seule son exposition CLI manque. | 2026-08-30 |
| Tunnel local | reutiliser | Le mécanisme `ssh -L` et les options de sûreté existent; Desktop doit seulement en posséder le cycle de vie. | 2026-08-30 |
| Script `open-remote-ui.sh` | creer nouveau | Le script démarre un relais temporaire et nécessite un terminal; le client doit joindre le service UI déjà vivant. | 2026-08-30 |
| Application desktop | creer nouveau | Aucun package ou composant client n'existe dans le workspace; `apps/bridget-desktop` sépare nettement le code client. | 2026-08-30 |
| Navigateur distant | approfondir | Aucune implémentation existante; capacité explicitement hors SPEC-074, à traiter dans une SPEC future. | 2026-08-30 |

## Réaudit après implémentation

| Surface créée | Réutilisation ou arbitrage vérifié | Résultat |
|---|---|---|
| `src/profile.rs`, `profile_store.rs`, `profile_service.rs` | Aucun modèle de profil client existant dans Bridget; réutilise les règles existantes de redaction et de stockage strict. | Création justifiée, sans coffre de secrets ni mutation de `~/.ssh`. |
| `src/ssh.rs`, `host_identity.rs`, `connection.rs` | Réutilise les options SSH déjà établies dans `scripts/federate-ssh.sh` et le contrat `UiEndpoint` ajouté au CLI. | Aucune commande libre ou tunnel générique ajouté. |
| `src/panels.rs` et Tauri multiwebview | Aucun panneau Desktop réutilisable; le feature Tauri `unstable` est l'unique coût pour deux origines isolées. | Deux panneaux locaux loopback maximum, sans capability pour `panel-*`. |
| `ui/` | Aucun frontend Desktop existant; l'UI distante reste servie par le relais Bridget, la coque ne réimplémente pas la conversation. | Création limitée aux profils, empreintes et états de tunnel. |
| `tests/*.rs` | Réutilisent le framework de test Rust standard déjà présent. | Aucun crate de test, mock HTTP ou framework de navigateur ajouté. |
| `icons/icon.svg`, `icons/icon.png` | Aucun asset de marque Desktop réutilisable. | Pictogramme vectoriel local minimal, sans dépendance graphique. |

### Dépendances ajoutées ou évitées

- Ajout direct nécessaire : `tauri` et `tauri-build`, limités à macOS et au workspace Cargo imbriqué `apps/bridget-desktop`.
- Dépendances réutilisées : `serde`, `serde_json`, `uuid`, déjà présentes dans le dépôt.
- Dépendances volontairement évitées : plugin shell Tauri, gestionnaire de secrets, client SSH Rust, serveur HTTP local supplémentaire, VNC/noVNC et bibliothèque navigateur.

### Conclusion de réaudit

`rg -n "Command::new|WebviewBuilder|token|PRIVATE KEY|127\\.0\\.0\\.1" apps/bridget-desktop` montre que les seuls processus créés sont SSH ou la découverte locale fixe, que les panneaux sont contraints à la boucle locale et que les sentinelles de secret restent dans les tests. Aucun doublon de relais, d'API métier ou de frontend de conversation n'a été introduit.

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
