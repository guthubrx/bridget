# Plan
1. Conserver les avis primaires et versions corrigées ; employer le correctif Gradle8 compatible avec AGP8 plutôt qu’une migration AGP9.
2. Actualiser rusqlite et sa copie SQLite, contrôler la compatibilité du SQL et les bases existantes. Réutiliser le cache Cargo du checkout principal via scripts/build.py.
3. Mettre Gradle à8.14.6 avec SHA256 officiel, corriger le graphe test GooglePlay et vérifier AGP/JDK. Recompiler et signer Android avec la clé existante.
4. Sauvegarder Caddy, sa configuration et son unité ; valider la nouvelle version avant activation ; tester tous les sites servis avant/après. Mettre WebView à jour via GooglePlay officiel.
5. Tester, relire, intégrer et pousser ; compiler les sources livrées Mac/Linux, activer, vérifier, nettoyer sans force.

Aucune délégation neuve : routage bloqué pour ce tour ; revue directe explicitement distincte d’une revue indépendante.


Construction Android : scripts/android-build.sh sélectionne le JDK privé corrigé17.0.21+9, ou JAVA_HOME explicitement fourni et vérifié. Le JDK Ubuntu17.0.20.1 reste présent pour les autres travaux ; aucune alternative Java globale ni processus Java étranger n’est modifié. Le JDK Temurin publié le22octobre suit la mise à jour de sécurité d’octobre. Gradle8.14.6, Netty4.1.139 et Protobuf3.25.8 corrigent aussi le graphe des outils de test, désormais verrouillé.

SQLite effectivement fourni par libsqlite3-sys0.39.0 :3.54.0 (64bits). Le correctif3.54.1 vise un débordement propre au32bits. Les binaires livrés sont x86_64Linux/aarch64Mac ; Android utilise son SQLite système. La feature fallible_uint de rusqlite0.41 conserve les conversions u64/usize précédentes avec vérification de bornes.


[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/178-securite-dependances/plan.md#extrait-1).
