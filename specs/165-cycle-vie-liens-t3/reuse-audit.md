# Audit de reutilisation de l'existant — Cycle de vie des liens T3

## Decision

Statut: PASS
Date: 2026-10-14
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/specs/165-cycle-vie-liens-t3

Conclusion courte: Le plan étend les contrats et la boucle de réconciliation existants. Il ne crée aucun service, protocole ou stockage. L'audit a remplacé une règle de délai neuve par les états existants et a ajouté la jonction du lecteur déjà créé.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 8 |
| Items audites | 8 |
| Reutilisations deja prevues | 8 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 5 |
| Specs existantes applicables | 2 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Présence d'un fil T3 | `ThreadSummary::is_live()` | `crates/bridget-daemon/src/t3code_contract.rs:280` | La règle archive, suppression et rangement reste inchangée. |
| Filiation V2 | Contrat T3 déjà établi | `specs/149-sous-agents-lineage/research-t3.md:9` | Le champ interne est neuf ; la source de vérité V2 existe déjà. |
| États terminaux | `turn_is_final()` | `crates/bridget-daemon/src/t3code.rs:3361` | La nouvelle décision réutilise cette liste. |
| Ensemble de liens | `BTreeMap` et réconciliation | `crates/bridget-daemon/src/t3code.rs:1173` | Aucun registre parallèle de connexions n'est ajouté. |
| Fermeture d'un lien | `Link::close()` | `crates/bridget-daemon/src/t3code.rs:1381` | La finalisation est étendue aux liens morts et joint le lecteur. |
| Lecteur réseau | `spawn_reader()` | `crates/bridget-daemon/src/t3code.rs:1389` | Son handle existant devient possédé par `Link`. |
| Filtrage des projections natives | `is_native_projection()` | `crates/bridget-daemon/src/t3code_contract.rs:197` | Ce filtre reste indépendant du cycle de vie. |
| Remise de travail | `drive_queue()` | `crates/bridget-daemon/src/t3code.rs:1985` | La file existante participe au drain ; sa règle de remise reste inchangée. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Aucun | Aucun | — | Aucune |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucun | Aucun | — | Aucune |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/Users/moi/.speckit/constitution.md` | Réutiliser avant de créer | Extension des types, fonctions et boucles présents. |
| `.specify/memory/constitution.md` | Preuves et compatibilité | Tests V1, V2 et recette runtime requis. |
| `.specify/memory/standards.md` | Charger les références ciblées | Références Rust, tests et observabilité lues. |
| `AGENTS.md` | Diagnostiquer avant de modifier | Snapshot T3 et `lsof` ont établi la cause. |
| `AGENTS.md` | Tests obligatoires | Tests ciblés puis suite proportionnée au risque. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC-149 | Un sous-agent est une projection historique de tâche | Le lien n'est pas permanent après l'état terminal. |
| SPEC-147 | La fermeture et la reprise conservent les données T3 | Aucun historique n'est supprimé. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg "ThreadSummary|is_live|turn_is_final"` | `crates/bridget-daemon/src/` | Types et règles réutilisables trouvés. |
| `rg "Link|spawn_reader|drive_queue"` | `crates/bridget-daemon/src/t3code.rs` | Cycle de vie et lecteur existants trouvés. |
| `rg "subagent|projection|lineage"` | `specs/147-*` et `specs/149-*` | Contrats historiques compatibles trouvés. |
| Inspection du snapshot T3 et de `lsof` | Runtime local | Croissance causée par les liens historiques confirmée. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Signal de démarrage | reutiliser | Les états préparation, file et exécution donnent déjà le signal d'ouverture. | 2026-10-14 |
| États terminaux | reutiliser | `turn_is_final()` évite une deuxième liste divergente. | 2026-10-14 |
| Fin du lecteur | reutiliser | Posséder et joindre le handle complète `Link::close()` sans nouveau composant. | 2026-10-14 |
| Dernier résultat | reutiliser | `change_key()`, `refresh()`, le journal et l'ACK fournissent un drain déterministe. | 2026-10-14 |
| Reprise durable | reutiliser | `ThreadState::load()` porte déjà les obligations après redémarrage. | 2026-10-14 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
