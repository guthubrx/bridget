# Plan220 — proposition avant implémentation

1. Protocole additif ProviderCatalogueV1, observation liée à worker+époque+connexion+empreinte de définition. Champs fermés : état auth sans secret, empreinte opaque de compte, modèles/efforts, statut/cache/fraîcheur, dates et cause fermée.
2. Sonde locale bornée sur le programme Codex configuré ; contexte d’environnement exact. Initialiser avec l’identité Bridget ; account/read puis model/list paginé puis account/read. Aucun thread ou tour. Arrêt TERM confirmé, aucun SIGKILL ; un échec d’arrêt conserve le propriétaire du processus et interdit une nouvelle sonde concurrente.
3. Travail séparé de la file des huit commandes. Un créneau par worker conservé pendant les reconnexions. Déclenchement initial, changement non secret et renouvellement borné ; aucune sonde de modèle à chaque lecture capabilities.
4. Cache volatile du maître lié à la connexion worker, invalidé par époque/configuration/expiration. Projection publique additive model_catalogues ; catalogue déclaré clairement distinct. Les trois ports de catalogue utilisent le même contrat.
5. Réconcilier le catalogue d’admission avec ces observations sans changer les reçus admis ni la politique de modèle/compte. Une observation fournisseur est un fait, pas une autorité de modèle supplémentaire. Définir précisément le comportement si une découverte est indisponible.
6. Vérifications ciblées, revue, distribution/activation et preuve réelle hors mission métier.

Choix à trancher par l’analyse de conception : signal non secret fiable pour connexion externe ; portée du cache fournisseur partagé ; interaction avec un compte API sans identifiant public ; refus précis d’un modèle hors politique ; visibilité additive pour MCP214.
