# Ventilation du temps de sonde

Le 12 novembre 2026, les quatre candidats KS d'une sonde ont pris
50 817 ms sur un total de 50 819 ms. Le refus après 45 secondes ne prouve
ni une mémoire épuisée ni une file pleine. Le coût interne restait inconnu.

Le lot ajoute `candidate_phases_ms` à la ligne existante
`bridget_worker_probe172`. Il ne change aucun verdict, délai ou droit.
Chaque élément conserve le `candidate_id` déjà présent dans le journal :

- `stage` : dernière phase atteinte, `mapping`, `git`, `readiness` ou `verdict` ;
- `mapping_ms`, `git_ms`, `readiness_ms` : durées monotones des phases terminées ;
  `null` signifie non terminée ou non atteinte, `0` signifie moins de 1 ms ;
- `git` : `not_reached` avant Git, `uncached` sans cache de lot, `miss` à la
  première lecture de cette clé, `hit` à sa réutilisation dans le même lot.

Les candidats refusés avant l'appel de contrôle sont absents des deux listes
de durées. La dernière phase atteinte permet de lire un refus précoce sans
présenter une phase non exécutée comme instantanée. Aucun chemin, contenu
de configuration ou sortie fournisseur n'est ajouté au journal.

La somme des durées n'inclut pas tous les contrôles de fin du lot. Un `hit`
Git ne réutilise ni définition ni autorité du candidat. Les captures et
revalidations de permissions restent inchangées. Le hachage du lanceur est
une hypothèse de coût, pas encore une cause mesurée sur KS.

Revue indépendante : tâche 2ed70cd0-bd67-4e93-918d-c7fff75e4ccd.
Les observations sur les phases absentes, la double recherche dans le cache
et le test limité à la sérialisation sont traitées. Le test parcourt maintenant
le vrai contrôle : refus Git `miss`, même refus `hit`, puis mapping absent.
Une seule recherche `BTreeMap::entry` produit l'étiquette et la valeur.

Qualification : 31 tests ciblés, Clippy strict workspace/all-targets et release réussis sur Mac et Debian 12. Activation vérifiée le 12 novembre 2026 avec la version 0.8.48.

## Premières observations du runtime actif

Deux sondes naturelles de KS, après activation, portent la même admission
`a-b8180a56941625f291f5f127a946ec17`, avec un candidat chacune :

| Corrélation | Total candidat | Mapping | Git | Readiness |
|---|---:|---:|---:|---:|
| 355eac89-779f-44ab-b349-beff6ff74ff8 | 13 567 ms | 0 ms | 78 ms | 13 488 ms |
| cd01c91a-0ba9-448a-8080-64590ca96070 | 12 774 ms | 0 ms | 66 ms | 12 708 ms |

Le coût est localisé dans le contrôle de préparation et de permissions
(`readiness`), à plus de 99 % du temps candidat. Ce résultat ne distingue pas
encore ses opérations internes et ne prouve pas le coût du hachage. Aucune
mission métier n'a été créée pour obtenir ces mesures. Les deux cas ne
mesurent pas une sonde complète à quatre candidats ni son taux d'échec.

Preuve locale : /Users/moi/.cache/bridget283-284/ks-phases-after.json.
