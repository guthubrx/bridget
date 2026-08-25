# Implémentation — Spec 019

## Décision

Une trame Codex serveur→client qui porte simultanément `method` et `id` devient
un fait `provider_request`. Le payload persistant est reconstruit par liste
blanche :

```json
{
  "provider": "codex",
  "method": "item/commandExecution/requestApproval",
  "request_id": "approval-native",
  "turn_id": "turn-native",
  "state": "pending"
}
```

Le `message_id` Bridget reste dans l'enveloppe commune du journal. Aucun
`params`, prompt, corps de commande, chemin ou résultat n'est recopié. La
dernière requête est conservée en mémoire sous un verrou propre ; l'écriture du
fait est mise en file avant que le worker puisse consigner une échéance ou un
EOF. Le fait `error` reprend alors exactement cette projection sous
`pending_provider_request`.

## Preuve rouge puis verte

Le faux app-server a d'abord été ajouté seul. Le premier essai a été rejeté :
son contrôle inspectait le fichier des trames reçues par le faux serveur au
lieu de sa sortie. Après correction de l'oracle, la trame bloquante et sa
sentinelle étaient bien observées en mémoire, le tour atteignait l'échéance, et
le test échouait uniquement sur l'absence du fait durable.

Après instrumentation :

- l'échéance conserve la requête avant l'erreur et expurge la sentinelle ;
- la fermeture de stdout conserve la même corrélation ;
- `attach` rend la requête puis la corrélation terminale en français.
- un identifiant JSON-RPC d'un type invalide est remplacé sans recopier son
  contenu ;
- une saturation du journal produit un échec observable qui entraîne l'arrêt
  de la session au lieu de perdre silencieusement la preuve.

L'auto-revue hostile a aussi resserré l'ordre concurrent : le lecteur prend le
verrou de corrélation avant de lire le message actif. Une requête ainsi
linéarisée est donc mise en file dans le journal avant que le worker puisse
émettre son erreur terminale.

## Limites assumées

La session ne répond pas aux requêtes serveur, ne fixe pas
`approvalPolicy`, ne modifie pas le sandbox et n'attribue aucune panne
historique. Une future session corrective devra partir d'une trace réelle
produite par ce mécanisme.
