# Vérification 176

## Portée
Python d'observation et guides EN/FR. Aucun fichier Rust, protocole, droit ou service modifié. Les essais de retrait concernent seulement des dépôts temporaires. Les captures détaillées restent privées.

## Résultats
- Cartae : 60 tests du fichier, 59 réussis, 1 observation réelle ignorée quand lsof était incomplet. État avant les deux tests de revue finaux.
- Mac : 60 tests, 59 réussis, 1 essai de nettoyage de fixture ignoré quand l'observation était incomplète. Deux erreurs de préparation de cette recette ont été corrigées : Makefile absent du paquet privé et dossier Git info absent du modèle Git local.
- Delta final : 16 tests ciblés réussis sur Cartae et Mac, dont les cas FUSE sans avertissement, annotation d'exemption, portée échappée, préfixe brut, captures et budget. Les ajouts portent le fichier à 62 tests ; aucune nouvelle suite complète du dépôt n'est revendiquée.
- Processus réel depuis un autre cwd : fichier ouvert détecté, puis absence détectée après fermeture, sur les deux machines. Pas de simulation de lsof pour ces preuves.
- Binaire lsof Cartae 4.95.0 : exemption réelle sur /dev/shm conserve le fichier dans stdout. Le parser refuse cette observation. Un fichier mmap sous un voisin de préfixe donne UNKNmem et UNKNfd, également refusés.
- Dépassement réel du lecteur privilégié : TERM, retour -15, aucun survivant du groupe créé observé. Le code ne promet pas l'arrêt universel d'un outil hostile.
- Guides : 27 pages, 422 liens locaux, 11 paires linguistiques contrôlés.

Les premiers tests RED d'API manquante sont des erreurs de contrat, pas la reproduction d'un défaut de production. Les échecs intermédiaires de fixtures sont conservés dans les preuves privées ; ils ne sont pas présentés comme des RED métier.

## Mesures
Fixture privée : 12 commits partagés entre 32 puis 128 worktrees. Un passage par cas ; pas une distribution p95.

| Périmètre | Avant | Après |
|---|---:|---:|
| Appels Git, 32 worktrees | 129 | 58 |
| Appels Git, 128 worktrees | 513 | 154 |
| Phase Git/décision, 32, processus simulés vides | 0,196 s | 0,082 s |
| Phase Git/décision, 128, processus simulés vides | 1,065 s | 0,250 s |
| Inventaire 32, observation processus réelle | 1,422 s, incomplète | 6,210 s |
| Inventaire 128, observation processus réelle | 2,191 s, incomplète | 6,452 s |
| RSS maximale du groupe, 128 | 21 632 Kio | 62 996 Kio |

Les durées globales ne comparent pas un service identique : l'ancien lecteur n'avait pas accès à tous les processus. Le gain de 70 % porte sur les appels Git de la fixture128, pas sur le temps global. La mémoire est mesurée, pas plafonnée par le noyau.

Dépôt réel : 49 worktrees secondaires, 50 inscriptions. Avant : 4,570 s et 81 248 Kio. Après : 17,608 s et 49 128 Kio. L'observation processus de ce passage après est restée inconnue. Une lecture ultérieure de ces 49 portées a réussi complètement. Les registres et processus peuvent évoluer entre mesures ; aucun gain de temps global n'est revendiqué.

## Limites
Les commandes partagent 30 s, avec jusqu'à 1 s de grâce TERM. Les appels filesystem Python et le rendu restent hors délai dur. Captures lsof/Git : 8 Mio/1 Mio ; registre : 4 096 entrées. Les événements transitoires du système peuvent produire un inconnu et conserver les worktrees. Aucun retry automatique d'une mission métier n'est ajouté. Une observation n'est pas un verrou : la course avant retrait reste possible, comme avant.
