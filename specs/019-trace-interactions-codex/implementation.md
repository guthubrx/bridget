# Implémentation — Spec 019

## Décision

Une trame Codex serveur→client qui porte simultanément `method` et `id` devient
un fait `provider_request`. Le payload persistant est reconstruit par liste
blanche :

```json
{
  "provider": "codex",
  "method": "item/commandExecution/requestApproval",
  "request_id": "sha256:<64 hexadécimaux>",
  "turn_id": "sha256:<64 hexadécimaux>",
  "state": "pending"
}
```

Le `message_id` Bridget reste dans l'enveloppe commune du journal. Aucun
`params`, prompt, corps de commande, chemin, résultat ou chaîne fournisseur
libre n'est recopié. La méthode d'approbation connue est sélectionnée par
égalité stricte puis réémise depuis une constante locale. Une méthode inconnue,
un identifiant JSON-RPC valide et un identifiant de tour deviennent des
empreintes SHA-256 séparées par domaine. Leur sortie a toujours 71 caractères :
`sha256:` puis 64 hexadécimaux. Les identifiants JSON-RPC invalides sont projetés
depuis une constante de domaine sans que leur contenu participe à l'empreinte.

Une erreur JSON-RPC fournisseur conserve seulement son code entier borné et
une empreinte de l'objet d'erreur. Les raisons locales fermées — échéance, EOF,
arrêt — restent lisibles. Aucune projection ne tronque une entrée : ESC, retour
chariot ou contrôle bidirectionnel ne peut donc survivre comme préfixe.

La dernière requête est conservée en mémoire sous un verrou propre ; l'écriture
du fait est mise en file avant que le worker puisse consigner une échéance ou
un EOF. Le fait `error` reprend alors exactement cette projection sous
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

Après la charge de confidentialité du jury, quatre sondes ont d'abord rougi
uniquement parce que des chaînes libres traversaient des champs autorisés :
méthode, identifiant JSON-RPC chaîne, identifiant de tour et raison fournisseur.
Leurs contrôles positifs attestaient la trame lue, l'erreur `-32098` reçue, les
deux événements présents et une corrélation non vide et stable. Après la
projection opaque, les quatre sondes passent sans supprimer aucun événement.

L'auto-revue hostile a aussi resserré l'ordre concurrent : le lecteur prend le
verrou de corrélation avant de lire le message actif. Une requête ainsi
linéarisée est donc mise en file dans le journal avant que le worker puisse
émettre son erreur terminale.

## Limites assumées

La session ne répond pas aux requêtes serveur, ne fixe pas
`approvalPolicy`, ne modifie pas le sandbox et n'attribue aucune panne
historique. Une future session corrective devra partir d'une trace réelle
produite par ce mécanisme.
