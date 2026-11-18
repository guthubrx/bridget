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

## Publication vérifiée

Version 0.8.53, code Android 8053, tag sur d5068f0a28b854b6572b21a7cce27fd272f745f5.
Publication officielle : https://github.com/guthubrx/bridget/releases/tag/v0.8.53
Six actifs téléchargés anonymement et comparés aux tailles et SHA-256 locaux.
L’API releases/latest annonce v0.8.53, stable et publique, avec l’APK attendu.
APK : 4 012 604 octets ; SHA-256
7b122559c2202f70314be47c01292cea942f69678158a6cc377066ab2c4dab06.
Certificat de production inchangé, signature V3/API28 vérifiée contre 0.8.52.
Les 45 tests release, lintRelease et assembleRelease passent après correction.
Les sept tests du garde de publication passent. Build Mac réussi en 1 min 29 s,
Linux Debian 12 en 2 min 41 s ; glibc requise 2.34, plafond 2.36 respecté.
Aucune activation serveur ou installation sur téléphone déduite de la publication.

Ménage : aucun worktree secondaire ni branche superflue dans le dépôt local ou
sur origin. Gradle clean a retiré les sorties Android (180 412 Kio alloués avant
nettoyage). La copie de compilation SYS créée pour ce lot (129 448 Kio) et la
copie binaire intermédiaire locale (45 857 976 octets) ont été retirées après fin
des outils et vérification des usages. Ces tailles ne sont pas un gain DF mesuré.
Les maquettes utiles sont déplacées hors dépôt ; les APK, manifestes et logs de
livraison restent dans le cache privé bridget299-mobile. Le nettoyeur Rust garde
7,04 Gio de caches reconnus, sous son budget, ainsi que les exécutables utilisés.
La revue native est acquittée et sa garde close ; sa vue reste kept/review_required
suivant son reçu, sans retrait manuel. Aucun arrêt de T3 ni service n’a eu lieu.

Limite restante : le téléphone doit installer la mise à jour par son parcours
Android. Réglages > Vérifier les mises à jour force la lecture sans attendre le
prochain contrôle périodique. Le parcours visuel et la conservation de
l’association sur cet appareil restent à vérifier après installation réelle.
