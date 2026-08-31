# Contrat restart-continuity-v1

## Admission

Une remise idempotente dont l'intention est `trigger_turn` ou
`interrupt_and_start` est exécutable seulement si elle possède:

- une soumission contenant l'enveloppe exacte;
- une exécution non terminale;
- un lien durable delivery_id vers submission_id et execution_id.

## Livraison

`DeliverIdempotent.execution` est optionnel pour compatibilité. S'il est
présent, le wrapper crée le binding avant l'injection. Le delivery_id reste
l'unique identité de suppression des doublons.

## Reprise

- reconnexion avec tour vivant: aucune reconstruction;
- remise encore dispatching: rejeu de la même remise;
- exécution active sans remise en vol: continuation reconstructed;
- exécution terminale: aucune action;
- message absent ou corrompu: défaut visible, aucun prompt;
- plusieurs actifs concurrents: refus de reconstruction aveugle.

## Invariants

- un parent terminal ne rouvre jamais;
- un descendant conserve soumission et projet;
- une reprise est elle-même une remise idempotente;
- la politique de ronde n'est jamais consultée dans ce contrat.
