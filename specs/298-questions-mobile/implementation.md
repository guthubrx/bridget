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
Les contrôles finaux testReleaseUnitTest et lintRelease réussissent aussi (23s),
avec 41 tests release, zéro échec. Le build final conserve la configuration Firebase.

Revue indépendante 8877e43f-b7aa-4a02-887d-8182324b4f1e lue et acquittée.
Le cas local (réponse sauvegardée mais non envoyée) reste à vérifier dans le
bandeau et la liste active. Le filtre submitted reste replié : cet état atteste
l’enregistrement serveur, même si une commande locale avait une issue inconnue.
Le callback de visibilité devenu inutile est retiré. Le focus de notification fait
défiler la liste jusqu’à la question avant d’ouvrir le formulaire. La proposition
de traiter les formulaires malformés est hors du correctif : le serveur valide
déjà 1 à 16 questions et les champs requis ; aucune malformation observée ici.
Les comportements de récupération de notification générique et de focus persistant
préexistants restent hors lot, sans suppression de brouillon ni réponse.

Un candidat privé a été aligné, signé avec la lignée existante et contrôlé par
publish-android.py sans publication. Certificat de production inchangé, V3/API28.
SHA256 : 586457c5861a86997ee290d953ab681057fab0b0c2d5f5e6ed36df93a49e89cb.
Taille : 3 971 644 octets. Candidat conservé :
/Users/moi/.cache/bridget-mobile298/bridget-android-298-candidate.apk.

Publication et activation incomplètes : le mainteneur garde la coordination du
numéro commun et a confirmé que la version 0.8.52 est déjà publique. Aucun actif
0.8.52 n’a été remplacé. Le lot doit intégrer la prochaine publication ; le candidat
8052 ne sera pas proposé comme mise à jour automatique de 8052. Aucun appareil
Android accessible pour installer ou vérifier la conservation de l’association.
Le responsable de publication reçoit le commit et les contrôles ; l’activation et
les preuves visuelles restent ouvertes. Aucun redémarrage de T3 ni de service.

## Réservation de publication commune, 18 novembre 2026

Version suivante réservée : **0.8.53**, code Android **8053**. Le contrôle du
distant ne trouve ni tag v0.8.53 ni publication ultérieure à v0.8.52. Cette
réservation dans le suivi ne constitue ni un tag, ni une publication, ni une
activation. Le numéro des sources reste 0.8.52 jusqu'à la préparation du lot.

Le mainteneur Bridget porte le numéro commun, les builds serveur, la signature,
la publication et la coordination d'activation. Le responsable mobile298 garde
la propriété des fichiers Android et de leur vérification visuelle. Base intégrée
du mobile : 50cfb57b, comprenant aa6cb8ed. Le lot commun reprend main avec les
corrections serveur en attente, dont 296, 297 et le complément ledger 3dd52a95.

Parcours : figer le commit commun ; porter Cargo.toml et les trois paquets
Bridget de Cargo.lock à 0.8.53 ; reconstruire Android depuis cette source avec
Firebase et la lignée de signature existants, puis vérifier le code8053 contre
l'APK officiel0.8.52. Le candidat8052 reste une preuve de contrôle, jamais un
paquet à renommer en8053. Qualifier les binaires serveur issus du même commit
par le script de build et le parcours natif de maintenance. Publier les actifs
cohérents de v0.8.53 avec le garde Android, puis vérifier leurs téléchargements.
Ne pas écraser les actifs0.8.52 et ne pas contourner les clôtures non confirmées.

L'activation téléphone exige un appareil accessible, l'installation réelle et
le contrôle du parcours questions avec preuves visuelles. Elle n'est pas attestée
par les41tests JVM ni par la signature. Un arrêt de T3 reste soumis à une nouvelle
confirmation de François. Aucun build ni envoi concurrent n'est lancé par cette
réservation ; les blocages d'activation serveur et téléphone restent explicites.
