# Plan 045 — Refuser les options inconnues dans les messages

## Décision

Modifier uniquement les boucles de parsing de `cmd_send` et `cmd_reply` :

1. reconnaître `--` et copier le reste des arguments dans le corps ;
2. refuser tout autre jeton commençant par un tiret dans le bras inconnu ;
3. faire passer `--to` et `--from` par le helper existant `option_value` afin
   de nommer leur valeur manquante ;
4. conserver les mots ordinaires et toutes les options déjà reconnues.

Le helper d'erreur existant est réutilisé. Aucune nouvelle abstraction ni
dépendance n'est nécessaire.

## Pourquoi ce périmètre

- le défaut se trouve dans deux bras par défaut du même fichier ;
- la valeur manquante de `--to` et `--from` est la même faute de parsing ;
- la cohérence entre options reconnues appartient à un autre lot suspendu ;
- `--no-reply` n'existe pas dans la surface CLI et ne doit pas être créé ici.

## Complexité

Le parcours reste O(n), où n est le nombre d'arguments. Le séparateur permet
de copier chaque argument restant une seule fois. L'espace reste O(n) pour le
corps, comme sur la base.

## Vérification

1. Lister puis exécuter le harnais d'intégration sur la base gelée.
2. Ajouter des témoins du vrai binaire et constater leur rouge avant le code.
3. Implémenter la garde et constater le vert du harnais complet.
4. Retirer la garde dans les deux parseurs : les témoins `send` et `reply`
   doivent mourir en montrant un message sérialisé.
5. Neutraliser le séparateur : les témoins littéraux doivent mourir.
6. Restaurer par patch inverse, vérifier les condensats, le formatage, le diff
   et la compilation de toutes les cibles.

## Portes constitutionnelles

- **Périmètre** : deux fonctions de production, un harnais et la documentation
  de session.
- **Minimalisme** : aucun helper nouveau tant que deux branches simples
  suffisent ; potentiel de suppression estimé à 0 ligne à comportement
  constant.
- **Responsabilité future** : le contrat `--` est explicite et couvert par un
  oracle d'effet, ce qui réduit l'ambiguïté du parseur.
- **Dépendances** : aucune nouvelle dépendance.
