# ADR 012 — Activer les outils de pilotage par release admise

- **Date** : 2026-08-25
- **Statut** : Accepté
- **Portée** : `bridget-idle`, `bridget-ronde` et leurs installateurs

## Contexte

Un lien depuis `/Users/user/.local/bin/bridget-idle` vers un worktree a rendu
actif avant jury et merge.
Le code était propre, commité, poussé et identique à sa source : l'incident se
situe à la frontière chantier/production, pas dans la qualité du lot.

Le lien avait deux conséquences : une activation non conditionnée par le
verdict et une dépendance de production au cycle de vie du worktree. Un lien
vers le checkout principal après merge évite la seconde conséquence à
l'instant T, mais laisse toute future modification du checkout devenir active
sans activation explicite.

## Décision

Une commande de pilotage active sera un lien vers une release utilisateur
matérialisée hors de tout dépôt, sous un répertoire nommé par le SHA complet.
L'artefact sera extrait de l'objet Git du commit, accompagné d'une preuve
d'origine lisible, puis activé atomiquement.

La matérialisation n'est autorisée que depuis le checkout principal, sur la
branche `main`, avec un arbre propre et un `HEAD` ancêtre de `origin/main`.
Une entrée existante différente exige `--force`. Une release portant déjà le
même SHA mais des octets différents est refusée sans possibilité de forcer.

## Conséquences positives

- Le merge précède mécaniquement l'activation ; aucun canary clandestin ne
  peut passer par l'installateur.
- Supprimer ou modifier un worktree ne touche plus la production.
- `readlink` expose le SHA actif ; le fichier d'origine explique sa provenance.
- Un rollback vers un commit déjà admis reste possible et explicite.
- Les deux outils partagent une seule politique de frontière.

## Conséquences négatives

- Chaque SHA activé conserve une petite copie de chaque outil ; le nettoyage
  automatique est différé.
- La première migration depuis les anciens liens exige `--force`.
- L'installateur dépend de la fraîcheur locale de `origin/main` et demande un
  `git fetch origin` explicite en cas de doute.
- Le verdict n'est pas signé séparément : l'admission sur `origin/main` reste
  l'autorité du workflow.

## Alternatives rejetées

- **Lien vers le checkout principal** : encore mutable sans activation.
- **Copie directe dans `~/.local/bin`** : origine et rollback indiscernables.
- **Refus fondé seulement sur le chemin `.worktrees`** : ne prouve pas que le
  commit a été admis.
- **Fetch automatique** : effet réseau implicite au sein d'un installateur.
