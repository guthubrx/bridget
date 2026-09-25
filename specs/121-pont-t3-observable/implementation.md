# Journal 121 - Pont T3 observable, sobre, et réponses incertaines transmises

- **Base** : main `f0a543c1` - **Date** : 2026-09-25 - **Statut** : Implemented

## Décisions
- Tour spontané : preuve par couverture (un message de la page est antérieur à la demande du tour)
  et absence de message utilisateur à son horodatage. Origine enregistrée `t3-turn:<id>` : jamais un
  identifiant de message, donc jamais appariée à une demande. Nos notifications sont toujours des
  messages, un tour spontané ne peut pas en provenir : pas de boucle d'observation.
- Réponse incertaine : bornée au message utilisateur suivant (une notification intercalée coupe
  aussi : préférer moins que le texte d'autrui) ; aucune si rien n'a été écrit.
- Journal : les deux derniers avertissements répétés du chemin de projection passent par le
  signalement unique et différé de la 117.

## Vérifications
Voir tasks.md T004-T005.
