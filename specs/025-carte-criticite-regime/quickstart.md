# Quickstart 025 — Scénario de validation

## 1. Soumettre le lot

Préparer une requête canonique `review_lot_submit` portant l’identifiant de
projet configuré, une référence complète, la base et la tête. Relever le
guichet avec une commande Maicie normale.

Résultat attendu : une proposition unique `awaiting_decision`, accompagnée
des chemins déclencheurs complets et du nombre de citations non résolues.

## 2. Retenir le régime

Le référent envoie `review_regime_select` avec l’identifiant de soumission et
une valeur fermée. Il n’envoie aucun motif.

Résultat attendu : décision durable, direction `same`, `strengthened` ou
`lightened`, et état explicite indiquant que l’élection des relecteurs n’est
pas encore disponible.

## 3. Contrôler la carte et les chiffres

```text
maicie review list --config <chemin-absolu>
maicie review metrics --config <chemin-absolu>
```

La première sortie montre les chemins complets et leurs identifiants de
preuve. La seconde compte décisions, écarts, refus et ambiguïtés. Aucune ne
rend le patch ou le texte complet des constats.

## 4. Contre-épreuves obligatoires

1. ajouter un secret synthétique uniquement dans le diff : il déclenche sa
   règle éventuelle mais n’apparaît ni dans la base ni dans les sorties ;
2. citer `store.rs` avec deux homonymes : zéro zone élue et une ambiguïté ;
3. citer `store.rs:7000` quand un seul candidat atteint cette ligne : un seul
   chemin complet élu ;
4. déplacer la référence après avoir gelé la tête : refus durable ;
5. supprimer une règle de germe : son scénario devient simple et l’oracle
   rougit ;
6. présenter une base marquée v19 sans son DDL : v20 refuse sans modifier un
   octet.

## Limite visible

Ce quickstart ne mandate aucun relecteur. Une tentative d’avancer au-delà de
la décision reçoit `reviewer_election_unavailable` tant que le lot capteur
n’est pas absorbé.
