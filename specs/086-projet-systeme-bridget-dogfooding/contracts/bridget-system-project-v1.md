# Contrat - Projet système Bridget v1

## Déclaration locale

La commande d'administration contient version, `command_id`, chemin source absolu et génération attendue du catalogue. Elle n'est pas exposée comme mutation projet standard.

## Attestation minimale

Le daemon vérifie sans exécution:

- repository Git ou worktree valide
- manifeste Cargo workspace attendu
- crates Bridget attendues
- plugin `plugins/maicie`
- cohérence du chemin avec le git common dir

Les noms exacts de marqueurs sont versionnés dans l'implémentation et tout champ inconnu est refusé.

## Invariants

- zéro ou un projet `bridget_system`
- emplacement `exact_project + system_only`
- backend Docker avant activation du dogfooding
- aucune promotion via create/import/update standard
- aucun mount source dans un projet standard

## Refus fermés

- `source_not_configured`
- `source_missing`
- `source_not_bridget`
- `system_project_already_exists`
- `catalog_generation_mismatch`
- `docker_required`
- `store_unavailable`

