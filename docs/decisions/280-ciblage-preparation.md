# 280 : isoler les échecs des autres liaisons de préparation

Statut : qualification en cours.

## Contexte

La préparation inspecte les liaisons installées pour retrouver la machine et
l'autorité demandées. Un échec SSH de n'importe quelle liaison annulait cette
recherche, y compris après un succès attesté de la cible. La régression reproduit
les deux ordres possibles. Cela ne prouve pas la cause du code255 historique E6.

## Décision

Différer seulement project_preparation_process_failed et
project_preparation_ssh_inspect_timeout pendant la recherche. Une seule cible
attestée permet de poursuivre sur ce lien. Sans cible attestée, restituer la
première erreur intacte. Plusieurs cibles restent un refus d'ambiguïté. Les
refus de politique restent immédiats. Le mapping, l'identité, la révision et
l'empreinte sont contrôlés comme avant. Aucune autre machine ni connexion de
modèle n'est sélectionnée, aucun paramètre ni droit n'est élargi.

## Conséquences

Le cas de réussite reste O(n), n étant le nombre de liaisons. Le chemin d'erreur
peut maintenant parcourir n liaisons au lieu d'échouer à la première. Ce coût
permet de retrouver la cible tout en conservant le contrôle d'unicité. Les
probes restent séquentielles et les délais existants sont inchangés : connexion
SSH10s, commande300s, budget natif de préparation borné par son appelant. Ne pas
présenter ces délais comme une garantie de réussite en10s. Plusieurs liaisons
indisponibles peuvent retarder le résultat ; aucune attente globale des autres
travaux n'est prescrite. Aucun stockage durable ni protocole nouveau.
