# Qualité181 — périmètre audité avant activation

Décision de livraison pass,14/14contrôles. Les notes suivantes mesurent les niveaux de preuve et de maturité du périmètre.50correspond à fonctionnel sur périmètre observé avec une limite ;75à robuste sur cas normaux et erreurs pertinents. Les réserves de runtime sont celles de la révision auditée, avant activation. Le reçu de livraison apporte les preuves de bascule ultérieures. Aucun score global de qualité calculé par moyenne.

## Notes par axe

Qualité observée = somme des niveaux justifiés / maximum des critères notés. Niveaux ancrés de0 à4, poids figés. Le nombre de défauts ne donne plus la note : leurs pénalités restent un garde-fou. Couverture séparée. Note démontrée = points acquis / maximum de tous les critères applicables ; les inconnus ne rapportent aucun point. Aucun niveau justifié : non noté. Pas de moyenne globale des axes.

| Axe | Qualité observée | Points démontrés /100 | Critères notés | État |
|---|---|---|---|---|
| Sécurité | 70/100 (C) | 70 | 5/5 | Évalué |
| Sécurité IA | 75/100 (C) | 75 | 3/3 | Évalué |
| Fiabilité et exploitation | 58/100 (F) | 58 | 3/3 | Évalué |
| Tests et validation | 60/100 (D) | 60 | 5/5 | Évalué |
| Performance et ressources | 69/100 (D) | 69 | 4/4 | Évalué |
| Complexité algorithmique | 67/100 (D) | 67 | 3/3 | Évalué |
| Qualité du code et des erreurs | 67/100 (D) | 67 | 3/3 | Évalué |
| Architecture et dépendances | 69/100 (D) | 69 | 4/4 | Évalué |
| Responsabilité future et hygiène | 58/100 (F) | 58 | 3/3 | Évalué |
| Minimalisme et frugalité | 58/100 (F) | 58 | 3/3 | Évalué |
| Métier et fidélité des données | 75/100 (C) | 75 | 3/3 | Évalué |
| Traçabilité des exigences | 75/100 (C) | 75 | 2/2 | Évalué |

### Sécurité

| Critère | Niveau /4 | Justification |
|---|---|---|
| Analyse statique et validation des entrées | 3 | Entrées strictes, SAST0candidat ; analyseRustSemgrep limitée, revue des frontières nécessaire. |
| Protection des secrets courants | 3 | Sites signalés relus et réfutés avec preuves ; aucune authentification de clé candidate tentée. |
| Maîtrise des secrets historiques | 2 | Historique delta borné vide9c8e..9c8e ; fichiers noncommittés scannés. Pas de garantie sur les anciens commits. |
| Authentification et séparation des droits | 3 | Négatifs propriétaire/appareil/projet et séparation agent/humain couverts, guard155préservé. |
| Maîtrise des dépendances vulnérables | 3 | Trois lockfiles explicitement analysés,175/280/147entrées,0avisOSV. Pas analyse d’atteignabilité ni promesse contre avis inconnus. |


### Sécurité IA

| Critère | Niveau /4 | Justification |
|---|---|---|
| Confidentialité et pseudonymisation | 3 | Poussée générique, projection sans callbacks/acteur, brouillon chiffré local. Textes explicites seulement après authentification. |
| Permissions des appels externes | 3 | Aucune réponse humaine via outil agent, pas de gain de droit délégué ; demandes secrets refusées. |
| Résistance aux instructions malveillantes | 3 | JSON validé et textes inertes Compose ; aucune instruction du modèle ne fournit de principal/route ou prompt de substitution. |


### Fiabilité et exploitation

| Critère | Niveau /4 | Justification |
|---|---|---|
| Conservation et rotation des sauvegardes | 2 | Règles existantes actif+retour conservées ; procédures de sauvegarde lues, table additive incluse par backupSQLite. Rotation181pas encore livrée. |
| Reprise des traitements interrompus | 3 | Redémarrage daemon et commande notification coupée reprennent mêmeIDsansperte. |
| Continuité pendant une bascule | 2 | Maintenance bloquée si question active, test ; bascule de production181 attend livraison. |
| Restauration après sinistre | Non applicable | Pas de changement de stratégie de restauration après sinistre. Ce lot ajoute une tableSQLite sauvegardée par le mécanisme existant ; aucune qualificationDRrevendiquée. |


### Tests et validation

| Critère | Niveau /4 | Justification |
|---|---|---|
| Typage des sources et des tests | 3 | Rust/Kotlin construits et tests typés passent, avertissements historiques conservés. |
| Analyse de style et erreurs statiques | 2 | LintAndroidPASS, rustfmt des nouveaux modules. Pas de clippyglobal, style condensé des extensions historiques reste une limite. |
| Régressions et cas limites | 3 | Tests sur courses,contrôle appareil,révision,replay,capacités,limites32 et purge ; régression155casaffecté repris. Pas de FULL. |
| Qualification du runtime livré | 2 | Runtime isoléAndroid/native/FCMr éels qualifiés ; binaires de production181nonactivés au moment de cet audit. |
| Tests interface et intégration des données | 2 | Circuits UI réels et4modesréponse ; voix retour contrôlé/T3serveurcontrôlé distincts des moteurs réels nonqualifiés. |


### Performance et ressources

| Critère | Niveau /4 | Justification |
|---|---|---|
| Admission et saturation des demandes | 3 | Quotas de demandes et waits, saturation32testée, pas d’allocation non bornée nouvelle. |
| Budget mémoire | 3 | Bornes64K/32K/120K, pagination et2048demandes actives/100000tombstones au maximum ; pas de mesure pic mémoire propreàcharge. |
| Délais, annulation et libération | 3 | Délais/annulation/WaitPermit et socket deadline existants réutilisés, IPC hors verrou. |
| Mesures représentatives de latence et charge | 2 | Durées de tests réels et reprise fonctionnelle mesurées ; pas de banc2048demandes/100000tombstones ni p95. |


### Complexité algorithmique

| Critère | Niveau /4 | Justification |
|---|---|---|
| Complexité des points d’entrée | 2 | Points d’entrée existants étendus, nouveau domaine isolé ; grands modules historiques demeurent et quelques branches sont condensées. |
| Bornes des parcours et allocations | 3 | Validation linéaire sur entrée bornée, parcours paginés sousbudget, libération horslock. |
| Coût des accès et écritures de données | 3 | Clés/indexétat/commande/rétention ; purge secondelecture0écriture et indexpartiel. Pas de profilSQLàcharge maximale. |


### Qualité du code et des erreurs

| Critère | Niveau /4 | Justification |
|---|---|---|
| Gestion des refus et messages utilisateur | 3 | Refus distincts/inconnu/confirmé guidés, ancienserveur noncompatible explicité, FR/EN. |
| Gestion des erreurs partielles | 3 | Sauvegarde notification avant réseau, jamais nouveaucommandIDincertain ; rétablissement réel. |
| Ergonomie des parcours concernés | 2 | Volet/agrandissement/provenance/choixlibres fonctionnent ; reconnaissanceaudioetT3réelencorenonqualifiés. |


### Architecture et dépendances

| Critère | Niveau /4 | Justification |
|---|---|---|
| Frontières et résolution des dépendances | 3 | DomaineBridget sansT3, adaptateurconf iné, refus155séparés ; pas de nouvellelib. |
| Maîtrise des cycles | 3 | Lecture des imports et flux : transporttypes->daemonstore, adaptateur->protocole, aucunimportT3dansdomaine. |
| Séparation construction et exécution | 3 | SDK/buildconfig public séparés du namespace/credentials ; binaire/associationpreuve distingués. |
| Assemblages et dépendances dynamiques | 2 | Assemblages concernéspassent etmoteurvocalIntent facultatif ; pas de qualificationtoutesversionshôtes/fournisseurs. |


### Responsabilité future et hygiène

| Critère | Niveau /4 | Justification |
|---|---|---|
| Invariants et compromis expliqués | 3 | Invariants droits/idempotence/ACKexplicites, source source-of-truth, compromis cons ignés. |
| Périmètre et maintenabilité du changement | 2 | Périmètrecomplet conservé et réutilisation14éléments ; orchestration UI/notification encoredupliquée légère. |
| Revue contradictoire et responsabilité | 2 | Revueautrefournisseur demandéeviaBridget,échecdeadlineconservé ; pas de verdictindépendant ni autoritéinventée. |


### Minimalisme et frugalité

| Critère | Niveau /4 | Justification |
|---|---|---|
| Dépendances nécessaires en exploitation | 3 | Pas nouvelle dépendance externe/service ; MCP/IPC/FCM/SDKvocaldéjàdisponiblesréutilisés. |
| Poids et ressources justifiés | 2 | Corps/quotas bornés etHTML469207octets ; poidsbinairefinaletchargegrandeéchelleattendentmesures. |
| Code mort et simplifications possibles | 2 | Lecturecibléedoublons et branches ; potentiel environ12lignes d’orchestration réponse communes, estimation nonmesurée, extractionnonjustifiéeauprixd’unnouveauwrapper. |


### Métier et fidélité des données

| Critère | Niveau /4 | Justification |
|---|---|---|
| Fidélité des données lues | 3 | Textes/options/descriptions et sélections/texteslibrespréservés, callbacks corrélés et aucune interprétation. |
| Cohérence et idempotence des versions | 3 | Révision/clé/enveloppe immuables, lost-receiptretrymêmecommande et19tests181 ciblés. |
| Conformité aux règles et données de référence | 3 | Choixutilisateurlocalseulement/T3facultatif/voixéditionrespectés ; aucune nouvelleautorisationexécution. |


### Traçabilité des exigences

| Critère | Niveau /4 | Justification |
|---|---|---|
| Lien constats, corrections et preuves | 3 | Findings avant/après avec empreintes etfragments ; rawcapturesprivéesHTMLversionné. |
| Couverture des exigences obligatoires | 3 | FR001–013 reliées ; SC006réservéàlivraisonfuture, limites explicitespas masquées. |
