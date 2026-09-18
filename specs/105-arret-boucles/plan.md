# Plan

1. Reproduire l’enveloppe inconditionnelle et la création d’attentes sur `reply=false` dans les tests T3 existants.
2. Respecter `reply` au dispatch et au relais. Persister la preuve pour les nouvelles demandes ; pour les anciennes, utiliser la réconciliation existante avec les demandes du daemon, sans inventer une preuve en cas d’absence dans la liste bornée.
3. Aligner les consignes terminal, Codex, ACP et Claude. Conserver la séparation entre réponse finale relayée en mode géré et réponse via outil en mode interactif.
4. Tester messages sans demande, réponses corrélées, demandes explicites, notifications, redémarrage, annulation et absence de preuve. Exécuter les tests pertinents existants.
5. Documenter le comportement et les preuves de validation. Ne pas relancer les services de production dans cette phase.

Complexité : garde constant par remise/réponse ; réconciliation des anciennes attentes indexée par identifiant et expéditeur, sans recherche quadratique supplémentaire. Aucune dépendance nouvelle.
