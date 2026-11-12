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

Qualification : 31 tests ciblés, Clippy strict workspace/all-targets et release réussis sur Mac et Debian 12. Activation encore en cours.
