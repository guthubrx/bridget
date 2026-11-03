# 230 — Sommeil natif des agents

Date : 3 novembre 2026. Décision utilisateur acceptée.

## Contexte

L'annuaire conserve des agents arrêtés. Un simple filtre d'affichage laisserait
ces agents sollicitables. Les présences temporaires et les rôles durables ne
constituent pas un état de suivi humain. Le client lié possède aussi ses
propres effets lors du rangement d'un fil.

## Décision

Stocker le sommeil par identité native dans le magasin principal, avec
révision, origine et reçu idempotent. Le contrôler aux frontières d'envoi et
de sélection. Conserver les missions déjà admises et leurs résultats.

Le réglage de synchronisation est bidirectionnel par défaut, selon le choix
explicite de François. Il peut être désactivé par fiche. Une opération en
attente ne bloque pas l'action native. L'adaptateur confirme le même identifiant
de commande et ne transforme pas une sollicitation en réveil explicite.

Un ancien lecteur qui ignore le sommeil ne peut pas servir de retour logiciel.
Le marqueur de données 8 et le format `agents.dormancy.v1` imposent un lecteur
compatible. La livraison doit préparer un retour comprenant cette fonction.

## Conséquences

Le sommeil est indépendant de la présence, de l'exécution et de la conservation
des données. Les vues Android et leurs caches suivent le classement serveur.
Les outils d'un autre programme qui contournent Bridget restent régis par ce
programme ; l'adaptateur ne leur invente pas une interdiction.
