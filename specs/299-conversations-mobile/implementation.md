# Vérifications et livraison 299

18 novembre 2026 : 45 tests release Android, zéro échec, lintRelease et
assembleRelease réussis sur Cartae. Les quatre nouveaux tests couvrent favoris,
récents, recherche, dédoublonnage, limite de 50 contacts et filiation explicite.
Aucune nouvelle dépendance ; les routes serveur existantes sont réutilisées.

Preuves visuelles incomplètes : device_list ne trouve aucun appareil Android ;
l’émulateur est absent du SDK exposé. Aucun diaporama reconstitué ni installation
sur téléphone n’est présenté comme une preuve. Les maquettes restent des propositions.

Le login Claude Cartae est de nouveau détecté configuré après la reconnexion
humaine ; aucun essai fournisseur métier n’est déduit de ce contrôle.

Le ménage initial ne trouve aucun worktree secondaire ni branche fusionnée à
retirer. Le cache Rust du checkout reste actif et nécessaire à la publication.

Revue indépendante native 8bb1d4a5-38fe-437a-a07f-3903d1edfc71 : un défaut
confirmé de reprise après resync_required. Correction : invalider le curseur et
les pages de cette liste, proposer Actualiser dans tous les états et expliquer
la reprise depuis la première page. La boîte est créée une seule fois au niveau
MobileApp, pas une fois par annuaire/volet. Le tri est mémorisé par ses entrées.
Les favoris complets préexistants restent compatibles ; aucun changement de format.
La suite JVM couvre le modèle pur, pas les appels HTTP ni les interactions Android.
Le scénario réseau de pagination invalidée reste non exécuté faute d’appareil.
