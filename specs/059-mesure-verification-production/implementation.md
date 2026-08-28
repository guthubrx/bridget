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

