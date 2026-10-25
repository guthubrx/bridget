# ADR 042 — Appariement des réponses par origine prouvée, et entretien périodique du daemon

- **Date** : 2026-09-24
- **Statut** : Accepté
- **Session** : 116

## Contexte

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/042-appariement-par-origine-prouvee-et-entretien.md#extrait-1).

- un tour qui n'écrit rien est invisible au décompte, qui ne voit les tours qu'à travers leurs
  messages. Notre propre enveloppe d'un message sans réponse attendue demande à l'agent de ne rien
  accuser, et fabrique donc ces tours muets ;
- un agent Claude est réveillé en arrière-plan quand un sous-agent termine, et ouvre des tours sans
  message utilisateur (un fil relevé : 58 messages pour 139 tours écrits) ;
- plusieurs messages tapés pendant un tour partagent ce tour.

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/042-appariement-par-origine-prouvee-et-entretien.md#extrait-2).

Par ailleurs, l'état du daemon s'accumulait : la purge des sagas d'envoi existait sans être appelée,
le ramassage des temporaires ne passait qu'au démarrage, et aucun mécanisme ne retirait l'état
d'identité des processus disparus ni ne bornait les journaux.

## Décision

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/042-appariement-par-origine-prouvee-et-entretien.md#extrait-3).

## Conséquences

**Positives** : les réponses aux demandes suivies ne dépendent plus d'un décompte fragile ; un
destinataire muet ne laisse plus son expéditeur attendre jusqu'à l'échéance ; l'état persistant et
les journaux sont bornés sans intervention.

**Négatives** : une demande dont le tour s'est terminé avant la lecture suivante du pont n'a pas
d'origine relevée et retombe sur le calcul par rang. Une demande muette est annoncée avec une
quinzaine de secondes de délai. Un rejeu idempotent plus de 37 jours après l'envoi serait réexpédié
au lieu d'être refusé comme expiré. Quelques lignes de journal peuvent se perdre à la rotation.
