# ADR-016 - Notifications UI fondées sur les preuves agent

**Statut** : Accepté

## Contexte

L'interface doit signaler à l'humain le travail réel d'un agent et l'arrivée
d'une réponse sans présenter l'acceptation interne de Bridget comme une action
de l'agent.

## Décision

1. L'indicateur de travail est dérivé exclusivement des événements fournisseur
   déjà journalisés.
2. La réponse attendue est suivie comme attente métier, distincte de l'échéance
   d'exécution du fournisseur.
3. Les notifications utilisent l'API navigateur native, activée par un geste
   explicite. Elles ne couvrent que la page encore ouverte.
4. Le clic de notification cible le message par son identifiant ; la cible
   interne de nouveaux messages est le repli si le clic natif n'est pas livré.

## Conséquences

- Aucun backend de notification, service worker, abonnement Push ou dépendance
  n'est ajouté.
- Le navigateur peut refuser les notifications et le clic est meilleur effort.
- Le journal existant devient la seule source de vérité UI pour l'activité.
- La navigation doit attendre le rendu du fil avant de faire défiler vers une
  cible.
