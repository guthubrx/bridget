# Audit de reutilisation de l'existant - 073-actions-agent-ui

## Decision

Statut: PASS
Date: 2026-08-30
Feature dir: `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/specs/073-actions-agent-ui`

Conclusion courte: Le plan etend les mecanismes deja livres au lieu de les reconstruire. La fiche SPEC-071, le fait de gestion `AgentInfo.persistent`, le contrat `StopOrder` et ses verdicts restent les sources uniques. Le seul contrat nouveau est un adaptateur HTTP local vers cet ordre existant, car aucune route UI d'arret n'existe aujourd'hui.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 12 |
| Items audites | 12 |
| Reutilisations deja prevues | 10 |
| Existants potentiellement pertinents | 2 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 2 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Projection d'eligibilite | `AgentInfo.persistent: Option<bool>` | `crates/bridget-transport/src/protocol.rs:2083` | `Some(true)` et `Some(false)` attestent un lancement gere, `None` interdit l'action. |
| Projection de ligne UI | `UiAgentRowV1` et `compose_agent_rows` | `crates/bridget-daemon/src/ui.rs:635`, `crates/bridget-daemon/src/ui.rs:1290` | Ajouter le fait brut sans nouvelle source de verite. |
| Adaptateur HTTP local | Routage et validation de `/v1/send` | `crates/bridget-daemon/src/ui.rs:765`, `crates/bridget-daemon/src/ui.rs:821`, `crates/bridget-daemon/src/ui.rs:889` | Creer une route distincte mais conserver le meme secret, le parseur et les reponses JSON du relais. |
| Lecture defensive avant mutation | `read_agent_list` | `crates/bridget-daemon/src/ui.rs:986` | Relire l'annuaire avant d'envoyer l'ordre pour refuser absent, arrete ou non gere. |
| Ordre de decommissionnement | `WrapperToDaemon::StopOrder` | `crates/bridget-transport/src/protocol.rs:1100`, `crates/bridget-daemon/src/cli.rs:460` | Aucune terminaison de processus n'est implementee dans le relais HTTP. |
| Verdict d'arret | `DaemonToWrapper::StopResult` et `StopOutcome` | `crates/bridget-transport/src/protocol.rs:1770`, `crates/bridget-daemon/src/daemon.rs:7550` | Conserver les issues propres, forcees, non gerees, absentes et expirees. |
| Fiche d'identite | Fiche globale SPEC-071 et `identityCardData` | `crates/bridget-daemon/assets/ui/app.js:4862`, `specs/071-identite-runtime-agent/tasks.md:42` | Transformer la fiche existante en panneau interactif, sans dupliquer les logos ni la normalisation runtime. |
| Positionnement du panneau | `identityCardPosition` | `crates/bridget-daemon/assets/ui/app.js:2817` | Conserver le calcul de placement dans la fenetre. |
| Historique apres arret | Section existante des agents arretes | `crates/bridget-daemon/assets/ui/app.js:5197`, `crates/bridget-daemon/assets/ui/theme.css:884` | Le rendu actuel separe deja actifs et arretes sans supprimer leurs fils. |
| Bancs de regression | Tests Node integres, tests Rust UI et relais | `crates/bridget-daemon/assets/ui/app.js:1`, `crates/bridget-daemon/src/ui.rs:2397`, `crates/bridget-daemon/tests/ui_relay_test.rs:1` | Etendre les bancs existants avec des cas SPEC-073, sans nouveau framework. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Panneau interactif non modal | Selecteur d'apparence d'agent | `crates/bridget-daemon/assets/ui/index.html:80`, `crates/bridget-daemon/assets/ui/app.js:6136` | Reutiliser seulement les conventions de fermeture et d'etat. Sa structure et son ancrage ne conviennent pas a une fiche globale de ligne. |
| Confirmation accessible | Aucun dialogue ou `alertdialog` existant | recherche `rg "role.?=.?dialog|aria-modal|alertdialog" crates/bridget-daemon/assets/ui` sans resultat | Creer le composant DOM natif minimal prevu, sans dependance. |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucun | Aucun | L'unique arret gere reste `StopOrder` ; aucune route UI d'arret ni confirmation modale n'existe | Aucune |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/home/moi/.speckit/constitution.md` | Reutiliser avant de creer | La fiche, le contrat d'arret, le superviseur et les verdicts existants restent uniques. |
| `/home/moi/.speckit/constitution.md` | Minimalisme et charge future | Aucun service, stockage ou paquet nouveau ; un adaptateur HTTP local seulement. |
| `/home/moi/.speckit/constitution.md` | Isolation par session | Le travail reste dans le worktree et la branche SPEC-073. |
| `/home/moi/.speckit/ref/standards-tests.md` | Tester le comportement et tracer la spec | Les nouveaux cas portent l'identifiant 073 dans leurs noms ou descriptions et couvrent les parcours utilisateur. |
| `/home/moi/.speckit/ref/code-quality-details.md` | Validation aux frontieres | Le corps HTTP est valide une fois par le relais avant l'ordre interne type. |
| `/home/moi/.speckit/ref/code-quality-details.md` | Erreurs explicites et idempotence | Aucun succes optimiste ; blocage d'une seconde requete en vol et verdict exact du daemon. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| `specs/071-identite-runtime-agent` | Fiche globale, catalogue runtime, logos locaux, valeurs inconnues non inferees | SPEC-073 transforme le declenchement et ajoute des actions sans recreer l'identite. |
| `specs/009-daemon-spawn` | `bridget stop`, arret synchrone borne, etat `stopped`, retrait du marqueur persistant | L'UI appelle ce cycle de vie exact et conserve ses garanties. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg "UiAgentRowV1|compose_agent_rows|/v1/send|post_ui_message"` | `crates/bridget-daemon/src/ui.rs` | Projection et patron de route locale identifies. |
| `rg "StopOrder|StopResult"` | `crates/`, `specs/` | Contrat et implementations d'arret uniques identifies. |
| `rg "agent-identity-card|identityCardPosition|renderAgentButton"` | `crates/bridget-daemon/assets/ui` | Fiche, positionnement et ligne SPEC-071 identifies. |
| `rg "role.?=.?dialog|aria-modal|alertdialog"` | `crates/bridget-daemon/assets/ui` | Aucun dialogue de confirmation reutilisable. |
| `rg "identity|runtime|provider|logo|stop"` | `specs/*` | SPEC-071 et SPEC-009 sont les deux precedents directs. |
| `rg "serde_json|tokio|http"` | `Cargo.toml`, `crates/bridget-daemon/Cargo.toml` | Les dependances necessaires sont deja presentes ; aucun ajout requis. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Route HTTP d'arret | creer nouveau | Aucune route equivalente n'existe ; elle adapte strictement `StopOrder` et ne porte aucun cycle de vie propre. | 2026-08-30 |
| Selecteur d'apparence | approfondir sans reutiliser le composant | Son comportement d'ouverture est utile, mais son contenu, son emplacement et son cycle de focus sont differents. | 2026-08-30 |
| Dialogue de confirmation | creer nouveau | Aucun dialogue accessible n'existe dans l'UI embarquee et aucune dependance n'est justifiee pour ce besoin borne. | 2026-08-30 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
