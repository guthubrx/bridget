# 279 : rendre les délais et les diagnostics exploitables

Statut : qualification en cours.

## Contexte

Une demande en file a expiré après le délai implicite de 60 secondes avant sa
remise au fil. Le reçu n'annonçait ni ce délai ni le fait que la file le consomme.
Un refus SSH antérieur ne pouvait être relié avec certitude à son diagnostic :
le journal source n'avait pas d'heure et le reçu ne portait pas la corrélation.

## Décision

Conserver la sémantique actuelle. Pour une demande suivie, le reçu MCP précise
le délai effectif et l'inclusion de l'attente en file. Le message durable conserve
exactement son option reply_timeout, y compris la différence None/Some(60).
Les avis de timeout et le refus d'une demande non injectée expliquent la portée,
la lecture du ledger et les conditions d'une décision du propriétaire.

Horodater le diagnostic source. Quand le relais utilise un worker source distant,
le maître journalise le même UUID, les machines source/cible et l'heure du retour.
Ce journal est limité au code fermé project_preparation_process_failed. Les noms
de machines sont ceux du parcours attesté, bornés et sans caractères de contrôle.
Aucun stderr, contenu de mission, jeton ni empreinte d'authentification n'est ajouté.
Un ancien refus sans données conservées reste sans cause reconstituée.

## Conséquences

Aucun changement du protocole daemon/worker, des délais, intentions, droits,
états ou reprises métier. Les champs du reçu MCP sont additifs. Une façade déjà
montée garde son ancien code jusqu'à son rechargement : T3 reste ouvert selon
la décision de l'utilisateur. Les messages du maître et du connecteur bénéficient
du chargement des services Bridget. Le coût ajouté est O(1), hors sérialisation
bornée des quelques champs du diagnostic.
