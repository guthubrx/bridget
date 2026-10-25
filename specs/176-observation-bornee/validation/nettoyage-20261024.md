# Nettoyage après livraison — 24 octobre 2026

Nettoyage demandé explicitement par l'utilisateur : commit, intégration, push,
puis retrait des worktrees, branches et builds devenus inutiles.
Périmètre : dépôt Bridget sur Mac et Cartae. Aucun autre projet nettoyé.

## Git et publication

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/176-observation-bornee/validation/nettoyage-20261024.md#extrait-1).

## Retraits effectués

| Ressource | Mac | Cartae |
|---|---:|---:|
| Worktrees secondaires | 1 | 41 |
| Branches locales | 0 | 41 |
| Binaires et sorties de compilation | 154 | 42 |
| Profils de cache Cargo nettoyés | 1 | 3 |
| Octets logiques des binaires et caches retirés | 4 426 708 545 | 6 027 672 589 |

Total : 42 worktrees, 41 branches locales, 2 branches distantes,
196 binaires ou sorties de compilation et 4 profils de cache.
Les fichiers retirés représentent 10 454 381 134 octets logiques, soit environ
10,45 Go. Ce chiffre ne mesure pas l'espace physique rendu au système de fichiers.

Les worktrees ont été retirés par Git sans forçage. Les fichiers ignorés
présents dans les espaces retirés ont été conservés : 244 fichiers déplacés
avec vérification SHA-256. Aucun travail non commité n'a été supprimé.

Trois branches étaient équivalentes par patch à des changements déjà intégrés.
Leurs commits d'origine ont été sauvegardés dans un bundle Git vérifié avant
retrait des références. Les autres branches retirées étaient fusionnées.

## Conservations

Il reste 9 worktrees secondaires : 1 sur Mac et 8 sur Cartae.
Six portent des fichiers non intégrés ; deux conservent un ancien correctif
non fusionné ; un appartient à une tâche dont l'annulation n'est pas terminée.
Ces états ne permettent pas leur retrait.

Sur Cartae, 25 anciens binaires restent conservés. Une observation complète
avait permis le premier lot. La fin du lot a ensuite été refusée car
l'observation des fichiers ouverts était incomplète. Le diagnostic lsof
mentionne fuse.portal ; la seconde lecture sans stderr ne satisfait toujours
pas le contrôle des enregistrements. Aucune absence d'usage n'en est déduite.
Un premier échec de permission sur un ancien binaire détenu par root n'a
supprimé aucun fichier supplémentaire ; les reçus distinguent les intentions
des suppressions réellement constatées.

Le binaire optimisé actif et une version récente de retour arrière restent
présents sur chaque machine. Les caches récents réutilisables sont conservés.
Les dossiers d'anciennes versions qui portent encore des sources ou des reçus
restent présents. Seuls 49 répertoires devenus vides ont été retirés sur Cartae.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/176-observation-bornee/validation/nettoyage-20261024.md#extrait-2).

## Vérification et preuves

Les contrôles portent sur Git, les processus utilisant les fichiers, les
verrous Cargo, les empreintes et les résultats des suppressions.
Aucun code produit n'est modifié par ce nettoyage ; aucune nouvelle suite
de tests, compilation ou campagne d'audit n'est nécessaire.

Les reçus détaillés et les fichiers conservés restent hors Git dans les
dossiers privés bridget-cleanup-20261024 sur les deux machines. Une archive
des preuves Cartae est également conservée sur Mac.

Le worktree source de la session 176 a été retiré après publication.
Ses sources restent reconstructibles depuis le commit de livraison
78d5278d ; ses fichiers ignorés et les preuves d'audit restent archivés.
Les anciens chemins de worktree dans les reçus précédents sont donc historiques.
