# Corrections 262 - refus exploitables et portables

- [x] Reproduire la protection des chemins hérités et conserver sa cause fermée, négociée sur le protocole worker.
- [x] Conserver le dépôt source détecté lorsque HEAD est introuvable et guider sans supposer une absence d’histoire.
- [x] Corriger l’expiration de réponse : référence complète, portée locale, issue à vérifier avant toute suite.
- [x] Exécuter les régressions ciblées et la revue indépendante.
- [ ] Pousser, compiler depuis la base active avec les seuls correctifs autorisés, activer et vérifier les cibles.
- [ ] Informer les deux demandeurs et nettoyer les intermédiaires inutilisés.

Tous les scénarios sont génériques : dossiers temporaires, aucune machine ni racine personnelle codée en dur. Aucune admission métier n’est rejouée.
