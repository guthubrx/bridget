# Audit de reutilisation de l'existant — 136

## Decision

Statut: PASS
Date: 2026-10-06
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles/specs/136-messages-utiles

Extension du fil102. Aucun service ou contrôleur parallèle. Le remplacement
explicite n'existe pas ; deux colonnes et un enum sont nécessaires.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 8 |
| Items audites | 8 |
| Reutilisations deja prevues | 7 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 4 |
| Specs existantes applicables | 4 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Classification du dépôt | ThreadAction::Post | crates/bridget-transport/src/protocol.rs:2187 | Étendre contrat, pas nouveau transport |
| Publication silencieuse | ThreadNotify | crates/bridget-transport/src/protocol.rs:2159 | history impose liste vide |
| Conservation/remplacement | thread_post/discussion_entries | crates/bridget-daemon/src/store/threads.rs:819 | Deux colonnes, relation unique, transaction existante |
| Projection bornée | read_range | crates/bridget-daemon/src/store/threads.rs:623 | Jointure indexée, aucune synthèse |
| Rejeu de lecture | thread_read | crates/bridget-daemon/src/store/threads.rs:1104 | Snapshot et reçu réutilisés |
| Histoire exacte | thread_history | crates/bridget-daemon/src/store/threads.rs:1250 | Pas de suppression |
| Façade CLI | parse_thread_args | crates/bridget-daemon/src/cli.rs:1076 | Deux options, strictes |
| Façade MCP | bridget_thread | crates/bridget-daemon/src/mcp.rs:1879 | Schéma existant étendu |

## Existant potentiellement pertinent non mentionne

Aucun. Le contrôleur135 n'est pas un remplacement de la publication structurée.

## Duplications evidentes

Aucune après recherche par nom et responsabilité.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| AGENTS.md | session/worktree, autorisation |136 approuvée,135 préservée |
| Constitution XVIII | complexité | jointure indexée, bornes pages/membres |
| Constitution XIX/XX | réutilisation/charge future | pas de bus ni IA, état dérivé |
| Standards tests | comportement/Gherkin | feature136 puis tests Rust du contrat |

Mémoire personnalisée Bridget: aucun MEMORY.md trouvé dans le chemin déclaré.

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
|102| silence, historique, reçus, coalescence | réutilisé |
|104| relecture exacte, provenance | pas de résumé sans preuve |
|114/130| lots bornés | ne suffit pas à éviter les corps périmés |
|135| mission/progrès/disposition | distinct, pas dupliqué |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| rg supersed/replaces/revision | core, transport, daemon | pas de remplacement d'entrée |
| rg silent/silenc/ThreadNotify | threads/store/t3code | mécanisme102 adapté |
| rg ThreadAction::Post/read_range/thread_read | contrat/CLI/MCP/store | points d'extension retrouvés |
| rg --files specs/docs/tests | artefacts | pas de136 ; Gherkin102 réutilisable |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
|Classe et remplacement|créer dans contrat existant|pas d'équivalent, nécessaires CLI/MCP/store|2026-10-06|
|Historique et wake|réutiliser102|même responsabilité, pas de table supplémentaire|2026-10-06|
|ADR|recherche136 tient l'ADR|pas de fichier ADR dédié redondant|2026-10-06|
|Direct legacy|conserver|classe absente, tout filtrage risquerait une perte|2026-10-06|
|Décodeur Option présent/null|créer dans protocol.rs|absence et null doivent différer ; aucun décodeur existant équivalent, deux champs concernés, contrainte de compatibilité écrite|2026-10-06|
|Borne ACTION_MAX_BYTES|créer dans threads.rs|BODY_MAX_BYTES existant16Kio reste utile au journal ; borne2048 distincte demandée|2026-10-06|
|Helper de test post136|créer dans harnais102|porte le nouveau contrat, réutilise Client/thread/Quartet existants ; aucun second harnais ou faux fournisseur|2026-10-06|

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
