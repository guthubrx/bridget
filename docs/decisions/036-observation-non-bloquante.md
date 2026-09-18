# ADR 036 — Observer sans orchestrer

Date : 2026-09-16. Périmètre : session 100, communication Bridget.

## Décision

Réutiliser le journal pour lire/partager un extrait borné via attach et send.
Réutiliser le writer du journal pour publier de petites métadonnées après
écriture confirmée, dans une file distincte de celle d'attach. Le flux attach
n'est pas drainé sans vue ; le détourner ferait perdre son contenu au lecteur.

Abonnements et cache d'écritures en mémoire daemon. Les propriétaires viennent
des preuves d'identité existantes. Quatre événements fermés, filtres simples,
notifications ordinaires sans reply, aucun script arbitraire et aucun nouveau
processus de coordination. Limites : 16/agent,128 total,TTL1h/max7j ; files256/64,
cinq notifications/s/abonnement, tentative1s/expiration5s ; cache4096/30s.

Les identifiants internes de notifications portent `bridget-observation:`.
Le journal ignore les observations issues de ces tours corrélés, pour empêcher
les boucles entre abonnés. Une fin de tour non corrélée (idle natif/T3 seul)
n'est pas un substitut fiable. Cette couverture limitée est explicite, pas
une prétention d'observation universelle. Les notifications ne clôturent jamais
les demandes de réponse et n'imposent aucune action aux destinataires.

## Compromis

Abonnements perdus au redémarrage ; once consommé au déclenchement même sans
remise ; mode ne-pas-déranger respecté ; compteurs de pertes/suppression visibles,
pertes source dans les logs. Pas de rattrapage historique. Collisions seulement
sur signaux d'écritures confirmées, même hôte/chemin lexical ; ni shell, ni
symlink/casse canonisés, ni analyse Git. Un avertissement n'empêche rien.

Pas de migration DB, dépendance externe, verrou de fichier, formulaire, mandat,
moteur de règles général ni couche le service compagnon/T3 obligatoire.

## Vérification

Tests purs filtres/TTL/propriétaire/collision/bornes ; vrai writer post-flush ;
producteurs positifs et négatifs ; sockets CLI/MCP et dispatcher daemon ;
communication/reply témoin sous pression de notifications. Tests dans des
namespaces privés, sans fournisseur externe. Après autorisation de livraison,
un vrai daemon isolé a validé les parcours CLI/MCP, puis l'installation existante
a été mise à jour avec sauvegarde ; aucune conversation fournisseur arrêtée.
