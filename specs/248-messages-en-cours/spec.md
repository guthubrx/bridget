#248 — Comptes rendus pendant un tour Codex

Signalement Parix3758 : messages545efd04 et1abfc9bc vers749 restent en vol ; parent dans le même tourCodex depuis15:03:30UTC. Premierreplyfalse encoredispatching ; secondreplytrue600s désormaistimed_out et retiré avant démarrage. Ne pas rejouer ces clés, ni lancer une mesure ou mission métier. T3 ne doit pas être arrêté.

Autoriser la coordination d’un enfant natif actif vers son parent sur une autre machine avec les mêmes preuves de relation/instances/révocations. L’enfant ne reçoit pas l’autorité de steer ou interrupt de son parent. Discovery conserve son plafond.

Remettre les comptes rendus sans réponse attendue pendant un tourCodex quand le daemon le permet et que le clientT3 négocieV2. Réutiliser le steer natif déjà pris en charge, sans nouvelle session ni interruption. Les demandesreplytrue et notifications non groupables gardent leur corrélation et attendent leur propre tour. Les expirations restent des expirations, pas des annulations volontaires.

R2 : la reconnexion du maître a recalculé le délai de tour du message d’avis de maintenance déjà en file. L’ancien spool comparait tous les octets et fermait le wrapper sur conflit. Comparer l’enveloppe complète sauf date de tour et libellés de présentation ; conserver la première trame sans changer son échéance ou son autorité. Tout vrai conflit de remise reste refusé sans ACK et sans arrêter le fournisseur. Aucun retour automatique de la mission Parix échouée.
