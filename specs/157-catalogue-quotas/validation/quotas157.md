# Quotas et contrat MCP 157

T013 : six tests métier échouent avant raccord, compilés et exécutés (aucun tick). Après T014/T015, six réussissent. Le refus garde zéro tâche et la clé libre ; resets_at vient du fait observé. Les trois sources passent par le vrai handle_rate_limit et sa clé agent_type. La mémoire survit au retrait de la présence et ignore les faits passés.

T014 : les deux tests historiques du handler passent dans T016-T020-red157-T014-historique. Le bilan de cette commande est 2PASS/3FAIL ; les trois échecs sont les nouveaux oracles MCP, avant leur raccord.

T016/T020 : les trois oracles MCP passent ensuite avec les cinq tests MCP existants. Le bilan de la commande mixte est 12PASS/10FAIL : 8MCP PASS, et 14fallback = 4PASS/10FAIL avant implémentation. T020 reste à clôturer après contrôle des clés/features du catalogue.

La préparation d'API worktree pour RED transmet une vraie Probe, sans sélection fallback ; aucun échec de compilation compté comme preuve. Les quatre fallback déjà verts vérifient les comportements conservés (modèle/effort/type, droits du parent, cible valide, sans liste). Les dix rouges portent sur la sélection, le résultat durable, l'épuisement et la validation nouvelle. Le chemin entre passes n'est pas atteint au RED, car la première sélection échoue ; il devra être exercé au GREEN.

Les reçus JSON/log adjacents portent commandes, dates, SHA, résultats et sources stables. Aucun fournisseur réel exécuté ; aucune validation globale déduite.
