# 276 : application inconnue sans résultat métier

Statut : correction en qualification.

Une lecture du guichet et une admission native appartiennent à deux registres.
L’absence ou l’expiration d’une clé du premier n’atteste pas l’absence d’une
tâche du second. Même une mutation dont la réponse est coupée peut avoir produit
son effet. applied:false exige une preuve métier ; sans cette preuve, applied
vaut null. La portée du résultat et l’absence de consultation native sont
explicites. Les booléens attestés par les payloads terminaux restent conservés.

La correction est une projection MCP. Les codes wire et les clés de registre ne
changent pas. Aucun rapprochement entre registres ne contourne le contrôle de
propriétaire natif. Les anciennes façades déjà chargées conservent leur format
jusqu’à rechargement ; aucune activation fictive ni arrêt T3 implicite.
