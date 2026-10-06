# Recherche 135 — Contrôle silencieux des missions

## Décision 1 — Contrôler les états, pas les tours de conversation

**Décision** : déclencher sur une obligation nouvelle ou une escalade.

**Raison** : une tranche d'âge ne constitue pas une information nouvelle. Les
guides Anthropic recommandent une architecture simple, des critères d'arrêt et
des preuves observables. La recherche Microsoft relie les interruptions
fréquentes à plus de stress et à une productivité perçue plus faible.

**Sources** :
- https://www.anthropic.com/engineering/building-effective-agents
- https://www.microsoft.com/en-us/research/publication/the-cost-of-email-use-in-the-workplace-lower-productivity-and-higher-stress/

## Décision 2 — Utiliser une machine d'escalade durable

**Décision** : conserver, pour chaque anomalie, son étape et la dernière preuve
observée. Une progression remet l'escalade à zéro.

**Raison** : la signature globale actuelle change avec l'âge. Un état par
anomalie permet la déduplication, la reprise après redémarrage et une escalade
bornée.

**Alternative rejetée** : augmenter le délai du LaunchAgent. Cela réduit la
fréquence, mais ne corrige ni les répétitions ni les agents endormis.

## Décision 3 — Enregistrer le progrès par une commande stricte

**Décision** : ajouter une commande `progress` qui exige un type fermé et une
preuve courte. La prise en charge reste une preuve initiale distincte.

**Raison** : un texte libre tel que « je vais faire » ne prouve rien. Une liste
fermée permet des tests et des diagnostics sans lire le raisonnement de l'agent.

## Décision 4 — Activer le contrat à une date précise

**Décision** : enregistrer `mission_control_enabled_at` dans le run. Les tâches
antérieures restent intactes. Elles peuvent être résumées, mais elles ne sont
pas déclarées acceptées.

**Raison** : la boucle Politique contient un historique réel. Une migration ne
doit ni inventer de décision ni transformer un verdict.
