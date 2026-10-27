# Session 184 — Confirmation du démarrage distant

Statut : en cours. Correction autorisée d'un incident récurrent KS-STOR.

## Besoin
Conserver et confirmer la même commande quand le lancement distant réussit mais que son accusé dépasse le délai réseau. Ne pas recréer de mission métier.

## Faits
Deux admissions réelles ont renvoyé remote_release:WorkerUnavailable alors que le wrapper et Claude tournaient dans leur worktree exact. Pour f0b0a469, observation des PID 2865011 et 2866251 ; aucune mutation de la tâche par le responsable du correctif. Le parent conserve le pilotage.
Le maître attend cinq secondes pour Release, malgré une revalidation distante et une file partagée. Après expiration de l'attente, un accusé Released d'une commande connue est refusé ; le lecteur worker ferme son canal sur ce refus.

## Exigences
- Attendre une confirmation lente dans une limite bornée, sans prolonger les droits du permis.
- Accepter sans mutation un accusé tardif pour la même identité worker, époque et commande.
- Refuser une époque périmée ou une autre instance.
- Conserver la tâche, le choix de routage et les protections contre les doublons.
