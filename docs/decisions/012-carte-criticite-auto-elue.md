# ADR 012 — Carte de criticité auto-élue et décision de régime traçable

**Statut** : Accepté
**Date** : 2026-08-25

## Contexte

La doctrine de revue compare déjà le diff d’un lot à une liste de zones
critiques, mais cette liste et le régime final étaient choisis par le référent.
Douze lots ont ainsi reçu le jury sans qu’aucun fait durable ne permette de
voir ou mesurer cette décision. Un prototype fondé sur des noms nus a ensuite
classé à tort des homonymes `main.rs` et `store.rs`.

Le registre du dû fournit des preuves utiles mais réactives. À froid, une carte
vide empêche précisément les revues qui produiraient les premiers constats.
Enfin, une fuite récente a montré qu’une écriture durable de données externes
est une porte critique même lorsqu’elle n’est ni migration, ni protocole, ni
permission.

## Décision

le service compagnon tient une carte par projet, sans liste de chemins configurable. Elle
combine quatre règles de germe, la détection dans le diff, les constats
Bloquants/2 Majors et les ancrages contractuels exacts. Toute zone est un
chemin complet relatif au dépôt ; une citation ambiguë n’élit jamais plusieurs
homonymes.

le service compagnon propose un régime. Le référent doit retenir explicitement une valeur
fermée et peut durcir ou alléger sans fournir de justification textuelle.
Chaque décision, écart et refus est durable et comptable. Le noyau qui calcule
la carte reste soumis au régime `jury_2x2` fixé par la présente décision ; il
ne peut ni le calculer ni le remplacer lui-même.

L’élection des relecteurs est séparée jusqu’à disponibilité d’un capteur
fiable. L’absence de cette dépendance produit un état explicite, jamais une
attente silencieuse ni un mandat fondé sur une présence périmée.

## Conséquences positives

- aucune personne ne choisit silencieusement les fichiers critiques ;
- les homonymes ne fabriquent plus de faux classement ;
- le démarrage à froid possède un germe universel ;
- les écarts et refus deviennent des mesures vérifiables ;
- les journaux ne recopient pas les contenus externes qui ont déclenché la
  décision.

## Conséquences négatives

- la règle entrée externe → écriture durable peut provoquer des jurys
  conservateurs ;
- les citations historiques ambiguës restent non élues tant qu’un fait ne les
  désambiguïse pas ;
- le référent reste une étape obligatoire pour retenir le régime ;
- les sorties de carte ne peuvent pas encore être approuvées puis retirées ;
- l’élection des relecteurs exige un lot ultérieur et un capteur absorbé.

## Alternatives rejetées

- **liste éditable** : elle rendrait la décision à la personne que le système
  doit mesurer ;
- **élection de tous les homonymes** : elle reproduit le faux positif observé ;
- **motif obligatoire** : une valeur gratuite satisfait la forme sans
  confronter les faits ;
- **décision automatique sans sélection du référent** : elle retire la décision
  sans rendre la responsabilité mesurable ;
- **consommer immédiatement `bridget-idle`** : son état périmé peut mandater un
  relecteur qui ne rendra jamais de verdict.
