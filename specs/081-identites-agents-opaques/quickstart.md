# Vérification opérateur - SPEC-081

1. Empêcher les nouveaux lancements et vérifier que les sauvegardes des deux bases peuvent être créées.
2. Exécuter bridget identity migrate --dry-run.
3. Vérifier les compteurs : identités, messages, exécutions et références Maicie converties ou mises en requires_retarget.
4. Exécuter bridget identity migrate --apply.
5. Relancer le daemon puis les agents auparavant actifs.
6. Renommer un agent dans Bridget Desktop et vérifier listes, conversations, étiquettes et notifications.
7. Envoyer un message entre deux agents et vérifier absence d'UUID et de nom historique dans le prompt.
8. Vérifier une délégation Maicie convertie et une référence explicitement à retargeter.

Résultat attendu : aucun client actif ne peut utiliser name ; les liens sont soit convertis avec le même agent_id, soit visiblement suspendus en attente de retarget.
