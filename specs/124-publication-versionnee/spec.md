# Spécification 124 - Publication versionnée, CHANGELOG et README à jour

## Fiche synthèse
Spec: 124-publication-versionnee | Statut: Implemented | Priorité: P2 | Date: 2026-09-25
Branche: session-124-publication-versionnee | Demande de l'utilisateur après la republication du 25/09.

## Problème observé
Le dépôt public était refait de zéro à chaque publication : un commit unique forcé, sans tag, sans
historique ni moyen de comparer deux versions. Pas de journal des changements ; README resté
en retrait des capacités récentes (et du nombre de tests).

## Exigences
- **FR-001** : une publication s'ajoute à l'historique public (commit et tag `vX.Y.Z`), sans force.
- **FR-002** : `CHANGELOG.md` versionné dans le dépôt de travail et publié ; sa section de version
  sert de message de commit et de tag.
- **FR-003** : confidentialité inchangée : arbre anonymisé contrôlé, message de version contrôlé
  contre les mêmes motifs interdits, auteur et publieur `guthubrx` en adresse noreply, sans
  modifier la configuration git.
- **FR-004** : la version annoncée (`Cargo.toml`, `bridget --help`) est celle du tag.

## Hors périmètre
Retrouver l'historique des publications antérieures : écrasé, non récupérable. `75a910f` devient
`v0.1.0`, point de départ.
