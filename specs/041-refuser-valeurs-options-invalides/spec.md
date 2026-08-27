# Session 041 — Refuser les valeurs d'options invalides

**Statut** : Implémentée

**Base gelée** : `90802b0377741b509f3743c5675544315b6f0f29`

**Objectif** : `54e6399e-b262-439a-bf34-3ccc110608d5`

## Défaut mesuré

Les commandes `send` et `reply` acceptent silencieusement une valeur invalide
pour `--timeout` ou `--hops`. Une conversion échouée devient respectivement
l'absence de délai ou la valeur par défaut `4`. Une option sans valeur est
également traitée comme absente. Les valeurs nulles et les nombres de sauts
négatifs traversent la CLI.

La sonde de référence exécute le vrai binaire contre une socket Unix jetable et
capture le message sérialisé : `--timeout abc` avec `--reply` envoie un délai de
60 secondes, tandis que `--hops nope` envoie quatre sauts. Les deux commandes
terminent avec un code nul.

## Propriété

Une valeur présente de `--timeout` ou `--hops` doit être un entier strictement
positif représentable par son type. Toute valeur invalide est refusée avec le
code 2 avant toute connexion au daemon. Le diagnostic nomme l'option et la
valeur reçue. Une option sans valeur est refusée en nommant l'option.

L'absence complète de l'option conserve son défaut documenté. Aucune borne
supérieure arbitraire n'est ajoutée ; seule la borne du type constitue une
limite. Les valeurs valides existantes restent inchangées.

## Critères d'acceptation

1. `send` et `reply` refusent `abc`, zéro et l'absence de valeur pour les deux
   options ; `--hops -1` est également refusé.
2. Le refus précède la connexion au daemon et toute création de PID, socket ou
   base dans un environnement jetable.
3. Les valeurs de contrôle `--timeout 9` et `--hops 2` sont conservées exactement.
4. Un mutant supprimant la validation actuelle du délai fait échouer les deux
   oracles `send` et `reply` du délai. Le mutant symétrique des sauts fait
   échouer les deux oracles correspondants.
5. Les appelants réels mesurés avec 60, 9 et 1 restent compatibles.
6. Sans dernier expéditeur enregistré, `reply` refuse encore une valeur
   invalide avec le code 2 et nomme l'option avant de lire cet état. Un mutant
   remontant la lecture d'état avant l'analyse des options tue cet oracle.

## Hors périmètre

La divergence entre la CLI et le MCP sur l'usage de `--timeout` sans demande de
réponse est une faute sémantique distincte. Cette session ne décide pas quelle
surface a raison et ne modifie pas ce comportement.
