> **Ronde r3 (binaire release r8) : voir la section « Ronde r3 » en fin de fichier.** Le tableau ci-dessous décrit la ronde r2 (binaire debug r5) ; les outils sont les mêmes, `fx.mjs` pointe maintenant vers le release r8.

# Recettes natives réseau 149 - T036 (interop) et T039 (scénarios dégradés) - ronde r2

Propriétaire : testeur Claude Sonnet 5.5 (claudeAgent), ronde natif/réseau r1.
Ces outils ne modifient aucun code de production, aucun autre dossier de recette et aucun service.
Ils n'utilisent aucun modèle.

## Ce que chaque couche est (réel ou simulé)

[Note technique de l’adaptateur](../../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-recipes/README.md#extrait-1).

Aucune preuve de modèle, aucune preuve de confinement OS, aucune preuve de la voie standalone GLM/Claude ici.
Ces points restent aux autres propriétaires (T037, T038).

## Fichiers

[Note technique de l’adaptateur](../../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-recipes/README.md#extrait-2).

## Exécution

[Note technique de l’adaptateur](../../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-recipes/README.md#extrait-3).

[Note technique de l’adaptateur](../../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-recipes/README.md#extrait-4).

## Garde-fous appliqués

[Note technique de l’adaptateur](../../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-recipes/README.md#extrait-5).


## Ronde r3 - binaire release r8

Binaire : `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-0a29ad9b2cdb` (SHA-256 `0a29ad9b2cdb88b1c19f95d9a9bfd1cd89292e269a92fa440864a25bdfa5dde6`). Résultats : `results-r3/` ; r2 archivés dans `results-r2/` (index `results-r2/INDEX.md`).
`fx.mjs` : `NATIVE149_BIN` / `NATIVE149_BIN_SHA256` rejouent un autre binaire ; `NATIVE149_RESULTS` change le dossier ; `NATIVE149_CODEX_PROVIDER` choisit le fournisseur fermé (`codex149b.py` en r3). `fx-r2.mjs.txt` et `recovery149-r2.ts` sont les versions r2.

[Note technique de l’adaptateur](../../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-recipes/README.md#extrait-6).

[Note technique de l’adaptateur](../../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-recipes/README.md#extrait-7).
