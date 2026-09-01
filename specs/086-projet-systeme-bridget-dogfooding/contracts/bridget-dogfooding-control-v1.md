# Contrat - Pilotage du dogfooding Bridget v1

## Lecture

La projection expose disponibilité, mode confirmé, génération, project id opaque, backend, epoch et raison fermée. Le chemin complet n'est montré que dans la surface locale expert déjà autorisée.

## Prévisualisation

La requête contient:

- `contract_version`
- `command_id`
- `expected_generation`
- `expected_project_generation`
- `requested_mode`: disabled ou enabled

La réponse décrit la recréation requise, les agents actifs éventuels et les limites qui restent en vigueur.

## Application

L'application reprend exactement la prévisualisation confirmée. Le daemon:

1. revérifie projet, backend, agents et générations;
2. calcule les mounts;
3. recrée et atteste l'environnement;
4. publie mode et nouvel epoch;
5. compense vers l'ancien mode en cas d'échec.

En mode enabled, le checkout principal reste read-only. Bridget exige un worktree attribué, non principal et hors branche `main`; seuls ce worktree et les métadonnées Git nécessaires sont writable.

## Limites permanentes affichées

L'activation ne vaut jamais autorisation automatique de merge main, push, installation, redémarrage ou déploiement. Les métadonnées Git sont partagées par le projet: ce mode ne promet aucune isolation entre agents du même conteneur.

## Refus fermés

- `system_project_unavailable`
- `docker_required`
- `environment_busy`
- `generation_mismatch`
- `attestation_failed`
- `recreate_failed`
- `compensation_failed`
