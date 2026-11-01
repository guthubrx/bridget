# Audit Code V15

Périmètre : /home/moi/.cache/bridget222-audit-source
Sources : commit 91083f5d453dab13a9dbd67647a8f6fea0b4917e ; mode fix.
Décision : **pass** ; note globale : A. Les notes de qualité par axe sont présentées ci-dessous ; le compteur de pénalités n'est pas une note de qualité.
Contrôles vérifiés : 14/14.

## Notes par axe

Qualité observée = somme des niveaux justifiés / maximum des critères notés. Niveaux ancrés de0 à4, poids figés. Le nombre de défauts ne donne plus la note : leurs pénalités restent un garde-fou. Couverture séparée. Note démontrée = points acquis / maximum de tous les critères applicables ; les inconnus ne rapportent aucun point. Aucun niveau justifié : non noté. Pas de moyenne globale des axes.

| Axe | Qualité observée | Points démontrés /100 | Critères notés | État |
|---|---|---|---|---|
| Sécurité | 75/100 (C) | 75 | 5/5 | Évalué |
| Sécurité IA | 75/100 (C) | 75 | 3/3 | Évalué |
| Fiabilité et exploitation | 50/100 (F) | 50 | 3/3 | Évalué |
| Tests et validation | 50/100 (F) | 50 | 4/4 | Évalué |
| Performance et ressources | 50/100 (F) | 50 | 4/4 | Évalué |
| Complexité algorithmique | 50/100 (F) | 50 | 3/3 | Évalué |
| Qualité du code et des erreurs | 50/100 (F) | 50 | 3/3 | Évalué |
| Architecture et dépendances | 75/100 (C) | 75 | 4/4 | Évalué |
| Responsabilité future et hygiène | 50/100 (F) | 50 | 3/3 | Évalué |
| Minimalisme et frugalité | 50/100 (F) | 50 | 3/3 | Évalué |
| Métier et fidélité des données | 75/100 (C) | 75 | 3/3 | Évalué |
| Traçabilité des exigences | 75/100 (C) | 75 | 2/2 | Évalué |

### Sécurité

| Critère | Niveau /4 | Justification |
|---|---|---|
| Analyse statique et validation des entrées | 3 | Niveau3 : Entrées fermées, autorité corrélée et scans triés ; le modèle OS même UID et les composants hors delta limitent la portée, pas une certification globale. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Protection des secrets courants | 3 | Niveau3 : Entrées fermées, autorité corrélée et scans triés ; le modèle OS même UID et les composants hors delta limitent la portée, pas une certification globale. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Maîtrise des secrets historiques | 3 | Niveau3 : Entrées fermées, autorité corrélée et scans triés ; le modèle OS même UID et les composants hors delta limitent la portée, pas une certification globale. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Authentification et séparation des droits | 3 | Niveau3 : Entrées fermées, autorité corrélée et scans triés ; le modèle OS même UID et les composants hors delta limitent la portée, pas une certification globale. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Maîtrise des dépendances vulnérables | 3 | Niveau3 : Entrées fermées, autorité corrélée et scans triés ; le modèle OS même UID et les composants hors delta limitent la portée, pas une certification globale. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |


### Sécurité IA

| Critère | Niveau /4 | Justification |
|---|---|---|
| Confidentialité et pseudonymisation | 3 | Niveau3 : Mémoire donnée/source, jamais droit ; refus humain et grant natif. Les prompts ne constituent pas un sandbox contre un administrateur du compte. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Permissions des appels externes | 3 | Niveau3 : Mémoire donnée/source, jamais droit ; refus humain et grant natif. Les prompts ne constituent pas un sandbox contre un administrateur du compte. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Résistance aux instructions malveillantes | 3 | Niveau3 : Mémoire donnée/source, jamais droit ; refus humain et grant natif. Les prompts ne constituent pas un sandbox contre un administrateur du compte. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |


### Fiabilité et exploitation

| Critère | Niveau /4 | Justification |
|---|---|---|
| Conservation et rotation des sauvegardes | 2 | Niveau2 : Transactions et reçus persistants examinés ; conservation de l'issue inconnue. Pas de campagne de sinistre ni bascule réelle dans cet audit. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Reprise des traitements interrompus | 2 | Niveau2 : Transactions et reçus persistants examinés ; conservation de l'issue inconnue. Pas de campagne de sinistre ni bascule réelle dans cet audit. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Continuité pendant une bascule | 2 | Niveau2 : Transactions et reçus persistants examinés ; conservation de l'issue inconnue. Pas de campagne de sinistre ni bascule réelle dans cet audit. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Restauration après sinistre | Non applicable | Périmètre de cet audit : code222 avant activation ; qualification du runtime livré et restauration complète restent T030/T031. Aucun verdict de déploiement ou de sinistre donné. Révision2 : delta mobile relu et test ciblé PASS ; preuves antérieures réutilisées pour les sources inchangées, sans nouvelle campagne générale. |


### Tests et validation

| Critère | Niveau /4 | Justification |
|---|---|---|
| Typage des sources et des tests | 2 | Niveau2 : Typage/assemblage et cas ciblés attestés ; couverture de branches utile, sans campagne exhaustive ni UI authentifiée annoncée. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Analyse de style et erreurs statiques | 2 | Niveau2 : Typage/assemblage et cas ciblés attestés ; couverture de branches utile, sans campagne exhaustive ni UI authentifiée annoncée. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Régressions et cas limites | 2 | Niveau2 : Typage/assemblage et cas ciblés attestés ; couverture de branches utile, sans campagne exhaustive ni UI authentifiée annoncée. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Qualification du runtime livré | Non applicable | Périmètre de cet audit : code222 avant activation ; qualification du runtime livré et restauration complète restent T030/T031. Aucun verdict de déploiement ou de sinistre donné. Révision2 : delta mobile relu et test ciblé PASS ; preuves antérieures réutilisées pour les sources inchangées, sans nouvelle campagne générale. |
| Tests interface et intégration des données | 2 | Niveau2 : Typage/assemblage et cas ciblés attestés ; couverture de branches utile, sans campagne exhaustive ni UI authentifiée annoncée. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |


### Performance et ressources

| Critère | Niveau /4 | Justification |
|---|---|---|
| Admission et saturation des demandes | 2 | Niveau2 : Tailles de documents/pages/lanes bornées et causes explicites ; pas de mesure représentative à très grand corpus. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Budget mémoire | 2 | Niveau2 : Tailles de documents/pages/lanes bornées et causes explicites ; pas de mesure représentative à très grand corpus. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Délais, annulation et libération | 2 | Niveau2 : Tailles de documents/pages/lanes bornées et causes explicites ; pas de mesure représentative à très grand corpus. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Mesures représentatives de latence et charge | 2 | Niveau2 : Tailles de documents/pages/lanes bornées et causes explicites ; pas de mesure représentative à très grand corpus. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |


### Complexité algorithmique

| Critère | Niveau /4 | Justification |
|---|---|---|
| Complexité des points d’entrée | 2 | Niveau2 : JOIN enlève le N+1 non borné ; tri/rappel et quotas techniques documentés. La taille durable du corpus reste libre et la latence non constante. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Bornes des parcours et allocations | 2 | Niveau2 : JOIN enlève le N+1 non borné ; tri/rappel et quotas techniques documentés. La taille durable du corpus reste libre et la latence non constante. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Coût des accès et écritures de données | 2 | Niveau2 : JOIN enlève le N+1 non borné ; tri/rappel et quotas techniques documentés. La taille durable du corpus reste libre et la latence non constante. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |


### Qualité du code et des erreurs

| Critère | Niveau /4 | Justification |
|---|---|---|
| Gestion des refus et messages utilisateur | 2 | Niveau2 : Refus et reprises visibles dans CLI/Kotlin ; circuit Android réel encore à vérifier à la livraison. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Gestion des erreurs partielles | 2 | Niveau2 : Refus et reprises visibles dans CLI/Kotlin ; circuit Android réel encore à vérifier à la livraison. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Ergonomie des parcours concernés | 2 | Niveau2 : Refus et reprises visibles dans CLI/Kotlin ; circuit Android réel encore à vérifier à la livraison. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |


### Architecture et dépendances

| Critère | Niveau /4 | Justification |
|---|---|---|
| Frontières et résolution des dépendances | 3 | Niveau3 : Une autorité de soumission, Fleet et domaine de profils existants ; aucune dépendance nouvelle. Les adaptateurs existants ne sont pas réaudités exhaustivement. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Maîtrise des cycles | 3 | Niveau3 : Une autorité de soumission, Fleet et domaine de profils existants ; aucune dépendance nouvelle. Les adaptateurs existants ne sont pas réaudités exhaustivement. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Séparation construction et exécution | 3 | Niveau3 : Une autorité de soumission, Fleet et domaine de profils existants ; aucune dépendance nouvelle. Les adaptateurs existants ne sont pas réaudités exhaustivement. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Assemblages et dépendances dynamiques | 3 | Niveau3 : Une autorité de soumission, Fleet et domaine de profils existants ; aucune dépendance nouvelle. Les adaptateurs existants ne sont pas réaudités exhaustivement. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |


### Responsabilité future et hygiène

| Critère | Niveau /4 | Justification |
|---|---|---|
| Invariants et compromis expliqués | 2 | Niveau2 : Décisions et limites écrites, avis contradictoire partiel conservé ; charge de maintenance accrue par le périmètre complet demandé. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Périmètre et maintenabilité du changement | 2 | Niveau2 : Décisions et limites écrites, avis contradictoire partiel conservé ; charge de maintenance accrue par le périmètre complet demandé. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Revue contradictoire et responsabilité | 2 | Niveau2 : Décisions et limites écrites, avis contradictoire partiel conservé ; charge de maintenance accrue par le périmètre complet demandé. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |


### Minimalisme et frugalité

| Critère | Niveau /4 | Justification |
|---|---|---|
| Dépendances nécessaires en exploitation | 2 | Niveau2 : Réutilisation du stockage et du transport ; pas de moteur supplémentaire. Environ20–40 lignes de formatage potentiellement partageables, non mesure exhaustive. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Poids et ressources justifiés | 2 | Niveau2 : Réutilisation du stockage et du transport ; pas de moteur supplémentaire. Environ20–40 lignes de formatage potentiellement partageables, non mesure exhaustive. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Code mort et simplifications possibles | 2 | Niveau2 : Réutilisation du stockage et du transport ; pas de moteur supplémentaire. Environ20–40 lignes de formatage potentiellement partageables, non mesure exhaustive. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |


### Métier et fidélité des données

| Critère | Niveau /4 | Justification |
|---|---|---|
| Fidélité des données lues | 3 | Niveau3 : Profils et mémoires versionnés, provenance et tombstones, idempotence ; une réponse rapportée n'est pas une preuve de vérité métier. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Cohérence et idempotence des versions | 3 | Niveau3 : Profils et mémoires versionnés, provenance et tombstones, idempotence ; une réponse rapportée n'est pas une preuve de vérité métier. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Conformité aux règles et données de référence | 3 | Niveau3 : Profils et mémoires versionnés, provenance et tombstones, idempotence ; une réponse rapportée n'est pas une preuve de vérité métier. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |


### Traçabilité des exigences

| Critère | Niveau /4 | Justification |
|---|---|---|
| Lien constats, corrections et preuves | 3 | Niveau3 : Sources et tests liés à FR001–030, delta avant/après conservé ; FR031 reste une porte séparée. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |
| Couverture des exigences obligatoires | 3 | Niveau3 : Sources et tests liés à FR001–030, delta avant/après conservé ; FR031 reste une porte séparée. Pour atteindre4 : qualifier les limites d'intégration ou la mesure mentionnées, sans les déduire d'un test local. |



## Risque constaté

{"critical": 0, "high": 0, "medium": 0, "low": 0, "info": 0, "total": 0, "candidates": 0, "suppressed": 0}

- [refuted] MEDIUM : Exponential event growth with `ENABLE_WIKILINKS`
  /home/moi/.cache/bridget222-audit-source/Cargo.lock:1 ; c12f22eab138af1c77d3
  Le seul appel Parser du produit utilise Options::empty ; condition ENABLE_WIKILINKS absente, non pilotable par les données.
  Action : Réexaminer si les options du parseur changent ; aucune mise à jour de dépendance prétendue.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/crates/bridget-daemon/src/mobile/store.rs:947 ; 759639ecd93da925d906
  Constante synthétique dans un module de test ou une fixture d'intégration ; aucune valeur de compte de production.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/crates/bridget-daemon/tests/core_089_service_test.rs:71 ; 0caf085f2d09e199f547
  Constante synthétique dans un module de test ou une fixture d'intégration ; aucune valeur de compte de production.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/crates/bridget-daemon/tests/fixtures/artifacts/generated-provenance-v1.json:2 ; e7758adb035e1b7b029c
  Clé d'idempotence ou chaîne SQL littérale de test, pas un secret fournisseur.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/crates/bridget-daemon/tests/held159_e2e.rs:1198 ; 1ccba275685005e5acbc
  Clé d'idempotence ou chaîne SQL littérale de test, pas un secret fournisseur.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/crates/bridget-daemon/tests/held159_e2e.rs:1273 ; d620a3f055d3fdbe9a99
  Clé d'idempotence ou chaîne SQL littérale de test, pas un secret fournisseur.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/crates/bridget-daemon/tests/held159_e2e.rs:1778 ; eb6175c2e0c7e69007f0
  Clé d'idempotence ou chaîne SQL littérale de test, pas un secret fournisseur.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/crates/bridget-daemon/src/cli.rs:12121 ; 274c2ac1987724fd6536
  Clé d'idempotence ou chaîne SQL littérale de test, pas un secret fournisseur.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/crates/bridget-daemon/src/wrapper.rs:8802 ; 99646f503a41dcb1e4ef
  Constante synthétique dans un module de test ou une fixture d'intégration ; aucune valeur de compte de production.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/skills/bridget/references/commandes.md:1415 ; 4db296e5e1be4fb6851b
  Clé d'idempotence ou chaîne SQL littérale de test, pas un secret fournisseur.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/026-operations-greffe-central/contracts/greffe-authorization.example.json:4 ; 38571998a7598bcbc851
  Exemple de portée/identifiant ou fixture de protocole, pas un jeton de compte.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/038-regeneration-politique/contracts/greffe-authorization-refresh.example.json:4 ; f1727ff5aad87fca21d5
  Exemple de portée/identifiant ou fixture de protocole, pas un jeton de compte.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/090-codex-interactif/probe_shared_session.py:78 ; 00aef57d2b23fd220298
  Exemple de portée/identifiant ou fixture de protocole, pas un jeton de compte.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/148-identite-delegation/validation/glm-recette-reelle-r2.md:54 ; ce92ea010cfa7d0bf6fa
  Empreinte SHA-256 de programme ou de définition rapportée dans une recette, pas un secret.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/bridget-source-manifest-before.json:887 ; fc022f7caf03536f1cba
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/bridget-source-manifest-before.json:1401 ; 7f71e9553e14309aeaeb
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/bridget-source-manifest-before.json:1415 ; e6836994487846332c1f
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/bridget-source-manifest-before.json:1429 ; 63e0190c29fdda7cb2f9
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:37 ; ad05a3def43e81fcc265
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:52 ; cc6388e7fcabef574222
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:71 ; 9db463cc7a4af09a05aa
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:226 ; e45950a9c6e63fb9d229
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:336 ; 158c6cd8bf58905b0244
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:870 ; 1d307a36d5949af9477c
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:948 ; f52a07ecc1ca55709184
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/bridget-source-manifest-before.json:2187 ; 26f683fcf82bb0ff01ae
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1222 ; 75b42b286b82339c3d1d
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1223 ; 40f1e85c913739d637c8
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1224 ; 65c79b66a99f0470fc57
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1225 ; 1445b51490df37b5bf87
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1311 ; d6bd73ba6dcde938b616
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/155-demandes-enfant/validation/schema155-generation.json:20 ; 3c62cc615e54a94676e3
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/155-demandes-enfant/validation/schema155-generation.json:150 ; ccdb5bf316bc782d59d1
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/155-demandes-enfant/validation/schema155-generation.json:152 ; dc1275c1c3253345fd4a
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/155-demandes-enfant/validation/schema155-generation.json:153 ; 51df6a613f1fcafe8add
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/155-demandes-enfant/validation/schema155-generation.json:157 ; 0e596c2da0d13fa2bbcc
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/155-demandes-enfant/validation/schema155-generation.json:187 ; 0128e853546ce457c2df
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/155-demandes-enfant/validation/schema155-generation.json:188 ; b9dfa191ed20b52b05ed
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/155-demandes-enfant/validation/schema155-generation.json:207 ; c85a2bcce92ebb05a88b
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1778 ; 1ff44714c738da148774
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/156-guidage-file/validation/diff-156-after-T032.patch:245 ; 2f0fa9460150877c7f54
  Clé d'idempotence ou chaîne SQL littérale de test, pas un secret fournisseur.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:1866 ; 9fca9579c817408b3a11
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:2239 ; 28a55963bad11de9d465
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:2255 ; 83b9d24a7fcc23a00f3a
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:2259 ; b2ca3050bba5aa41dd96
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:2267 ; 0b38b30f5526b388600f
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/156-guidage-file/validation/diff-156.patch:245 ; 642e7c1486a259464a74
  Clé d'idempotence ou chaîne SQL littérale de test, pas un secret fournisseur.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/149-sous-agents-lineage/validation/t3-final-artifact-check.json:2291 ; 185cae8935e9dfdbfcc0
  Empreinte de fichier dans un manifeste de sources ou de génération, pas une clé d'authentification.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/168-routage-flotte/validation/release168.md:7 ; da0b0fc627212288b032
  Identifiants d'objets Git publics de la release, pas une autorisation.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/155-demandes-enfant/validation/diff-155.patch:25109 ; 4defb24dcc2052e837a4
  Copie historique d'une fixture ou d'un manifeste de fichiers ; même mécanisme non authentifiant.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/155-demandes-enfant/validation/diff-155.patch:25239 ; 89d1a83667a7035fd5d0
  Copie historique d'une fixture ou d'un manifeste de fichiers ; même mécanisme non authentifiant.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/155-demandes-enfant/validation/diff-155.patch:25241 ; b49bb1a3a7e082710ff4
  Copie historique d'une fixture ou d'un manifeste de fichiers ; même mécanisme non authentifiant.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/155-demandes-enfant/validation/diff-155.patch:25242 ; bf6468883478b7e8c138
  Copie historique d'une fixture ou d'un manifeste de fichiers ; même mécanisme non authentifiant.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/155-demandes-enfant/validation/diff-155.patch:25246 ; 740be850e425cd1def4c
  Copie historique d'une fixture ou d'un manifeste de fichiers ; même mécanisme non authentifiant.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/155-demandes-enfant/validation/diff-155.patch:25276 ; 6ec3098201706943c190
  Copie historique d'une fixture ou d'un manifeste de fichiers ; même mécanisme non authentifiant.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/155-demandes-enfant/validation/diff-155.patch:25277 ; 2ecd8841a91e555603c8
  Copie historique d'une fixture ou d'un manifeste de fichiers ; même mécanisme non authentifiant.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [refuted] HIGH : Detected a Generic API Key, potentially exposing access to various services and sensitive operations.
  /home/moi/.cache/bridget222-audit-source/specs/155-demandes-enfant/validation/diff-155.patch:25296 ; 201f864d74dea449d45f
  Copie historique d'une fixture ou d'un manifeste de fichiers ; même mécanisme non authentifiant.
  Action : Aucune rotation ni suppression d'une preuve : ce contenu n'est pas une authentification utilisable.
- [fixed] HIGH : Lecture de la désignation répétée pour chaque candidat
  /home/moi/.cache/bridget222-audit-source/crates/bridget-daemon/src/daemon/specialists222.rs:1230 ; 92cf3615e646eeb39b5f
  Le comparateur ancien exécutait roles_for pour chacun des n candidats ; le JOIN charge maintenant la désignation une seule fois. Validation analytique de source et tests fonctionnels, pas benchmark de charge.
  Action : JOIN specialist_roles.official dans specialist_candidates ; aucun SQL dans le comparateur.

## Vérification réalisée

- security.static : pass ; completed, 0 candidat(s).  Scanners révision2 requis par le protocole effectivement exécutés : 0 Semgrep/0 historique/56 signaux génériques/10 avis dépendances, mêmes catégories triées, aucun nouveau signal222.
- security.secrets : pass ; 56 signaux génériques triés dans leurs sources : fixtures, SQL, identifiants ou empreintes, pas de secret de compte attesté. Révision2 : mêmes empreintes de sources et mêmes57 signaux rapprochés ;9 dépendances héritées horsdelta conservées comme nonrésolues dans leur registre.
- security.history : pass ; completed, 0 candidat(s).  Scanners révision2 requis par le protocole effectivement exécutés : 0 Semgrep/0 historique/56 signaux génériques/10 avis dépendances, mêmes catégories triées, aucun nouveau signal222.
- security.dependencies : pass ; Contrôle différentiel222 : packages tiers Cargo identiques, verrous Gradle et composant Caddy inchangés. Neuf signaux hors delta conservés et transmis ; un chemin Rust vulnérable non activé, réfutation sourcée. Révision2 : mêmes empreintes de sources et mêmes57 signaux rapprochés ;9 dépendances héritées horsdelta conservées comme nonrésolues dans leur registre.
- security.authorization : pass ; Frontières agent/humain/appareil suivies ; portée bornée, aucun booléen client privilégié, recontrôle grant avant RELEASE. Limite même UID ADR011/027 explicite. Révision2 : delta mobile relu et test ciblé PASS ; preuves antérieures réutilisées pour les sources inchangées, sans nouvelle campagne générale.
- correctness.logic : pass ; Versions gelées, pas de changement d'identité ou de mission à l'import/conversion ; import CLI inactif et cas de mémoire/oubli vérifiés. Révision2 : delta mobile relu et test ciblé PASS ; preuves antérieures réutilisées pour les sources inchangées, sans nouvelle campagne générale.
- correctness.concurrency : pass ; Réserve prepared atomique, canon immutable, promotion autorisée, grant revérifié avant RELEASE ; lanes de capture bornées. Pas de test de charge hostile annoncé. Révision2 : delta mobile relu et test ciblé PASS ; preuves antérieures réutilisées pour les sources inchangées, sans nouvelle campagne générale.
- performance.complexity : pass ; N+1 non borné des candidats retiré par JOIN ; parcours bornés décrits. Rappel reste O(M log M), corpus non plafonné, aucune latence constante promise. Révision2 : delta mobile relu et test ciblé PASS ; preuves antérieures réutilisées pour les sources inchangées, sans nouvelle campagne générale.
- tests.regression : pass ; 13 daemon222 +1 transport +1 autorité181 PASS sur source vérifiée byte à byte ;9 modèles Android et assembleRelease. Les échecs de cadre sont conservés. Révision2 : delta mobile relu et test ciblé PASS ; preuves antérieures réutilisées pour les sources inchangées, sans nouvelle campagne générale.
- architecture.dependencies : pass ; AgentProfileStore/Fleet/ExecutionStore réutilisés ; modules222 portent des responsabilités distinctes, pas d'orchestrateur ni dépendance tierce nouvelle. Révision2 : delta mobile relu et test ciblé PASS ; preuves antérieures réutilisées pour les sources inchangées, sans nouvelle campagne générale.
- quality.minimalism : pass ; Recherche d'existant et arbitrages conservés ; potentiel20–40 lignes partagé seulement, sans nouvelle abstraction prématurée. Révision2 : delta mobile relu et test ciblé PASS ; preuves antérieures réutilisées pour les sources inchangées, sans nouvelle campagne générale.
- quality.llm : pass ; Un souvenir reste une donnée et une source ne donne pas une autorisation ; décisions et prompts séparés, responsabilités et limites expliquées. L'avis indépendant reste partiel. Révision2 : delta mobile relu et test ciblé PASS ; preuves antérieures réutilisées pour les sources inchangées, sans nouvelle campagne générale.
- reliability.observability : pass ; Refus fermés, next_action et portée, issue inconnue préservée ; capture unavailable visible si magasin disponible. Préparations ne deviennent pas des missions par reprise. Révision2 : delta mobile relu et test ciblé PASS ; preuves antérieures réutilisées pour les sources inchangées, sans nouvelle campagne générale.
- spec.traceability : pass ; Matrice FR001–FR030 relie mécanismes et témoins du delta. FR031, activation et UI réelle restent une porte de livraison distincte, non prouvée par cet audit de code. Révision2 : delta mobile relu et test ciblé PASS ; preuves antérieures réutilisées pour les sources inchangées, sans nouvelle campagne générale.

## Limites

Aucune limite déclarée pour les contrôles définis. Cela ne prouve pas l'absence de tout défaut.

## Minimalisme et responsabilité future

Potentiel minimalisme : ~20–40 lignes de formatage partageables à comportement constant ; estimation de lecture, pas suppression nécessaire prouvée.

Même responsable pour code et audit final ; revues Claude R1/R2 partielles, limites et traitements conservés. Aucun second orchestrateur. Delta mobile révision2 autoaudité, revue indépendante indisponible/admission inconnue conservée ; ne vaut pas APPROVE externe.

## Tendance

Comparer seulement des périmètres, contrôles et conditions équivalents.
