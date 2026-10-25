# Archives techniques de l’adaptateur T3

Document d’origine : [specs/110-reprise-nom/spec.md](../../../../../../specs/110-reprise-nom/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Spec: 110-reprise-nom
Statut: Implemented (livré 2026-09-19)
Priorité: P1
Tâches: 6/6
Date: 2026-09-19
Branche: session-110-reprise-nom
Dépendances : extension de nom 089, annuaire vivant, adaptateur T3 098.

<a id="extrait-2"></a>

## Extrait 2

Constaté le 2026-09-19 sur le poste : trois fils T3 voient leur renommage refusé en boucle,
toutes les 330 secondes, avec le motif `NameConflict`. Exemple : le fil affiché `claude-horizon`
dans T3 est connu de Bridget sous son titre automatique, parce que le nom `claude-horizon` est
détenu depuis le 14 septembre par un profil qui n'apparaît plus dans l'annuaire.

<a id="extrait-3"></a>

## Extrait 3

Conséquence : l'utilisateur et les agents voient deux noms différents pour le même fil, et une
demande adressée au nom affiché dans T3 ne trouve pas son destinataire.

<a id="extrait-4"></a>

## Extrait 4

- **US1 (P1) — Récupérer son nom.** Un fil renommé dans T3 demande ce nom à Bridget. Le détenteur
  actuel est absent de l'annuaire vivant : le nom est transféré, le demandeur le porte.
- **US2 (P1) — Ne pas voler un nom vivant.** Le détenteur est connecté : la demande est refusée
  comme aujourd'hui, avec le même motif.
- **US3 (P2) — L'ancien détenteur reste identifiable.** Le profil dépossédé reçoit un nom dérivé,
  jamais un vide. Son identifiant et son historique ne changent pas.

<a id="extrait-5"></a>

## Extrait 5

- **SC-001** : sur une base de test, un nom détenu par un profil absent de l'annuaire est transféré
  au demandeur, et l'ancien détenteur porte un nom dérivé distinct.
- **SC-002** : le même appel, avec un détenteur présent dans l'annuaire, est refusé `NameConflict`.
- **SC-003** : recette complète verte, sans régression de l'extension de nom 089.
- **SC-004** : après livraison, les trois fils en conflit portent leur nom T3 dans l'annuaire, et le
  journal du pont ne montre plus de refus en boucle.
