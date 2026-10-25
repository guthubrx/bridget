# Archives techniques de l’adaptateur T3

Document d’origine : [specs/165-cycle-vie-liens-t3/reuse-audit.md](../../../../../../specs/165-cycle-vie-liens-t3/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Statut: PASS
Date: 2026-10-14
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/specs/165-cycle-vie-liens-t3

<a id="extrait-2"></a>

## Extrait 2

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

<a id="extrait-3"></a>

## Extrait 3

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/Users/moi/.speckit/constitution.md` | Réutiliser avant de créer | Extension des types, fonctions et boucles présents. |
| `.specify/memory/constitution.md` | Preuves et compatibilité | Tests V1, V2 et recette runtime requis. |
| `.specify/memory/standards.md` | Charger les références ciblées | Références Rust, tests et observabilité lues. |
| `AGENTS.md` | Diagnostiquer avant de modifier | Snapshot T3 et `lsof` ont établi la cause. |
| `AGENTS.md` | Tests obligatoires | Tests ciblés puis suite proportionnée au risque. |

<a id="extrait-4"></a>

## Extrait 4

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC-149 | Un sous-agent est une projection historique de tâche | Le lien n'est pas permanent après l'état terminal. |
| SPEC-147 | La fermeture et la reprise conservent les données T3 | Aucun historique n'est supprimé. |

<a id="extrait-5"></a>

## Extrait 5

| Requete | Portee | Resultat |
|---|---|---|
| `rg "ThreadSummary|is_live|turn_is_final"` | `crates/bridget-daemon/src/` | Types et règles réutilisables trouvés. |
| `rg "Link|spawn_reader|drive_queue"` | `crates/bridget-daemon/src/t3code.rs` | Cycle de vie et lecteur existants trouvés. |
| `rg "subagent|projection|lineage"` | `specs/147-*` et `specs/149-*` | Contrats historiques compatibles trouvés. |
| Inspection du snapshot T3 et de `lsof` | Runtime local | Croissance causée par les liens historiques confirmée. |
