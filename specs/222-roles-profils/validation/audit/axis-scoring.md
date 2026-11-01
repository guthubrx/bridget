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
