# Journal d'implémentation 059

## Métadonnées

- Branche : `session-059-mesure-verification-production`
- Base gelée : `394c0f5c8d352a606c85db7232bb1ad4fa79446e`
- Objectif : `924ccb3c-2ac1-49e7-85a9-746d32d36a11`
- Délégation : `f3ca9807-bdf2-4be9-88bb-bbeef803a86e`
- Message : `78cba682-d9e5-434c-b4fb-c3078606118e`

## Tranche 0 — réservation

La branche vide a été publiée au même objet que la base gelée. Aucun fichier
de la session 058 n'est dans le périmètre.

## Tranche 1 — contrat

Le contrat sépare objectifs et racines, publie leur couverture et interdit un
ratio ponctuel sur une population partiellement classée. Aucune cible n'est
encodée.

## Tranche 2 — oracle rouge réel

Commande :

```text
scripts/test-bridget-rapport-vp.sh
```

Le harnais exécute la ronde contre la base Maicie réelle, avec la borne gelée
de la population humaine. La ronde termine sa lecture et produit un JSON ; le
témoin meurt ensuite à l'assertion finale de projection, pas au montage :

```text
AssertionError: la ronde réelle ne publie pas encore la mesure V/P
```

Code de sortie : 1. Aucun payload du corpus n'est imprimé par le harnais.

## Tranche 3 — mesure et contre-épreuve

La ronde lit désormais tous les objectifs dans sa copie SQLite, valide leurs
identifiants et horodatages, puis sélectionne séparément la baseline gelée et
la fenêtre glissante. Toute ligne non annotée reste `indeterminate`.

Sur la baseline réelle :

```text
population=169 id_bytes=6253
sha256=e2a279624ecd4fbd9ca7f8f97effa45777c15b6b3b9574f3d43a9d90298d5a55
classification=5/169 root=5/169
objectifs: production=1 verification=4 indeterminate=164
racines: production=1 verification=1
expansion verification=4.0
```

Les deux ratios restent `unavailable` avec le motif
`classification_coverage_incomplete`. `target` vaut `null` et `origin_used`
vaut `false`.

### Mutant de granularité

Mutation unique dans l'agrégateur :

```text
roots[classification].add(root_id)
                              |
                              +-- remplacé par row["id"]
```

Le montage et la lecture des 169 objectifs réussissent. Le témoin meurt à
l'assertion finale de racine avec la valeur interdite réellement produite :

```text
AssertionError: le découpage a changé la racine vérifiée: 4
```

Code de sortie : 1. Après restauration, le même harnais rend :

```text
rapport V/P réel : baseline, couverture et invariance de racine vérifiées
```

Le SHA-256 de `scripts/bridget-ronde.py` vaut avant et après mutant :
`62fedf91a7a5d17c55600a830a27ae9fce43a1eb89d73d06d6a2b73fd2a81a2b`.

Le harnais historique de ronde reste vert et exerce en plus la dégradation :
un ancien schéma sans `payload_json` rend la mesure `unavailable`, tandis que
les agents, demandes et objectifs à évaluer restent publiés.
