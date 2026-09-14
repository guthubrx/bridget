# Audit de reutilisation de l'existant — 098 Adaptateur t3code

## Decision

Statut: PASS
Date: 2026-09-14
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/098-t3code-adapter/specs/098-t3code-adapter
Conclusion courte (révision 2, option A) : le plan réutilise la commande embarquée 096, le service launchd/systemd de 095, la remise idempotente et le journal de 089, la présence par canal réel de 097. Deux créations justifiées (pont t3code, contrat isolé), une extension de dépendance existante (`uuid` v5), une dépendance nouvelle minimale arbitrée (`minreq` sans TLS). Abandonnés après contre-revue : fichier MCP dans t3code, identité MCP hébergée, écriture de `settings.json`. Aucune duplication évidente.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 10 |
| Items audites | 10 |
| Reutilisations deja prevues | 5 |
| Existants potentiellement pertinents | 2 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 4 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Commande `bridget t3` | `federate::run`, dispatch `cli.rs` | `crates/bridget-daemon/src/federate.rs:88`, `crates/bridget-daemon/src/cli.rs:170` | même schéma d'entrée qu'en 096 |
| Remise idempotente vers un fil | `deliver_idempotent_to_interactive`, `IdempotentDeliveryTracker` | `crates/bridget-daemon/src/wrapper.rs:751`, `:572` | fermeture d'injection = `dispatch` HTTP |
| Présence par fil | `connect_and_register_with_domain_at`, `PresenceMode::Cli` + transport libre | `crates/bridget-daemon/src/wrapper.rs:1492`, `crates/bridget-daemon/src/daemon.rs:5852` | même schéma que `claude_pty` (097) |
| Journal et attach par fil | `JournalWriter::start_with_live_feed_and_failure`, `AttachRelayWorker` | `crates/bridget-transport/src/journal.rs:335`, `crates/bridget-daemon/src/wrapper.rs:1820` | événements `turn_start`/`update`/`turn_end` |
| Service de fond | script 095 launchd/systemd embarqué | `scripts/federate-ssh.sh:194-236, 426-436` | `bridget t3 serve` installé comme la fédération |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Client HTTP local | aucun ; seul `tungstenite` (handshake WebSocket) | `crates/bridget-transport/Cargo.toml:10`, `Cargo.lock` sans reqwest/ureq/hyper | arbitré ci-dessous |
| Identifiant stable dérivé du fil | `uuid` v4 seulement | `Cargo.toml:17` (`features = ["v4","serde"]`) | activer la fonctionnalité `v5` de la dépendance existante |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| aucune | — | — | — |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` XIX | dépendance nouvelle seulement justifiée par écrit | arbitrage `minreq` ci-dessous |
| `~/.speckit/constitution.md` XX §6 | privacy gate : secrets hors journaux | jeton 0600, jamais journalisé, révoqué au retrait |
| `~/.speckit/constitution.md` XVIII | complexité documentée | sondage O(fils) par passe, cadence bornée |
| `~/.speckit/ref/standards-tests.md` | un `.feature` par spec | `tests/features/098-t3code-adapter.feature` |
| Mémoire `bridget-tests-umask-077-env-prive` | gates en environnement privé | quickstart |
| `AGENTS.md` | pas de daemon/wrapper sans état isolé | faux serveur t3code et état Bridget privés dans les tests |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 096 federate-cli | commande d'administration embarquée, refus sans terminal, idempotence | modèle de `bridget t3` |
| 095 federation-services | service launchd/systemd, statut, retrait | modèle de `bridget t3 serve` en service |
| 097 claude-sans-tmux | présence par canal réel, refus sans voie de remise, journal des tours | contrat de présence `t3code | cli` |
| 090 codex-interactif | harnais Python avec faux serveur | faux serveur HTTP t3code |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg "HTTP/1.1|Content-Length|TcpStream::connect"` | crates/*/src | aucun client HTTP |
| `rg "^(reqwest|ureq|hyper|http) " Cargo.lock` | racine | absent |
| `rg "Uuid::new_v5"` | crates | absent ; `uuid` sans `v5` |
| `rg "launchctl|systemd|plist" scripts/federate-ssh.sh` | scripts | service géré aux lignes 194-236, 426-436 |
| `ls /Applications`, `~/.t3/userdata` | poste | app « T3 Code (Alpha) » 0.0.40 (Electron, `app.asar`), pas de CLI `t3` dans le PATH, serveur non lancé, `settings.json` presque vide |
| `ps -Eww` sur `bridget mcp` | poste | `CLAUDE_CODE_SESSION_ID` transmis par Claude Code, rien par Codex |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Client HTTP | créer : dépendance `minreq` sans TLS (`default-features = false`) | aucun client dans le workspace ; usage loopback JSON uniquement ; écrire un parseur HTTP/1.1 maison (chunked, en-têtes) serait plus de code et de surface qu'une crate de 1 dépendance sans TLS ; `reqwest`/`ureq` écartés (TLS et arbre lourds) | 2026-09-14 |
| UUID stable par fil | étendre `uuid` avec la fonctionnalité `v5` | dépendance déjà présente ; pas de hachage maison | 2026-09-14 |
| Commande `t3 auth` | exiger le CLI `t3` (npm) comme prérequis ; refus nommé s'il manque | l'app installée est un Electron sans binaire CLI ; écrire dans la base d'auth de t3code serait « trifouiller » | 2026-09-14 |
| Service de fond | créer : plist LaunchAgent / unité systemd écrits par `t3code::service` (~60 lignes) | le script 095 (scripts/federate-ssh.sh:194-236, 426-436) est spécialisé SSH : cible, clé d'hôte, runner autonome, reçus ; y greffer un service sans SSH aurait ajouté plus de couture et de branches que le bloc dédié ; duplication légère assumée et commentée | 2026-09-14 |
| Identité de fil | réutiliser `uuid` v5 puis forcer la présentation v4 (`stable_uuid`) | `bridget_core::router::validate_agent_id` exige un v4 canonique (crates/bridget-core/src/router.rs:12) ; l'identité reste déterministe | 2026-09-14 |
| Nom humain du fil | réutiliser `WrapperToDaemon::DisplayNameSet` (crates/bridget-daemon/src/communication/client.rs:48) | aucun nouveau champ dans Register ; refus de doublon géré par le daemon | 2026-09-14 |
| Session partagée | créer `t3code::Session` (renouvellement unique par génération) | aucun équivalent ; le wrapper n'a pas de jeton HTTP à renouveler | 2026-09-14 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
