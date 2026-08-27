# Plan 046 — Distinguer connexion et activité

## Pourquoi corriger le champ existant

`last_seen_secs` expose déjà l'âge de la dernière capacité observée et non la
durée de connexion. Un message émis est une preuve de cette même capacité ; le
défaut est donc un écrivain manquant, pas une grandeur nouvelle. Ajouter un
second champ conserverait un nom trompeur sans préserver d'âge de connexion,
car les tours et le runtime écrasent déjà l'horloge actuelle.

## Algorithme

1. Après une préparation idempotente durable réussie, résoudre la présence de
   `message.from` et appeler son rafraîchissement de capacité.
2. Après une écriture réussie du message historique dans le ledger, appliquer
   la même opération.
3. Si le nom ne correspond à aucune présence enregistrée, ne rien créer et ne
   rien rafraîchir.
4. Ne toucher ni à `link_seen`, ni au heartbeat, ni à la réception du message
   par la cible : une connexion MCP auxiliaire ne prouve pas le socket wrapper.

## Oracles et mutant

Deux témoins utilisent les chemins productifs complets avec une présence dont
`capacity_seen` a été vieillie à 1 900 secondes. Ils exigent une réponse
nominale et un âge public inférieur à deux secondes. Le contrôle négatif envoie
vers une cible inconnue et exige un refus avec un âge toujours supérieur à
1 800 secondes.

Après le correctif, le mutant retire les deux appels de rafraîchissement, sans
modifier la préparation ni le ledger. Les deux témoins nominaux doivent mourir
à l'assertion d'âge ; le contrôle de refus doit rester vert.

Un second mutant remplace le rafraîchissement d'activité par le
rafraîchissement général de capacité, qui touche aussi `link_seen`. Les deux
témoins nominaux doivent alors mourir à l'assertion sur l'âge du socket, tandis
que le contrôle de refus reste vert. Il protège la séparation entre activité
métier et santé du lien principal.

## Portée de validation

La production modifiée appartient à `bridget-daemon`, dont aucun paquet ne
dépend. La closure ciblée est donc le paquet daemon. Base et tête sont compilées
et comptées dans des targets physiquement distincts ; le workspace complet est
joué une seule fois sur la candidature finale.
