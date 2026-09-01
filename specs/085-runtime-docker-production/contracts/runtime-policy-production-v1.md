# Contrat de configuration - Politique runtime de production

## Champs fermés

- identifiant et version de politique
- référence d'image épinglée par digest
- UID/GID non root
- limites CPU, mémoire et PIDs
- tailles tmpfs
- mode réseau parmi un enum supporté
- identifiants d'exécutables approuvés
- racine d'état hôte administrée

## Interdictions

- options Docker libres
- montage du socket Docker
- montage hôte fourni par un projet
- secret ou credential en clair
- tag mutable sans digest résolu
- mode privilégié, ajout de capacité ou désactivation de `no-new-privileges`

## Installation

Le service reçoit les chemins absolus de la politique racine, de la politique runtime et du catalogue de ressources. Les fichiers appartiennent à l'opérateur et ne sont pas modifiables par les conteneurs.

