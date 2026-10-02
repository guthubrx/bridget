# Spécification 131 - Une réponse relayée se signale dans le fil de l'utilisateur

## Fiche synthèse
Spec: 131-relais-visible | Statut: Implemented | Priorité: P3 | Date: 2026-10-02
Branche: session-131-relais-visible | Retour d'expérience opus2D, point 5.

## Problème observé (données réelles)
01/10 05:13:26Z, fil du coordinateur `opus_city_ai` : une demande avec réponse attendue d'un autre
agent ; la réponse finale du coordinateur, relayée à cet agent, est restée affichée dans le fil de
l'utilisateur sans repère et en tutoyant l'autre agent. À 05:32:24Z l'utilisateur demande :
« C'est à moi que tu parles ou c'est à la jambe bridget ». Ce n'était pas un raisonnement interne :
le raisonnement est déjà rangé à part par T3.

## Exigence
- **FR-001** : l'enveloppe d'une demande avec réponse attendue demande d'ouvrir la réponse par
  « ↪ Réponse à <expéditeur> (relayée par Bridget) : ».
