# Audit de reutilisation de l'existant — Noms humains dans les messages Bridget

## Decision

Statut: PASS
Date: 2026-10-05
Feature dir: specs/134-noms-humains-messages

Conclusion courte: Le plan étend les points de présentation, d’enrichissement et
d’enregistrement déjà présents. Aucun service, stockage, transport ou concept
concurrent n’est créé. Les quatre items extraits possèdent un existant direct à
réutiliser et aucune duplication ne reste à arbitrer.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 4 |
| Items audites | 4 |
| Reutilisations deja prevues | 4 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 5 |
| Specs existantes applicables | 3 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Libellé humain commun | `BridgetMessage::sender_label()` | `crates/bridget-core/src/message.rs:198` | Étendre la projection centrale, sans nouveau helper. |
| Profil remis avec le message | Enrichissement daemon déjà effectué sur les deux chemins de remise | `crates/bridget-daemon/src/daemon.rs:3951`, `crates/bridget-daemon/src/daemon.rs:13514` | Aucun second annuaire. |
| Réparation identité-profil | `AgentProfileStore::ensure_agent_ids()` transactionnel | `crates/bridget-daemon/src/agent_profile.rs:390` | Renforcer le contrat idempotent existant. |
| Libellé dans les lots T3 | `batch_envelope()` et `sender_label()` unitaire | `crates/bridget-daemon/src/t3code.rs:2982`, `crates/bridget-daemon/src/t3code.rs:3061` | Remplacer l’accès direct à `message.from` à la ligne 3008. |

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
| `AGENTS.md` | Worktree, tests réels, chemins absolus et aucune trace d’IA dans Git | Session isolée, validations fraîches et commits sobres. |
| `.specify/memory/constitution.md` | La constitution globale est obligatoire | Gates relues avant planification. |
| `/Users/moi/.speckit/constitution.md` Articles XVIII–XX | Complexité, minimalisme et responsabilité future | O(1) pour le rendu, O(n) pour l’enregistrement, aucune abstraction ou dépendance. |
| `/Users/moi/.speckit/ref/code-quality-details.md` | Un `ensure_*` doit être idempotent et transactionnel | Le profil manquant est réparé dans la transaction existante. |
| `/Users/moi/.speckit/ref/standards-tests.md` | Tests de comportement traçables par identifiant de spec | Les nouveaux tests portent le préfixe `spec134`. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| `specs/110-reprise-nom` | Nom humain réattribué sans changer l’identité | Conserver l’UUID et réutiliser le registre de profils. |
| `specs/114-joignabilite` | Enveloppe unitaire et lots T3 bornés | Appliquer le même libellé à chaque élément du lot. |
| `specs/133-relais-sous-agents` | Provenance visible après l’identité du parent | Préserver le suffixe délégué existant. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg -n "sender_label|from_display_name" crates/` | Cœur, daemon et transports | Une projection centrale et deux enrichissements daemon trouvés. |
| `rg -n "ensure_agent_ids|agent_profiles" crates/bridget-daemon/` | Registre SQLite | Un seul chemin transactionnel à renforcer. |
| `rg -n "batch_envelope|message.from" crates/bridget-daemon/src/t3code.rs` | Pont T3 | Un accès direct à l’UUID dans le rendu groupé. |
| `rg -n "display_name|batch_envelope" specs/110-* specs/114-* specs/133-*` | Specs livrées | Trois patterns compatibles à réutiliser. |
| `rg -n "serde|rusqlite|uuid" Cargo.toml crates/*/Cargo.toml` | Dépendances | Toutes les briques requises sont déjà présentes. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Format visible | reutiliser | Étendre `sender_label()` en `nom (UUID)` et garder son suffixe délégué. | 2026-10-05 |
| Profil absent | reutiliser | Renforcer `ensure_agent_ids()` plutôt que créer une migration ou une commande. | 2026-10-05 |
| Lots T3 | reutiliser | Appeler le même libellé pour chaque message du lot. | 2026-10-05 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
