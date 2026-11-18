# Questions mobiles compactes

Signalement humain : 74f6e19a-95e7-4f2d-805f-823ae5503a17, 18 novembre 2026.

La capture fournie montre deux cartes fixes sous la conversation, dont une réponse
déjà enregistrée. Le filtre incluait submitted et conservait jusqu’à deux cartes.
Le correctif remplace ces cartes par une ligne ouvrant À traiter. Seules les
questions encore répondables et les issues inconnues sont signalées. Les réponses
envoyées et les demandes indisponibles restent consultables dans une section
repliée. Les notifications ouvrent directement la question dans À traiter.

Aucune réponse n’est déduite d’un message libre. Aucun état serveur n’est modifié.
Les brouillons et clés des réponses conservées restent inchangés.

Vérification Cartae : testDebugUnitTest, lintDebug, assembleRelease réussis en 63s.
41 tests JVM, zéro échec, dont deux tests de régression couvrant le classement des
états et la conservation des issues inconnues. Avertissements existants Kotlin et
Gradle sans lien avec ces modifications.

Preuves incomplètes : avant fourni par l’utilisateur ; device_list ne trouve aucun
appareil Android et indique un émulateur absent du SDK actif. Aucun après visuel
ni installation sur le téléphone revendiqué. APK de contrôle non signé construit
sous /home/moi/.cache/bridget-mobile298-build ; il n’est pas un paquet publié.
La publication signée et l’activation restent à terminer par le parcours mobile.
