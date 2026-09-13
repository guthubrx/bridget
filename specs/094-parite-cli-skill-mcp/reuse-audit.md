# Audit de reutilisation de l'existant — 094

## Decision

Statut: PASS
Date: 2026-09-07
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/091-communication-agent-ux/specs/094-parite-cli-skill-mcp

Le plan étend les façades et gardes existantes. Aucun nouveau stockage, service
ou protocole. Le renfort de frontière est nécessaire à l'exposition, pas une
seconde autorité. Audit auteur et revue indépendante rapprochés du code.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 9 |
| Items audites | 9 |
| Reutilisations deja prevues | 9 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 4 |
| Specs existantes applicables | 5 |

## Reutilisations correctement identifiees

Préfixe absolu des preuves : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/091-communication-agent-ux/

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| rename | rename_display_name | crates/bridget-daemon/src/communication/client.rs:46 | aucun nouveau handler métier |
| dnd | Availability et handle_availability | crates/bridget-daemon/src/daemon.rs:6551 | garder reçu, borner entrée |
| domain | Domain et agent-domains | crates/bridget-daemon/src/cli.rs:3255 | factoriser sauvegarde au client |
| runtime | Runtime Declared | crates/bridget-daemon/src/daemon.rs:6034 | déclaration pas sélection |
| status | get_status | crates/bridget-daemon/src/daemon.rs:12034 | projection assainie |
| control_status | ControlStateRead/History | crates/bridget-daemon/src/daemon.rs:9032 | Client/Lookup, portée dérivée |
| identité | live_connection_identity | crates/bridget-daemon/src/daemon.rs:7069 | appliquer à chaque mutation propre |
| permissions | listes wrapper | crates/bridget-daemon/src/wrapper.rs:4424 | étendre listes fermées |
| documentation | skill et README | skills/bridget/SKILL.md | référence exhaustive liée, pas double manuel |

## Existant potentiellement pertinent non mentionne

Aucun.

## Duplications evidentes

Aucune : les nouvelles entrées MCP portent validation/autorité/projection ;
pas de simple duplication de commandes sans responsabilité.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| constitution globale | XIX/XX | réutiliser, complexité bornée |
| constitution projet | source globale | sync à jour |
| AGENTS | worktree/ownership | cwd agent vivant préservé, branche094 |
| skill Bridget | canal unique et UUID | aucun accès shell alternatif ni cible libre |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 001 | nom révisé | conserver reçus |
| 005 | DND/domaine | mêmes effets/reconnexion |
| 039 | identité MCP | résoudre à chaque appel |
| 089 | client borné | partagé CLI/MCP |
| 091 | politiques fournisseurs | listes exactes et test never |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| rg rename/Availability/Domain | CLI/client/daemon | gardes manquantes aux deux dispatchs |
| rg Runtime/Declared | CLI/daemon | garde limitée Declared, hooks préservés |
| rg ControlStateRead/ControlHistory | MCP/daemon/CLI | Lookup suffit à lecture |
| lecture reprise/reaper | CLI/modules | lecteurs machine/écriture, restent humains |
| aide binaire + répartiteur + tools | CLI/MCP | inventaire complet, alias sans doublons |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| domaine sauvegarde | réutiliser/factoriser | erreurs actuellement ignorées | 2026-09-07 |
| gardes daemon | étendre | usurpation prouvée sur nouvelles surfaces | 2026-09-07 |
| reaper/reprise | documenter CLI | aucune projection mince, fichiers locaux | 2026-09-07 |
| administration | documenter CLI | aucune autorité générale déléguée par ce lot | 2026-09-07 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
