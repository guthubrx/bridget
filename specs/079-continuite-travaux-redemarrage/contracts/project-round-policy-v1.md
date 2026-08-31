# Contrat project-round-policy-v1

## Opérations

- `list`: projette toutes les liaisons projet et leur état effectif;
- `status`: projette un projet;
- `enable`: active la génération exacte;
- `disable`: désactive la génération exacte.

## Requête de mutation

- version 1;
- command_id stable;
- project_id non vide;
- binding_generation strictement positive;
- état souhaité fermé.

## Refus

- projet absent;
- liaison inactive;
- génération divergente;
- commande divergente;
- enveloppe ou version invalide;
- store indisponible.

## Projection

- project_id;
- binding_generation courante;
- enabled effectif;
- configured indique si une décision existe pour cette génération;
- revision;
- updated_at.

## Scheduler

Le scheduler ne reçoit comme cibles que les projections `active`, `configured`
et `enabled`. Il ne rattrape pas les occurrences antérieures.
## Dispatch interne

Le dispatcher calcule `floor(now / 420) * 420`, liste les politiques actives
et demande une émission distincte pour chaque `ProjectReference`. Le daemon
revérifie la politique et la génération avant admission.

La remise porte:

- `origin=routine`;
- `intent=trigger_turn`;
- la référence structurée `project_id + binding_generation`;
- une clé stable dérivée de projet, génération et occurrence.

Une occurrence antérieure à la fenêtre courante est refusée. Une politique
réactivée ne rejoue donc aucun tick manqué. Le destinataire reste le
coordinateur Bridget commun; aucun provider ni runtime n'est choisi par cette
politique.

