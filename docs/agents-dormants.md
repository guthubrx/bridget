# Agents dormants

Un agent dormant reste dans l'historique. Bridget conserve son identité, ses
rôles, ses données et les travaux déjà engagés. Le sommeil n'arrête pas un
programme et ne termine pas une mission.

Il empêche les **nouvelles sollicitations Bridget** : messages directs,
notifications de groupe, occurrences programmées et nouvelles sélections pour
un rôle. Les autres membres d'un groupe continuent de recevoir ses avis.
Le message reste lisible dans le fil. Aucun avis écarté pendant le sommeil
n'est envoyé automatiquement au réveil.

« Réveiller » rend l'agent disponible sans lancer un tour. Un message, une
reconnexion ou un résultat ne le réveille pas. Les résultats de travaux déjà
admis restent collectés. Une réponse à une demande directe déjà acceptée est
conservée dans le journal et dans son reçu de réponse, sans nouveau tour chez
le destinataire dormant. Les questions humaines restent accessibles.

## Terminal

```sh
bridget agents --table --global
bridget agents --dormant --table --global
bridget agents --all --json --global
bridget agents sleep <UUID>
bridget agents wake <UUID>
```

Les deux dernières commandes modifient le suivi. Elles rendent un reçu avec
`request_id`. Conserver ce reçu si la réponse est perdue. Reprendre la même
action avec `--request-id <UUID-du-reçu>`, le même agent et les mêmes options.
Un choix différent exige une nouvelle action. `--revision <n>` protège contre
un changement effectué sur un autre appareil.

## Application

Choisir « Actifs », « Agents dormants » ou « Tous ». Ouvrir une fiche, puis son menu :
« Mettre en sommeil » ou « Réveiller ». Les favoris respectent aussi ce filtre.
L'application conserve l'enveloppe d'une action non confirmée et la rapproche
avant d'envoyer un autre choix.

## Clients liés

La synchronisation fonctionne dans les deux sens par défaut. Un réglage sur la
fiche permet « Bridget uniquement ». Au terminal, ajouter `--local-only` pour
ce mode ou `--synchronize` pour réactiver la synchronisation.

Le choix natif est conservé si le client n'est pas disponible. La fiche indique
alors une synchronisation en attente. Le connecteur reprend la même opération.
Seules les fiches déjà liées sont concernées. Les contrats des adaptateurs
précisent la traduction et les effets propres à chaque client.

Le sommeil est une règle de sollicitation dans Bridget. Il ne désactive pas
un programme tiers, ses outils indépendants ou la saisie humaine dans ce
programme. Les identités et les états des travaux restent séparés du sommeil.
