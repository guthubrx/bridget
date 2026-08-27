# Modèle de données — Session 038

## MarkerSource

- `host` : nom d'hôte mesuré par le scanner, non fourni comme autorité par
  l'appelant ; chaîne non vide sans caractère de contrôle.
- `marker_directory` : chemin absolu canonique du répertoire scanné.

La valeur est optionnelle dans une politique v1 lue par la garde, mais
obligatoire pour toute régénération.

## MarkerInventory

- `version` : version fermée du contrat d'inventaire.
- `source` : `MarkerSource` réellement observée.
- `observed_at` : seconde Unix de l'observation.
- `complete` : vrai seulement si le répertoire et chaque entrée ont été lus.
- `live` : liste des couples principal/instance dont PID et naissance concordent.
- `stale` : marqueurs valides dont le processus n'existe plus ou dont la
  naissance diverge.

Un inventaire avec zéro entrée `live` n'est jamais consommable.

## PrincipalPolicy

- `principal` : identité déjà approuvée.
- `marker_source` : source attendue de ce principal.
- `actions` : ensemble existant, jamais modifié par la régénération.
- `instances` : grants existants ; une régénération vivante les remplace par un
  grant unique conservant expiration et révocation.

## RegenerationReport

- `applied` : indique si le chemin final a été remplacé.
- `previous_generation` / `next_generation` : génération lue et proposée/appliquée.
- `refreshed_principals` : principaux dont l'instance a changé.
- `unchanged_principals` : principaux déjà alignés.
- `dead_principals` : principaux approuvés absents et conservés.
- `unapproved_principals` : marqueurs vivants ignorés faute d'approbation.

Le rapport ne contient jamais `attestation_key`.

## Transitions

```text
politique N + inventaires complets + changement
    -> plan N+1
    -> (--apply) politique N+1 atomique

politique N + inventaires complets + aucun changement
    -> rapport inchangé, aucune écriture

politique N + source absente/incomplète/vide/ambiguë
    -> erreur, politique N octet-identique
```
