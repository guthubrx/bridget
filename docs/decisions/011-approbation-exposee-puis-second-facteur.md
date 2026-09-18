# ADR 011 — Approbation exposée à l'interface, second facteur différé

**Date** : 2026-08-24
**Statut** : Accepté
**Portée** : approbation d'activation de profil, interface locale et distante

## Contexte

L'approbation d'activation d'un profil exige aujourd'hui un terminal
interactif : le service compagnon vérifie que son entrée ET sa sortie sont un terminal,
affiche la définition à autoriser, puis attend que l'humain tape « oui ».
Aucune option ne permet de contourner ce contrôle
(`plugins/guichet/src/main.rs`, `confirm_local_profile_approval`).

La garantie tient par **impossibilité** et non par confiance : un agent qui
tourne sur cette machine peut exécuter des commandes, écrire des fichiers et
émettre des requêtes réseau, mais il ne peut pas appuyer sur les touches d'un
clavier relié à un terminal ouvert.

L'utilisateur ne veut pas passer par la ligne de commande pour approuver, et
accepte l'exposition. Décision prise en connaissance du risque, après que
celui-ci ait été énoncé.

## Décision

**En premier temps : l'approbation est exposée à l'interface, sans second
facteur.** L'écran présenté reste identique — profil, type d'agent, modèle,
effort, commande, arguments, variables interdites — et une action explicite
de l'utilisateur reste requise. Ce qui disparaît est le contrôle de terminal,
donc la preuve structurelle qu'un humain agit.

**En second temps : un code à usage unique par canal séparé.** L'interface
présente la proposition ; la confirmation exige un code arrivé par un autre
chemin — le canal Telegram déjà en place est le candidat. Un agent local ne
reçoit pas les messages personnels de l'utilisateur : la preuve structurelle
est donc rétablie, et elle fonctionne AUSSI à distance, mieux que le terminal
puisqu'elle n'exige pas d'être devant la machine.

## Risque accepté, et sa borne exacte

Ce que l'approbation autorise n'est PAS l'exécution arbitraire : elle valide
une proposition **déjà persistée**, dont l'empreinte de définition est
comparée dans la transaction d'écriture. Les définitions vivent dans un
registre en 0600 que l'interface ne modifie pas.

Le risque réel est donc borné à ceci : **un agent pourrait faire naître un
agent parmi ceux déjà déclarés.** C'est de l'auto-réplication limitée au
catalogue existant, pas l'ouverture d'un interpréteur de commandes. Sérieux,
mais pas illimité.

Aggravant à surveiller : plus le registre contient de types d'agents, plus la
surface grandit. Ajouter un type au registre devient donc, tant que le second
facteur n'existe pas, une décision de sécurité et non de confort.

## Conséquences

**Positives.** L'interface est utilisable immédiatement, sans détour par le
terminal. Le distant devient possible en écriture, pas seulement en lecture.
Le temps gagné va au chemin critique.

**Négatives.** Pendant la fenêtre entre les deux temps, rien ne distingue
structurellement l'utilisateur d'un agent local devant l'approbation. La
garde ne s'affaiblit pas : elle est absente.

**Condition de sortie, écrite ici pour ne pas dériver.** Le second facteur
doit être livré avant que le distant soit ouvert hors du réseau local, et
avant tout ajout de type d'agent au registre motivé par le confort. Si l'un
des deux arrive avant, cette ADR est en dette et doit être rouverte.

## Alternatives écartées

**Garder le terminal seul** : refusé par l'utilisateur, et le motif est
recevable — approuver depuis une interface est le geste naturel, et l'exiger
en ligne de commande décourage l'usage de l'approbation elle-même, ce qui est
un risque en soi.

**Livrer le second facteur d'abord** : coûterait le temps du chemin critique
pour une garantie dont l'absence est temporaire et bornée.

**Ne rien écrire et exposer** : refusé par le référent. Une garde retirée
sans trace se lit six mois plus tard comme un oubli, et personne ne sait
qu'il y avait une raison. C'est exactement ce qui a coûté trois faux
diagnostics dans la nuit du 23 au 24 août.
