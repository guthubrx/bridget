# Audit241 — candidat avant activation

Décision de code calculée : pass,14contrôles/14. Les notes ci-dessous sont
les niveaux de qualité justifiés ; le100du compteur de pénalités signifie
aucun défaut confirmé ouvert dans le candidat, pas une qualité parfaite.
Les portes d’activation et les mesures de production restent séparées.

# Audit Code V15

Périmètre : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/241-fiabilite-usage
Sources : commit 04ad641e85c8e131cef4fcd17c43a10980b15104 ; mode fix.
Décision : **pass** ; note globale : A. Les notes de qualité par axe sont présentées ci-dessous ; le compteur de pénalités n'est pas une note de qualité.
Contrôles vérifiés : 14/14.

# Audit Code V15

Périmètre : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/241-fiabilite-usage
Sources : commit 04ad641e85c8e131cef4fcd17c43a10980b15104 ; mode fix.
Décision : **pass** ; note globale : A. Les notes de qualité par axe sont présentées ci-dessous ; le compteur de pénalités n'est pas une note de qualité.
Contrôles vérifiés : 14/14.

## Notes par axe

Qualité observée = somme des niveaux justifiés / maximum des critères notés. Niveaux ancrés de0 à4, poids figés. Le nombre de défauts ne donne plus la note : leurs pénalités restent un garde-fou. Couverture séparée. Note démontrée = points acquis / maximum de tous les critères applicables ; les inconnus ne rapportent aucun point. Aucun niveau justifié : non noté. Pas de moyenne globale des axes.

| Axe | Qualité observée | Points démontrés /100 | Critères notés | État |
|---|---|---|---|---|
| Sécurité | 65/100 (D) | 65 | 5/5 | Évalué |
| Sécurité IA | 75/100 (C) | 75 | 3/3 | Évalué |
| Fiabilité et exploitation | 58/100 (F) | 58 | 3/3 | Évalué |
| Tests et validation | 69/100 (D) | 69 | 4/4 | Évalué |
| Performance et ressources | 69/100 (D) | 69 | 4/4 | Évalué |
| Complexité algorithmique | 67/100 (D) | 67 | 3/3 | Évalué |
| Qualité du code et des erreurs | 67/100 (D) | 67 | 3/3 | Évalué |
| Architecture et dépendances | 75/100 (C) | 75 | 4/4 | Évalué |
| Responsabilité future et hygiène | 67/100 (D) | 67 | 3/3 | Évalué |
| Minimalisme et frugalité | 67/100 (D) | 67 | 3/3 | Évalué |
| Métier et fidélité des données | 75/100 (C) | 75 | 3/3 | Évalué |
| Traçabilité des exigences | 75/100 (C) | 75 | 2/2 | Évalué |

### Sécurité

| Critère | Niveau /4 | Justification |
|---|---|---|
| Analyse statique et validation des entrées | 3 | Entrées et variantes fermées, capacité négociée et tests de refus ; Semgrep sans candidat. Pas de preuve d’absence universelle de défaut. |
| Protection des secrets courants | 3 | Journaux de phase sans corps/trame/secret ; Gitleaks fichiers et environnement de sonde borné. |
| Maîtrise des secrets historiques | 2 | Historique du delta seulement : avant commit l’intervalle04ad..04ad est vide ; fichiers candidats scannés séparément. Aucun audit historique complet. |
| Authentification et séparation des droits | 3 | Autorité de mission issue uniquement de provider_turn attesté ; propriétaire/source/instances révoquées et relues après I/O, aucun historique promu. |
| Maîtrise des dépendances vulnérables | 2 | Douze signaux qualifiés avec contre-revue ; préconditions Rust/APK absentes. Huit entrées Go hors241 restent ouvertes sur Caddy, pas de patch prétendu. |


### Sécurité IA

| Critère | Niveau /4 | Justification |
|---|---|---|
| Confidentialité et pseudonymisation | 3 | Pas de nouvelle transmission métier ni credential ; sondes fixes, sortie non renvoyée et journaux fermés ; preuves fabriquées. |
| Permissions des appels externes | 3 | Aucun Bash ajouté ; profils projetés et exigences d’écriture qualifiés avant placement ; axes humains conservés. |
| Résistance aux instructions malveillantes | 3 | Texte de mission choisit seulement des outils fermés, jamais des arguments de sonde ; PATH exclut les exécutables du dépôt. Pas de compréhension de tout programme. |


### Fiabilité et exploitation

| Critère | Niveau /4 | Justification |
|---|---|---|
| Conservation et rotation des sauvegardes | 2 | Réouverture SQLite et lectures d’anciens formats testées ; scripts natifs de livraison prévoient sauvegardes, aucune restauration réelle prétendue avant activation. |
| Reprise des traitements interrompus | 3 | Vraies sockets et SQLite : réponse perdue, même clé concurrente, reprise après fin du tour, révocation/cancel conservées. |
| Continuité pendant une bascule | 2 | Les images candidat/retour sont qualifiées sans fournisseur ; continuité de la bascule réelle reste gate de livraison, aucune coupure de T3 permise. |
| Restauration après sinistre | Non applicable | Hors modification241 : aucun mécanisme de restauration après sinistre modifié ; pas de restauration réelle exécutée. |


### Tests et validation

| Critère | Niveau /4 | Justification |
|---|---|---|
| Typage des sources et des tests | 3 | Assemblages Rust et tests des types/JSON ; Python53cas. Pas de nouveau TypeScript/Go applicatif. |
| Analyse de style et erreurs statiques | 2 | Rust compile et Semgrep ; avertissements historiques conservés, aucun formatage général des grands modules pour gonfler le diff. |
| Régressions et cas limites | 3 | 25cas241 finaux Mac/Linux, suites affectées et contre-exemples des revues ; tests non applicables/ignorés non comptés comme réussis. |
| Qualification du runtime livré | 3 | Binaire réel candidat et retour Mac : plan/export/matérialisation sans fournisseur ni workspaceGit, code/source inchangés ; images Linux qualifiées séparément avant installation. |
| Tests interface et intégration des données | Non applicable | Aucun composant graphique Android modifié ; build/version/signature seulement. Pas de recette UI ni installation téléphone revendiquée. |


### Performance et ressources

| Critère | Niveau /4 | Justification |
|---|---|---|
| Admission et saturation des demandes | 3 | Sondes groupées avant sélection ; aucune Task ni réservation sur refus ; limite de nombre de worktrees supprimée sans retirer les claims uniques. |
| Budget mémoire | 3 | Copie/hachage progressifs et bornes de métadonnées conservées ; limite32Gio retirée, overflow vérifié ; NOTICE avant gros contenu divergent. |
| Délais, annulation et libération | 3 | Budget global du raccordement, une reprise avant mutation, annulation terminale ; sonde3s et arrêt du seul enfant créé. |
| Mesures représentatives de latence et charge | 2 | Vue lente et réponses indépendantes mesurées ; latence de relais observée. Cause globale historiqueJSONL et taux en production non établis. |


### Complexité algorithmique

| Critère | Niveau /4 | Justification |
|---|---|---|
| Complexité des points d’entrée | 2 | Extension des points natifs plutôt que moteur supplémentaire ; native_delegation/daemon restent volumineux, parseur volontairement limité. |
| Bornes des parcours et allocations | 3 | Sondes O(candidats+profils), ensemble fermé d’outils ; trames et cartes bornées, flux de fichiers ; pas d’allocation au volume total. |
| Coût des accès et écritures de données | 3 | Statut par clé et pages vérifiés, lecture4097lignes indépendante ; suppression du scan total des tâches pour quotaWT ; pas de nouvelle table. |


### Qualité du code et des erreurs

| Critère | Niveau /4 | Justification |
|---|---|---|
| Gestion des refus et messages utilisateur | 3 | Codes et motifs disponibles/absents/inutilisables/inconnus, portée locale et prochaine action ; aucune cause de droits/réseau inventée. |
| Gestion des erreurs partielles | 3 | Avant envoi reprend ; après envoi récupère le même canon/reçu ; issue inconnue conservée sans nouvelle mission ni faux succès. |
| Ergonomie des parcours concernés | 2 | Réserves par outil, pas de nouveaux paramètres requis et pas de demande humaine répétée ; bénéfice quotidien reste à mesurer après activation. |


### Architecture et dépendances

| Critère | Niveau /4 | Justification |
|---|---|---|
| Frontières et résolution des dépendances | 3 | Capacité WorkloadToolsV1 négociée, Task commune et protections de sources réutilisées ; pas de nouveau store/runtime. |
| Maîtrise des cycles | 3 | Graphe Cargo troiscrates inchangé, shlex feuille déjà verrouillée ajoutée directement au daemon ; aucune dépendance inverse. |
| Séparation construction et exécution | 3 | Sondes fixes sans installer ; Cargo via build.py, artefacts signés hors target ; cfg(test) Python ne modifie pas les lancements de production. |
| Assemblages et dépendances dynamiques | 3 | Mac signé et Linux construits depuis mêmes sources figées ; lecteurs compatibles vérifiés, ancien240 refusé comme retour. |


### Responsabilité future et hygiène

| Critère | Niveau /4 | Justification |
|---|---|---|
| Invariants et compromis expliqués | 3 | ADR autorité durable, limites et décisions de plafonds explicites ; maintien des révocations et absence de promotion historique. |
| Périmètre et maintenabilité du changement | 2 | Pas de couche parallèle ; réutilisations recherchées avant édition. Extension importante sur gros modules, limite de maintenance conservée. |
| Revue contradictoire et responsabilité | 3 | Plan puis trois rondes indépendantes ont produit des corrections concrètes ; R4 lue, objections disposées explicitement dans validation.md ; autoanalyse indépendante=false. |


### Minimalisme et frugalité

| Critère | Niveau /4 | Justification |
|---|---|---|
| Dépendances nécessaires en exploitation | 3 | Un seul parseur lexical shlex2.0.1 déjà dans lock ; aucune installation/runtime de modèle/service ajoutée. |
| Poids et ressources justifiés | 3 | Caches communs via build.py, pas de target par revue ; volume utilisateur limité par espace réel ; poids des binaires conservé dans reçus. |
| Code mort et simplifications possibles | 2 | Suppression quotas/scan de Task et entretien synchrone ; pas de campagne globale de code mort ni refonte des gros modules. |


### Métier et fidélité des données

| Critère | Niveau /4 | Justification |
|---|---|---|
| Fidélité des données lues | 3 | État/résultat durable par clé, mêmes canons, pas de mutation des missions existantes ; inventaires sources vérifiés. |
| Cohérence et idempotence des versions | 3 | Perte de réponse/concurrence testées sans doublon ; contrats lecture/retour garantissent distinction historique/nouveau. |
| Conformité aux règles et données de référence | 3 | Exclusions utilisateur T3/mission conservées, autorité attestée et machines imposées ; aucune autorisation supplémentaire. |


### Traçabilité des exigences

| Critère | Niveau /4 | Justification |
|---|---|---|
| Lien constats, corrections et preuves | 3 | Résultats bruts, SHA de sources et revues conservés ; échecs de test/scanner non effacés, compteurs non additionnés. |
| Couverture des exigences obligatoires | 3 | 13FR/7SC confrontés au code et tests ; Converge2 sans tâches manquantes, portes opérationnelles encore distinctes. |



## Risque constaté

{"critical": 0, "high": 0, "medium": 0, "low": 0, "info": 0, "total": 0, "candidates": 0, "suppressed": 0}

- [refuted] MEDIUM : Exponential event growth with `ENABLE_WIKILINKS`
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/241-fiabilite-usage/Cargo.lock:1 ; 68ff57c2a101560b1b67
  Précondition ENABLE_WIKILINKS absente : unique Parser de attach_renderer.rs utilise Options::empty(). La bibliothèque n’est pas déclarée patchée.
  Action : Conserver la condition et le périmètre ; requalifier si les options du parseur, classpaths ou artefacts livrés changent. Caddy : qualification distincte du binaire actuel avant toute conclusion.
- [refuted] MEDIUM : Quadratic parsing of nested image wikilinks with `ENABLE_WIKILINKS`
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/241-fiabilite-usage/Cargo.lock:1 ; a51adb6fdc45e001bb74
  Précondition ENABLE_WIKILINKS absente : unique Parser de attach_renderer.rs utilise Options::empty(). La bibliothèque n’est pas déclarée patchée.
  Action : Conserver la condition et le périmètre ; requalifier si les options du parseur, classpaths ou artefacts livrés changent. Caddy : qualification distincte du binaire actuel avant toute conclusion.
- [refuted] HIGH : Google Guava: CompactHashMap/CompactHashSet/MapMakerInternalMap deserialization triggers uncontrolled memory allocation
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/241-fiabilite-usage/android/app/gradle.lockfile:1 ; 2d0f2529ddad630b049b
  Guava appartient au classpath de build/instrumentation, pas releaseRuntimeClasspath ; aucun ObjectInputStream/readObject dans les sources Android. Réfutation pour APK241, pas réparation de l’outillage.
  Action : Conserver la condition et le périmètre ; requalifier si les options du parseur, classpaths ou artefacts livrés changent. Caddy : qualification distincte du binaire actuel avant toute conclusion.
- [refuted] HIGH : Google Guava: CompactHashMap/CompactHashSet/MapMakerInternalMap deserialization triggers uncontrolled memory allocation
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/241-fiabilite-usage/android/buildscript-gradle.lockfile:1 ; 51048ab9c3ce6c607110
  Guava appartient au classpath de build/instrumentation, pas releaseRuntimeClasspath ; aucun ObjectInputStream/readObject dans les sources Android. Réfutation pour APK241, pas réparation de l’outillage.
  Action : Conserver la condition et le périmètre ; requalifier si les options du parseur, classpaths ou artefacts livrés changent. Caddy : qualification distincte du binaire actuel avant toute conclusion.
- [refuted] MEDIUM : [Inclusion dans241 réfutée ; Caddy distinct reste ouvert] [Inclusion dans241 réfutée ; Caddy distinct reste ouvert] Go JOSE: Panic during CBC-HMAC processing
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/241-fiabilite-usage/deploy/caddy-security/go.mod:1 ; 57b4e73d275f71003132
  Module Go distinct inchangé, non compilé ni livré dans241. Réfutation limitée à son inclusion dans ce candidat. Avis NON RÉFUTÉ pour le Caddy déployé ; qualification séparée encore ouverte.
  Action : Conserver la condition et le périmètre ; requalifier si les options du parseur, classpaths ou artefacts livrés changent. Caddy : qualification distincte du binaire actuel avant toute conclusion.
- [refuted] HIGH : [Inclusion dans241 réfutée ; Caddy distinct reste ouvert] [Inclusion dans241 réfutée ; Caddy distinct reste ouvert] Go JOSE: DoS when parsing JWKs that contain RSA keys
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/241-fiabilite-usage/deploy/caddy-security/go.mod:1 ; 8236b1d29df167fbba26
  Module Go distinct inchangé, non compilé ni livré dans241. Réfutation limitée à son inclusion dans ce candidat. Avis NON RÉFUTÉ pour le Caddy déployé ; qualification séparée encore ouverte.
  Action : Conserver la condition et le périmètre ; requalifier si les options du parseur, classpaths ou artefacts livrés changent. Caddy : qualification distincte du binaire actuel avant toute conclusion.
- [refuted] MEDIUM : [Inclusion dans241 réfutée ; Caddy distinct reste ouvert] [Inclusion dans241 réfutée ; Caddy distinct reste ouvert] Go JOSE: Panics in JSON parsing
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/241-fiabilite-usage/deploy/caddy-security/go.mod:1 ; ad3a1a690f21974f9018
  Module Go distinct inchangé, non compilé ni livré dans241. Réfutation limitée à son inclusion dans ce candidat. Avis NON RÉFUTÉ pour le Caddy déployé ; qualification séparée encore ouverte.
  Action : Conserver la condition et le périmètre ; requalifier si les options du parseur, classpaths ou artefacts livrés changent. Caddy : qualification distincte du binaire actuel avant toute conclusion.
- [refuted] HIGH : [Inclusion dans241 réfutée ; Caddy distinct reste ouvert] [Inclusion dans241 réfutée ; Caddy distinct reste ouvert] Go JOSE: DoS when parsing JWKs that contain RSA keys
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/241-fiabilite-usage/deploy/caddy-security/go.mod:1 ; b0833d5a19c59a5e37b6
  Module Go distinct inchangé, non compilé ni livré dans241. Réfutation limitée à son inclusion dans ce candidat. Avis NON RÉFUTÉ pour le Caddy déployé ; qualification séparée encore ouverte.
  Action : Conserver la condition et le périmètre ; requalifier si les options du parseur, classpaths ou artefacts livrés changent. Caddy : qualification distincte du binaire actuel avant toute conclusion.
- [refuted] MEDIUM : [Inclusion dans241 réfutée ; Caddy distinct reste ouvert] [Inclusion dans241 réfutée ; Caddy distinct reste ouvert] Panic on two-block frames in github.com/klauspost/compress
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/241-fiabilite-usage/deploy/caddy-security/go.mod:1 ; 5262d4a2eed3d3d3cfcd
  Module Go distinct inchangé, non compilé ni livré dans241. Réfutation limitée à son inclusion dans ce candidat. Avis NON RÉFUTÉ pour le Caddy déployé ; qualification séparée encore ouverte.
  Action : Conserver la condition et le périmètre ; requalifier si les options du parseur, classpaths ou artefacts livrés changent. Caddy : qualification distincte du binaire actuel avant toute conclusion.
- [refuted] MEDIUM : [Inclusion dans241 réfutée ; Caddy distinct reste ouvert] [Inclusion dans241 réfutée ; Caddy distinct reste ouvert] The golang.org/x/crypto/openpgp package is unmaintained, unsafe by design, and has known security issues
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/241-fiabilite-usage/deploy/caddy-security/go.mod:1 ; 3e3befc544bb49ed8c60
  Module Go distinct inchangé, non compilé ni livré dans241. Réfutation limitée à son inclusion dans ce candidat. Avis NON RÉFUTÉ pour le Caddy déployé ; qualification séparée encore ouverte.
  Action : Conserver la condition et le périmètre ; requalifier si les options du parseur, classpaths ou artefacts livrés changent. Caddy : qualification distincte du binaire actuel avant toute conclusion.
- [refuted] MEDIUM : [Inclusion dans241 réfutée ; Caddy distinct reste ouvert] [Inclusion dans241 réfutée ; Caddy distinct reste ouvert] Avoid quadratic complexity when coalescing text nodes in html in golang.org/x/net
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/241-fiabilite-usage/deploy/caddy-security/go.mod:1 ; 82adf7d7f0e032317d5f
  Module Go distinct inchangé, non compilé ni livré dans241. Réfutation limitée à son inclusion dans ce candidat. Avis NON RÉFUTÉ pour le Caddy déployé ; qualification séparée encore ouverte.
  Action : Conserver la condition et le périmètre ; requalifier si les options du parseur, classpaths ou artefacts livrés changent. Caddy : qualification distincte du binaire actuel avant toute conclusion.
- [refuted] MEDIUM : [Inclusion dans241 réfutée ; Caddy distinct reste ouvert] [Inclusion dans241 réfutée ; Caddy distinct reste ouvert] Check all ancestor namespace transitions when rendering raw text in html in golang.org/x/net
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/241-fiabilite-usage/deploy/caddy-security/go.mod:1 ; 641007b1a98249700b64
  Module Go distinct inchangé, non compilé ni livré dans241. Réfutation limitée à son inclusion dans ce candidat. Avis NON RÉFUTÉ pour le Caddy déployé ; qualification séparée encore ouverte.
  Action : Conserver la condition et le périmètre ; requalifier si les options du parseur, classpaths ou artefacts livrés changent. Caddy : qualification distincte du binaire actuel avant toute conclusion.

## Vérification réalisée

- security.static : pass ; completed, 0 candidat(s). 
- security.secrets : pass ; completed, 0 candidat(s). 
- security.history : pass ; completed, 0 candidat(s).  Historique limité à 04ad641e85c8e131cef4fcd17c43a10980b15104..04ad641e85c8e131cef4fcd17c43a10980b15104; les commits antérieurs sont hors périmètre.
- security.dependencies : pass ; Douze signaux qualifiés pour le périmètre241 ; huit Go horslivrable restent ouverts pour Caddy. Ce contrôle ne qualifie pas Caddy déployé.
- security.authorization : pass ; Propriétaire/snapshot/révocations et source attestée testés, pas de nouveau droit.
- correctness.logic : pass ; Overflow, espace réel, NOTICE prioritaire et intégrité des fichiers ; pas de quota implicite réintroduit.
- correctness.concurrency : pass ; Réponse perdue et même clé concurrente sans doublon ; annulation et budget uniques, pas de mutation rejouée.
- performance.complexity : pass ; Bornes mémoire/métadonnées justifiées, fichiers en flux ; collecteur hors mutex/dispatch ; pas de scan global des Task pour quotaWT.
- tests.regression : pass ; Tests utiles Mac/Linux/Python, cas réels de sockets et contre-exemples de revue ; preuves recouvrantes non additionnées.
- architecture.dependencies : pass ; Troiscrates et magasins existants ; shlex feuille déjà verrouillée, aucun service/moteur doublonné.
- quality.minimalism : pass ; Réutilisation native avec recherche préalable, suppression scans/quotas/entretien synchrone ; grands modules historiques restent une limite.
- quality.llm : pass ; Invariants explicités, revues adverses et contredits corrigés ; analyse propre independent=false ; aucune preuve humaine fabriquée.
- reliability.observability : pass ; Phase/code/durée sans corps de trame ; défaut lent reproduit/corrigé, cause de tous anciens JSONL reste inconnue.
- spec.traceability : pass ; 13FR/7SC confrontés code/tests, Converge2 ; activation et Git restent des portes opérationnelles séparées.

## Limites

Aucune limite déclarée pour les contrôles définis. Cela ne prouve pas l'absence de tout défaut.

## Minimalisme et responsabilité future

Aucun nouveau service/store/runtime. Une arête shlex2.0.1 déjà verrouillée remplace les mots naïfs ; collecteur unique hors réponse, pas un thread par demande. Quotas de volume/nombre supprimés au profit de flux/espace et métadonnées bornées. Grands modules historiques non refondus.

Analyse propriétaire independent=false. Revues Claude distinctes avec résultats et ACK, objections traitées ; limites de langage et causalité JSONL conservées. Aucune mission métier ni application T3 redémarrée pour vérifier. Caddy distinct reste ouvert, jamais déclaré corrigé.

## Tendance

Comparer seulement des périmètres, contrôles et conditions équivalents.

## Dépendances et portée

Deux avis Rust exigent WIKILINKS, désactivé dans l’unique parseur. Deux avis
Guava concernent les classpaths de build/tests, hors APK. Huit entrées Go
concernent Caddy distinct inchangé/non livré dans241 : leur inclusion dans
ce candidat est réfutée ; leur exposition sur Caddy reste ouverte. Aucune
acceptation de risque ni correction de Caddy inventée. Contre-revue44471d90
terminée/lue/ACK, frontière vérifiée par diff vide deploy/caddy-security.

## Limites conservées

Cause globale historiqueJSONL encore inconnue ; couplage à une vue lente
reproduit/corrigé et instrumentation ajoutée. Compréhension limitée des commandes
connues, programmes arbitraires non certifiés. Pas de recette UI ni restauration
après sinistre déclarée. Historique Git borné au delta. Gros modules historiques
non refondus. Les preuves brutes et anciennes tentatives sont conservées hors Git.
