# Audit de reutilisation de l'existant — 093

## Decision

Statut: PASS
Date: 2026-09-06
Feature dir: specs/093-attach-conversation

Lecture indépendante de l'équipier, confirmée par inspection du pilote. Le
renderer existant porte déjà blocs, bornes et largeur. Aucun doublon de service,
transport, journal ni base proposé. Seul le parser Markdown manque.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 7 |
| Items audites | 7 |
| Reutilisations deja prevues | 5 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 4 |
| Specs existantes applicables | 3 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Accumulation | TurnBlock | crates/bridget-daemon/src/attach.rs:1103 | Brute et bornée, ne pas doubler |
| Disposition et resize | BlockRenderer | crates/bridget-daemon/src/attach.rs:1518 | Étendre refresh_geometry:1858 |
| Styles/sécurité | sanitize_data, écriture CRLF | crates/bridget-daemon/src/attach.rs:2037 | Pas d'ANSI extérieur |
| Largeur | unicode-width | crates/bridget-daemon/Cargo.toml:31 | Mesurer avant ajout styles |
| Test terminal | PseudoTerminal | crates/bridget-daemon/src/attach.rs (tests) | Réutiliser PTY et renderer réel |
| Parser CommonMark | Aucun | Cargo.toml/Cargo.lock, rg sans résultat | Ajouter pulldown-cmark sans backend HTML |
| Projection Markdown pure | Aucun | attach.rs:1204 .lines texte brut | Module privé attach_renderer.rs justifié |

## Existant potentiellement pertinent non mentionne

Aucun. Présence ListAgents déjà prévue ; pas de lookup supplémentaire.

## Duplications evidentes

Aucune. termimad et son moteur de disposition sont écartés pour éviter de doubler
BlockRenderer ; parseur artisanal écarté pour ne pas reconstruire CommonMark.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| AGENTS.md | cwd autorisé | Même worktree physique, branche093 |
| Constitution XIX/XX | réemploi | Renderer conservé, parser documenté |
| Constitution XVIII | bornes | O(n) par bloc borné et batch existant |
| Skill Bridget | journal fait autorité | Filtrage à la vue uniquement |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 089 | communication indépendante GUI | Aucun canal supplémentaire |
| 091 | footer/status/model | Conserver données attestées |
| 092 | clavier/historique | Aucun remplacement InputBuffer |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| rg markdown/pulldown/termimad/syntect | Cargo et crates | Aucun parser |
| rg render/resize/terminal_columns | attach.rs | BlockRenderer et geometry |
| Lecture 091/092 plan | specs | Réemplois confirmés |
| Audit Agent23 9ae2a7b0f2474 | renderer et manifestes | Même proposition, indent UUID identifié |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Parser | créer dépendance ciblée | CommonMark, pas de parser maison | 2026-09-06 |
| Module pur | créer attach_renderer.rs | Isoler sécurité/style/largeur testables sans nouveau moteur écran | 2026-09-06 |
| Resize | réutiliser géométrie | Reflow bloc géré, pas scrollback bis | 2026-09-06 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
