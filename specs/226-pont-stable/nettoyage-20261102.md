# Nettoyage demandé le 2 novembre 2026

Aucune compilation, aucun arrêt de service, aucune mutation de tâche ou de reçu
métier. Les contrôles de fichiers ouverts utilisent le lecteur natif de
scripts/build.py. Aucune observation incomplète n’est interprétée comme libre.

## Éléments retirés

- Mac : 5,50 Gio de caches par clean, puis 6,00 Gio par installed après
  égalité SHA-256 du binaire installé hors staging. Environ 11,50 Gio logiques
  supprimés. Le cache de compilation restant du checkout est conservé : 2,29 Gio
  logiques selon le script, environ 2,4 Go alloués selon du.
- Candidat T3 extrait, ancienne publication du 19 octobre avec son ancien
  retour, sorties générées et liens de dépendances du worktree intégré :
  1 474 731 184 octets logiques enregistrés, hors fichiers suivis retirés par Git.
  Les anciens petits manifestes et outils sont conservés dans le reçu privé.
- 31 copies de transfert et bundles entièrement intégrés supplémentaires :
  85 035 790 octets logiques enregistrés. Un premier passage s’est arrêté sur
  observation incomplète ; ses retraits antérieurs ne sont pas comptés dans ce
  sous-total, faute de reçu complet. Aucune suppression après ce refus.
- KS : image intermédiaire 226-bridge-stable-r2, 43 642 168 octets, retirée
  par le helper existant après observation native complète. Reçu distinct,
  sans écraser celui de la livraison précédente.
- Worktree T3 006-large-thread-command-reads, propre, intégré à
  local/main-20261009 et sans usage ouvert observé. Aucun fil T3 ne référence
  ce chemin dans sa projection au contrôle. Git worktree remove sans force.
- Branche locale et branche distante fork fix/large-thread-command-reads.
  Le commit 7a024e9b866c46be6491059a1218b985df19f37d reste sur la branche
  principale locale publiée et dans la version installée.
- Treize références distantes périmées de Bridget, purgées par fetch --prune :
  sept pour origin et six pour github, deux alias du même dépôt distant.
  Elles avaient déjà été supprimées du serveur.

Les nombres sont des tailles logiques, pas un gain physique de disque garanti.
Aucune suppression de la base utilisateur active. Aucun nettoyage Git global
avec --force ni suppression d’un travail non intégré.

## Preuves déplacées et éléments conservés

Les résultats, scripts de mesure et copie SQLite de 25 766 764 544 octets sont
préservés ensemble, par renommage sur le même volume, dans :
/Volumes/8TB2/50-repos-archives/11.Repositories/t3code/.t3-qualification-archives/226-volume-7a024e9b.
L’ancien chemin du worktree ne doit plus servir de référence active.

T3 : actif 0.1.0-local.20261102.volume, signature valide, serveur PID 23781, pont
running avec 19 fils au contrôle. Archive signée et application de retour du
2 novembre conservées. Modification préexistante pnpm-lock.yaml du checkout
T3 principal conservée. Les autres worktrees T3 appartiennent à des travaux
distincts ou non intégrés ; ils ne sont pas supprimés par cette tâche.

Bridget : actif 226-r3 et retour 225-r4 conservés sur quatre hôtes. Les cinq
processus Linux contrôlés chargent toujours SHA-256
7794a477407bff17d966bbd07d88052e3a42dad1a9ab671117795d648b9e683d ;
pointeurs rollback et absence d’activation concurrente vérifiés.
Mac : anciennes images installées conservées, verrou natif updater occupé ;
aucun arrêt du service pour nettoyer. L’image 178 reste aussi référencée par
un ancien LaunchAgent. Cartae : observation système des usages incomplète.
SYS : accès à l’observation système indisponible. Les images/worktrees sous
ces gardes restent conservés, ainsi que les caches récents sous budget.

Bridget distant conserve bridget198-worker-start, commit a8f66197, non intégré
et encore associé au worktree natif 1cc44f4d. Programme 221 et travaux 222/223
conservés. Les worktrees 224/225 ont des fichiers ignorés et des usages non
complètement observables sur Cartae ; aucun retrait forcé.
Archives de sources non prouvées identiques et preuves historiques conservées.

Reçus privés locaux :
/Users/moi/.cache/bridget226-pont-stable/cleanup-20261102.
Reçu KS : /home/moi/.cache/bridget226-pont-stable/cleanup-20261102.json.
