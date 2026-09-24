# ADR 042 — Appariement des réponses par origine prouvée, et entretien périodique du daemon

- **Date** : 2026-09-24
- **Statut** : Accepté
- **Session** : 116

## Contexte

Le pont T3 relaie à l'expéditeur la réponse du tour qu'a déclenché sa demande. T3 ne lie pas un
message utilisateur au tour qu'il lance : l'appariement se faisait par rang, le k-ième message
utilisateur répondant au k-ième tour assistant de la fenêtre lue, et seulement si les deux décomptes
étaient égaux. Sur les données réelles, cette égalité est l'exception :

- un tour qui n'écrit rien est invisible au décompte, qui ne voit les tours qu'à travers leurs
  messages. Notre propre enveloppe d'un message sans réponse attendue demande à l'agent de ne rien
  accuser, et fabrique donc ces tours muets ;
- un agent Claude est réveillé en arrière-plan quand un sous-agent termine, et ouvre des tours sans
  message utilisateur (un fil relevé : 58 messages pour 139 tours écrits) ;
- plusieurs messages tapés pendant un tour partagent ce tour.

Seize demandes suivies ont ainsi été abandonnées depuis le 14/09, destinataire ayant pourtant répondu.
Les points de contrôle de T3, envisagés comme décompte fiable, ont été écartés sur mesure : absents
hors dépôt Git, et sans correspondance un pour un avec les messages sur la plupart des fils.

Par ailleurs, l'état du daemon s'accumulait : la purge des sagas d'envoi existait sans être appelée,
le ramassage des temporaires ne passait qu'au démarrage, et aucun mécanisme ne retirait l'état
d'identité des processus disparus ni ne bornait les journaux.

## Décision

1. **Preuve directe d'abord.** Le pont relevait déjà, pour le journal d'observation, l'origine du
   dernier tour : le message utilisateur unique dont l'horodatage égale strictement l'horodatage de
   demande du tour, T3 projetant les deux de la même commande. Cette table est désormais alimentée à
   chaque lecture et consultée en premier : la réponse est le texte de CE tour, et seulement de lui.
   Le calcul par rang reste en repli, inchangé, quand l'origine n'a pas été relevée.
2. **Un tour prouvé muet est annoncé**, après cinq lectures concordantes pour absorber le décalage
   de projection de T3, par un texte explicitement attribué à Bridget.
3. **Entretien horaire du daemon**, premier passage deux minutes après le démarrage : remises
   expirées encore « en vol » passées en sort inconnu ; envois expirés depuis plus de 30 jours purgés
   (les lancements d'équipiers, liés en cascade au même enregistrement, sont épargnés) ; état
   d'identité des processus disparus retiré ; temporaires et configurations MCP orphelins ramassés ;
   journaux de service au-delà de 20 Mio tournés, une génération conservée.
4. **Worktrees fusionnés** retirés par le script de construction, sous les conditions de la
   constitution (XVI.3).

## Conséquences

**Positives** : les réponses aux demandes suivies ne dépendent plus d'un décompte fragile ; un
destinataire muet ne laisse plus son expéditeur attendre jusqu'à l'échéance ; l'état persistant et
les journaux sont bornés sans intervention.

**Négatives** : une demande dont le tour s'est terminé avant la lecture suivante du pont n'a pas
d'origine relevée et retombe sur le calcul par rang. Une demande muette est annoncée avec une
quinzaine de secondes de délai. Un rejeu idempotent plus de 37 jours après l'envoi serait réexpédié
au lieu d'être refusé comme expiré. Quelques lignes de journal peuvent se perdre à la rotation.
