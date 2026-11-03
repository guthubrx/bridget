# Sommeil des agents liés à T3

L'adaptateur traduit les gestes de rangement et de restauration vers le
sommeil natif. Le réglage est bidirectionnel par défaut. « Bridget uniquement »
désactive ces échanges pour la fiche. Les identités déjà liées sont conservées
après fermeture de leur présence. Aucun import de tout l'historique T3.

## Effets

| Geste ou événement | Résultat |
|---|---|
| Mettre en sommeil dans Bridget | Refus immédiat des nouvelles sollicitations Bridget ; demande de classement du fil lié |
| Réveiller dans Bridget | Retour dans les actifs ; demande de restauration du fil lié, sans prompt ni nouveau tour |
| Rangement observé dans T3 | Sommeil de la fiche native déjà liée |
| Restauration explicite observée dans T3 | Réveil de la fiche native |
| Message, pulsation ou résultat | Aucun réveil natif implicite |
| Client non observé ou réponse perdue | Choix natif conservé, synchronisation en attente |

T3 refuse son rangement pendant un travail actif, une file manuelle ou une
question bloquante. Bridget ne les arrête ni ne les supprime. Le connecteur
attend un état permettant le rangement. Les effets propres au rangement T3
(terminaux inactifs, scripts et politiques de nettoyage) restent ceux de T3.

Un envoi par un outil indépendant de T3 peut remettre son fil en activité et
lancer du travail. Le sommeil Bridget ne désactive pas cet outil. Cet événement
n'est pas traduit en réveil implicite. Le dernier geste de classement confirmé
et l'état physique d'un travail restent deux faits distincts.

## Reprise et ordre des choix

Le maître conserve la file de classement, ses révisions et ses reçus.
Une commande client incertaine garde son identifiant. Le choix suivant du même
agent attend sa résolution. Les autres fiches continuent d'avancer.

Le client mémorise aussi certains refus. Un refus ne devient certain que sur
un reçu natif typé et corrélé à la commande exacte. Une erreur de transport ou
une erreur quelconque ne suffit pas. Après refus certain, une observation
nouvelle et un état admissible permettent au plus deux nouveaux essais de
classement. L'identifiant de l'action Bridget reste conservé ; les identifiants
des tentatives client sont distincts et leurs reçus restent enregistrés.
Ce mécanisme ne rejoue aucune mission.

Après trois départs refusés, le choix natif reste actif. L'application indique
le refus et permet une nouvelle confirmation humaine après résolution côté
client. Une commande non remise et remplacée par un choix plus récent n'est
pas envoyée. Une commande déjà incertaine n'est pas effacée.

Si la synchronisation est désactivée pendant une remise incertaine, seule cette
commande déjà partie est rapprochée avec le même identifiant. Le nouveau choix
reste propre à Bridget. Aucun nouvel ordre de classement ne part après ce
rapprochement. La fiche indique cette attente avant d'afficher « Bridget uniquement ».

Les séquences observées et les reçus de commit empêchent un ancien instantané
d'annuler un choix plus récent. Les refus non reconnus restent explicitement
inconnus. Aucun numéro de version d'un fournisseur de modèle n'intervient dans
ce contrat.
