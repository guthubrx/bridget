# État runtime avant correction — SPEC-165

Date : 2026-10-14  
Pont : PID 66786, démarré le 2026-10-14 à 19:23:19 CEST  
Serveur T3 : PID 30234, `http://127.0.0.1:3773`

## Inventaire T3 observé

| Mesure | Valeur |
|---|---:|
| Fils totaux | 1 530 |
| Fils vivants selon l'ancien filtre | 1 130 |
| Sous-agents vivants selon l'ancien filtre | 1 100 |
| Sous-agents terminés | 834 |
| Sous-agents au repos sans activité courante | 266 |
| Activités courantes observées | 2 |
| Liens annoncés par le pont à 19:42 CEST | 1 121 |

Les 834 sous-agents terminés restent dans l'historique T3. Ils ne doivent pas rester connectés à Bridget.

## Ressources du PID 66786 à 19:47 CEST

| Type | Nombre |
|---|---:|
| Total `lsof` | 4 495 |
| Sockets Unix | 3 363 |
| Fichiers réguliers | 1 129 |
| IPv4 | 1 |

La forme dominante est trois sockets Unix par lien. Un verrou d'instance et parfois un journal ajoutent des fichiers réguliers.

## Limite temporaire

Le LaunchAgent actif fixe `maxfiles (soft)` à 16 384. Cette valeur a restauré le service après les erreurs `Too many open files`. Elle ne constitue pas le correctif. La recette finale doit retirer cet override et prouver le fonctionnement avec la limite normale de 4 096 appliquée par Bridget.

## Cause vérifiée

La boucle ouvre un lien complet pour chaque fil T3 non archivé. Elle ne distingue pas une conversation principale d'un sous-agent historique terminé. Le nombre de descripteurs croît donc avec l'historique des délégations.

## Preuve du contrat V2 réel

Lecture authentifiée et expurgée de `GET /api/orchestration/shell` le 14 octobre 2026 à 20:02 CEST :

- `schemaVersion` vaut `2` ;
- le chemin consommable est `threads[].lineage.relationshipToParent` ;
- 1 537 fils étaient présents : 406 racines, 10 forks et 1 121 sous-agents ;
- un échantillon de sous-agent portait la valeur exacte `subagent` et un `parentThreadId` ;
- aucun jeton, texte, titre ou identifiant complet n'est conservé dans cette preuve.

Le comptage des états compacts par identifiant dérivé a ensuite observé :

| Mesure de reprise | Valeur |
|---|---:|
| Sous-agents terminaux ou inactifs examinés | 1 121 |
| États compacts présents | 1 093 |
| `pending` non vide | 0 |
| Dernier tour connu non projeté | 0 |
| Obligations à rouvrir | 0 |

Le plafond prévu de 32 drains simultanés reste nécessaire comme garde contre un état futur ou ancien différent.
