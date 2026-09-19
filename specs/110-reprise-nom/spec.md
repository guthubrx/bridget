# Spécification 110 — Reprendre un nom d'affichage laissé par une identité éteinte

## Fiche synthèse

Spec: 110-reprise-nom
Statut: Implemented (livré 2026-09-19)
Priorité: P1
Tâches: 6/6
Date: 2026-09-19
Branche: session-110-reprise-nom
Dépendances : extension de nom 089, annuaire vivant, adaptateur T3 098.

## Problème observé

Constaté le 2026-09-19 sur le poste : trois fils T3 voient leur renommage refusé en boucle,
toutes les 330 secondes, avec le motif `NameConflict`. Exemple : le fil affiché `claude-horizon`
dans T3 est connu de Bridget sous son titre automatique, parce que le nom `claude-horizon` est
détenu depuis le 14 septembre par un profil qui n'apparaît plus dans l'annuaire.

Cause : le nom d'affichage est unique et durable, et rien ne le libère quand son détenteur
disparaît. Chaque réimport de conversation crée une identité neuve qui ne peut pas reprendre son
ancien nom. Mesure du jour : 268 profils enregistrés pour 16 agents connectés, dont 253 dormants
depuis plus de deux jours.

Conséquence : l'utilisateur et les agents voient deux noms différents pour le même fil, et une
demande adressée au nom affiché dans T3 ne trouve pas son destinataire.

## Scénarios utilisateur

- **US1 (P1) — Récupérer son nom.** Un fil renommé dans T3 demande ce nom à Bridget. Le détenteur
  actuel est absent de l'annuaire vivant : le nom est transféré, le demandeur le porte.
- **US2 (P1) — Ne pas voler un nom vivant.** Le détenteur est connecté : la demande est refusée
  comme aujourd'hui, avec le même motif.
- **US3 (P2) — L'ancien détenteur reste identifiable.** Le profil dépossédé reçoit un nom dérivé,
  jamais un vide. Son identifiant et son historique ne changent pas.

## Exigences fonctionnelles

- **FR-001** : une demande de nom déjà détenu est acceptée si et seulement si le détenteur est
  absent de l'annuaire vivant au moment de la demande.
- **FR-002** : si le détenteur est vivant, le refus reste `NameConflict`, inchangé.
- **FR-003** : lors d'un transfert, l'ancien détenteur reçoit un nom dérivé disponible, construit
  à partir de son nom courant ; il conserve son identifiant, son avatar et ses instructions.
- **FR-004** : le transfert est atomique : soit les deux profils changent, soit aucun.
- **FR-005** : la révision de chaque profil touché est incrémentée, pour que les lecteurs voient
  le changement.
- **FR-006** : la résolution d'un nom vers un identifiant continue de rendre le détenteur courant.
- **FR-007** : aucune identité n'est créée, supprimée ni fusionnée ; seul le libellé bouge.

## Critères de succès

- **SC-001** : sur une base de test, un nom détenu par un profil absent de l'annuaire est transféré
  au demandeur, et l'ancien détenteur porte un nom dérivé distinct.
- **SC-002** : le même appel, avec un détenteur présent dans l'annuaire, est refusé `NameConflict`.
- **SC-003** : recette complète verte, sans régression de l'extension de nom 089.
- **SC-004** : après livraison, les trois fils en conflit portent leur nom T3 dans l'annuaire, et le
  journal du pont ne montre plus de refus en boucle.

## Hors périmètre

Purge des profils dormants, fusion d'identités, libération automatique par ancienneté sans
consultation de l'annuaire, changement du format de sérialisation.
