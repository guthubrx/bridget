# Archives techniques de l’adaptateur T3

Document d’origine : [specs/134-noms-humains-messages/reuse-audit.md](../../../../../../specs/134-noms-humains-messages/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Libellé humain commun | `BridgetMessage::sender_label()` | `crates/bridget-core/src/message.rs:198` | Étendre la projection centrale, sans nouveau helper. |
| Profil remis avec le message | Enrichissement daemon déjà effectué sur les deux chemins de remise | `crates/bridget-daemon/src/daemon.rs:3951`, `crates/bridget-daemon/src/daemon.rs:13514` | Aucun second annuaire. |
| Réparation identité-profil | `AgentProfileStore::ensure_agent_ids()` transactionnel | `crates/bridget-daemon/src/agent_profile.rs:390` | Renforcer le contrat idempotent existant. |
| Libellé dans les lots T3 | `batch_envelope()` et `sender_label()` unitaire | `crates/bridget-daemon/src/t3code.rs:2982`, `crates/bridget-daemon/src/t3code.rs:3061` | Remplacer l’accès direct à `message.from` à la ligne 3008. |

<a id="extrait-2"></a>

## Extrait 2

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| `specs/110-reprise-nom` | Nom humain réattribué sans changer l’identité | Conserver l’UUID et réutiliser le registre de profils. |
| `specs/114-joignabilite` | Enveloppe unitaire et lots T3 bornés | Appliquer le même libellé à chaque élément du lot. |
| `specs/133-relais-sous-agents` | Provenance visible après l’identité du parent | Préserver le suffixe délégué existant. |

<a id="extrait-3"></a>

## Extrait 3

| Requete | Portee | Resultat |
|---|---|---|
| `rg -n "sender_label|from_display_name" crates/` | Cœur, daemon et transports | Une projection centrale et deux enrichissements daemon trouvés. |
| `rg -n "ensure_agent_ids|agent_profiles" crates/bridget-daemon/` | Registre SQLite | Un seul chemin transactionnel à renforcer. |
| `rg -n "batch_envelope|message.from" crates/bridget-daemon/src/t3code.rs` | Pont T3 | Un accès direct à l’UUID dans le rendu groupé. |
| `rg -n "display_name|batch_envelope" specs/110-* specs/114-* specs/133-*` | Specs livrées | Trois patterns compatibles à réutiliser. |
| `rg -n "serde|rusqlite|uuid" Cargo.toml crates/*/Cargo.toml` | Dépendances | Toutes les briques requises sont déjà présentes. |

<a id="extrait-4"></a>

## Extrait 4

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Format visible | reutiliser | Étendre `sender_label()` en `nom (UUID)` et garder son suffixe délégué. | 2026-10-05 |
| Profil absent | reutiliser | Renforcer `ensure_agent_ids()` plutôt que créer une migration ou une commande. | 2026-10-05 |
| Lots T3 | reutiliser | Appeler le même libellé pour chaque message du lot. | 2026-10-05 |
