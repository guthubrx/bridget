# Reçu183 — interface mobile — 25octobre2026

Implémentation : **Implemented,22/22tâches**. Pipeline : **In Progress**.
L’activation Linux est retenue. Aucune installation complète de la fédération
n’est revendiquée. État constaté vers23:03CET.

## Résultat et périmètre

Le cadrage approuvé est conservé : naviguer vite entre agents, savoir lesquels
requièrent une action, garder chaque brouillon et position, lire/rechercher/copier,
préparer fichiers/photos, dicter avant envoi et manipuler les pages HTML isolées.
Le volet réutilise l’annuaire natif, avec Tous/Favoris/recherche/icônes et états
explicites. Le compactage repose sur des événements structurés corrélés.
Aucun produit tiers ne définit ces parcours.

Phases0à7 exécutées : cadrage, specify, plan, audit de réutilisation, tâches,
AnalyzeP1/P2, implémentation, convergence et audit. Primitives SpecKit absentes :
protocoles manuels documentés dans analysis.md, et non présentés comme commandes
réussies. Checklist8/8 ; reuse-auditPASS12/12. Convergence **CONVERGED en2passages**,
le second après suppression de13lignes de production inactives. Aucune case
ouverte. Phase8 : Git et publication terminés, activation Linux bloquée.
Phase9 : candidats examinés, retraits vérifiés et conservations ci-dessous.

Créations de production : trois icônes vectorielles et ADR183. Deux classes de
recette Android ajoutées. Annuaire, avatars, saisie, stockage chiffré, CommonMark,
client mobile, journal et projections existants étendus. Aucune dépendance,
table, route ou couche serveur parallèle ajoutée. Arbitrages : reuse-audit.md.

## Git et publication

- Base214a33ecc3de1fafafc623f61477fc5b87c8cc8a.
- Source529d455a08f1bdec7ac7331cb3c819af04148929, branche session-183-interface-mobile.
- Intégration1d122d31abfdb8d9092be10980c1277903fb1e96.
- Correctif du faux positif historique : d1a9ce00, intégré dans
  **e2987fb7175f511d22766260370a918f49fd2cfb**. Exception limitée à une empreinte
  immuable, justification bilingue ; aucune règle du scanner désactivée.
- Corrections concurrentes706a70ab/9709ab20 conservées et contrôlées.
  Les reçus OVH ultérieurs ne changent aucun code de ce paquet.
- main et tagv0.6.3 poussés sans force vers https://github.com/guthubrx/bridget.git.
  SHA distant comparé à la cible. Aucune PR créée.
- CI verte40237248192 sur e2987fb7, puis40237620577 sur76acf676.
- Version **0.6.3**, Android6003. Publication réelle :
  https://github.com/guthubrx/bridget/releases/tag/v0.6.3 . Six fichiers téléchargés
  depuis la publication et comparés intégralement aux octets locaux.

Le tag et les binaires portent e2987fb7. Les commits de clôture documentaire
ultérieurs ne sont pas présentés comme une reconstruction des exécutables.

| Paquet | SHA256 |
|---|---|
| macOS ARM64, archive |8d3f5bb859717505b158786ee94fffe89f9b8d300826d033e07cae91c4d9c2e7|
| Linux x86_64, archive |e388ec81410784b89c11e7d4b803c5fbd93c034c93cf55e6a4a22071aa4406df|
| Android APK |8a43815746e20654c9a0ab1a20b82cd0fcf9c8e8dc83d4fdd4434e226835378a|

Binaire Mac : e3136509cbc3f6dfefe123cfe1e793d988a849fe00d0d68caa9b74f012eebeef.
Binaire Linux : f383f57b56b97db9febc890e14ebbac53ab4ea768d179b9a57fbcbb7ddbf1a9b.
Signature APK vérifiée contre0.6.2 : même certificat,
0c43c621254d9a6e0dbe148466f453f1aaba1a62bf23bd7332c61086f16a6588.
macOS : signature locale vérifiée. Linux publié requiert glibc2.39minimum ;
ne pas le substituer au worker Debian construit séparément.

## Tests et audit

- JVM30PASS,0échec/erreur.
- Parcours complet Android1PASS51,587s : dix alternances, brouillons, copie exacte,
  recherche, résultat de dictée contrôlé et retour d’aperçu.
- Régression Android20PASS10,809s, dont15refus de capacités du WebView réel.
- Rust31PASS ciblés : compactage4, annuaire2, file6, questions19.
- Intégration des sources concurrentes : préparation35PASS4,438s,
  permissions2PASS. Installateur26PASS4,427s.
- Contrôle historique complet sans détection après classement du faux positif.
  Guides :32pages,522liens locaux et13paires de langues valides.
- Builds finales Mac1min54s, Linux1min28s ; APKrelease recompilé et signé.
- AuditV15fix : décisionpass, release_blocked=false,14/14contrôles vérifiés,
 0findingactif,1LOWcorrigé. Manifestec709ee67b99062317ec20a33626fa264ccb542c6d79ee72ac330232a266f33d5.
  Audit figé de la feature529d455a ; intégrations et CI ultérieures vérifiées
  séparément. Pas de validation sans dérive prétendue sur ces commits ultérieurs.
- Semgrep et Gitleaks :0candidat dans leurs périmètres déclarés. OSV : trois
  vrais lockfiles analysés,0avis détecté. Échec initial de découverte OSV
  conservé ; la passe explicite réussie ne l’efface pas.
- Une compilation sous contrainte d’audit a atteint process_limit8. Cette passe
  reste un échec. Le contrôle mono-job réussi et les builds complètes hors de
  cette contrainte sont des preuves distinctes.

Deux contre-revues indépendantes ont été demandées, plan puis code. Elles ont
échoué avec api_error. Aucun verdict indépendant n’a été obtenu. Tâches
30de68a4-0a0a-4b51-8387-7a55a6c6b70b et2a13918a-b05b-46fe-8194-90e404a0c28b.
Les échecs ont été lus et acquittés ; aucune relance en boucle.

Les notes de qualité calculées ne sont pas la note de passage des14contrôles.
Aucun niveau maximal ou score global de qualité100n’est revendiqué.

| Axe | Note calculée |
|---|---|
| Sécurité | 70/100 |
| Sécurité IA | 75/100 |
| Fiabilité et exploitation | 58/100 |
| Tests et validation | 65/100 |
| Performance et ressources | 63/100 |
| Complexité algorithmique | 67/100 |
| Qualité du code et des erreurs | 75/100 |
| Architecture et dépendances | 69/100 |
| Responsabilité future et hygiène | 67/100 |
| Minimalisme et frugalité | 75/100 |
| Métier et fidélité des données | 75/100 |
| Traçabilité des exigences | 75/100 |

Réserves : aucune FULL, ni charge/heap approfondis ; voix réelle et Pixel non
qualifiés ; le compactage spontané d’un modèle n’a pas été observé. Le cycle
réel de journal contrôlé est vérifié. Aucun client desktop développé.

## Disque et runtime

Mac : /Users/moi/.local/bin/bridget pointe sur
/Users/moi/.local/lib/bridget-communication/183-e2987fb7175f/bin/bridget .
Service d’observation chargé depuis ce binaire, PID87075, santé vérifiée.
Retour vérifié : /Users/moi/.local/lib/bridget-communication/181-dd1410f43dc2/bin/bridget .
Le service de mises à jour a été rechargé pour conserver ce retour dans son état,
sans arrêter les modèles ni changer la politique automatiqueON.

Android : release6003 installée sur emulator-5582, association d’origine
restaurée par instrumentation avant l’installation, paquet de test retiré.
L’écran d’association d’origine reste ouvert ; aucun compte ni autorisation du
Pixel copié. Pixel absent, non mis à jour. WebViewBeta officiel157.0.8103.14
conservé pour passer le seuil de sécurité156.0.8078.48.

Linux : le nouveau binaire est copié et vérifié hors staging à
/home/moi/.local/lib/bridget-communication/183-e2987fb7175f/bin/bridget .
**Il n’est pas actif.** Core/mobile/workerCartae utilisent encore0.6.2 dans
/home/moi/.local/lib/bridget-communication/181-dd1410f43dc2/bin/bridget .
La maintenance contrôlée existante a refusé avant tout arrêt : admission active.
Aucun verrou de maintenance ajouté, aucun état SQL modifié pour passer le garde.

Admission retenue : bridget-ovh183-ksstore-qualification-20261025-02,
ownerbc833746-3f24-41cd-bc64-46d60f4c0769, chosen, task_id absent.
Le rattrapage courant ne lit que les époques précédentes ; ce reçu de l’époque
courante demeure en attente. Le responsable a été interrogé par Bridget,
requêtemcp-86512-6ade7a0f-1, sans réponse dans le délai. Il conserve sa tâche.
L’ancien e5959986-e426-422d-956f-e035dbcf89be reste également cancelling,
sans mission émise ; son état n’a pas été falsifié. Aucun nouveau travail créé
pour contourner l’une ou l’autre clé.

**Prochaine action exacte** : réconcilier le sort de la même admission avec son
responsable, vérifier qu’aucune admission/mission/commande/question n’est active,
puis reprendre la maintenance contrôlée, vérifier les PID/exécutables/santé,
finir son verrou et mettre à jour ce reçu. Ne pas forcer un arrêt pendant une
activité inconnue. Aucun redémarrage périodique ni attente automatique créé.

## Preuves visibles et nettoyage

**Preuves : diaporama vérifié**,30étapes,28images réelles uniques,
2344763octets ; SHA2565f09aa232298bea6e880ccc2d5807628c6be6f15b94e92362226c043ae206195.
Contrôles du diaporama à360/1000px, images décodées, navigation/zoom et console
sans erreur. Originaux et clics privés conservés.

/Users/moi/Nextcloud/10.Scripts/64.bridget/preuves-corrections/2026-10-25-183-interface-mobile/preuves.html

QuestionQAannulée par son propriétaire, enfantsMCPfermés, producteur/mobile/coreQA
arrêtés individuellement après identification. Seule la routeCaddy
bridget-mobile183-qa retirée avecIf-Match. Instance privée conservée comme preuve.
Deux fichiers synthétiques Android supprimés après comparaison des octets.

Worktrees/branches Linux retirés :179,180,181,182 et revue2a13918a.
Documents ignorés/audits179/181archivés, chaque fichier vérifié parSHA256 avant
retrait. Seuls caches de build et bytecode jetables accompagnent ces retraits.
Mac : worktrees mac-final-source et ancien mac-build-source181retirés après
lecture ciblée lsof+D sans fichier ouvert ni diagnostic. Trois APKMac supprimés,
37300622octets logiques ; deux binaires Mac obsolètes supprimés,61694112octets.
Aucun gain physique APFS n’est déduit de ces tailles.

Conservations explicites :

- Worktrees177/178 : tâches encore ouvertes dans le dernier suivi ; fichiers
  ignorés présents. Ne pas effacer un lot plus large pour fermer183.
- Worktree183-flotte-ovh et binaires183-706a70ab/183-b6c045cd : autre travail,
  workers qualifiés ou en attente, dont un binaire Mac encore chargé PID85257.
- Worktree183-interface-mobile : activationLinuxencore à finir ; conserver le
  checkout de poursuite et ses constructions tant que ce blocage persiste.
- Mac : ancien0.6.0 encore chargé PID22992 ;178 encore référencé par son
  LaunchAgent ;181 chargé par des clientsMCP et conservé comme retour vérifié.
- Linux :181 actif et180retour de cette version ;183 candidat de la présente
  livraison. Les binaires distants d’un autre lot ne sont pas supprimés.
- Cache source candidatMac108Mo, originaux, manifestes et journaux privés :
  preuves de construction et de recette conservées sous /Users/moi/.cache/bridget183 .
- Audits/reçus/archives : /home/moi/.cache/bridget183 . Aucune base supprimée.

installed appelé sur les deux plateformes après copie/digest hors staging.
Mac : budget2,70Gio au passage ; retrait ultérieur de son checkout final.
Linux : observation ponctuellement incomplète, nettoyage refusé sans forcer.
Les inventaires Linux ultérieurs sont complets et ont autorisé les retraits
nommés ; ils ne transforment pas ce refus en succès rétroactif.

## Temps et état final

ETAinitiale180–300min, départ vers19:55CET. Dernier recalibrage publié22:28 :
45–90min restants. Le chiffre spécifique aprèsTasks n’est pas reconstruit sans
son relevé d’origine. Temps au reçu : environ188min, borne initiale non dépassée.
Ce temps ne vaut pas durée de livraison complète : attente de réconciliation
concurrente non résolue. Aucune clôtureImplemented ne remplace une activation.

Première tâche fonctionnelle non cochée : **aucune**. Reste opérationnel :
activation Linux et contrôle/cleanup associé. Pipeline **In Progress**.

## Contrôle de clôture documentaire

main021a103c21fdbf5aff2654f24c99890a96a0df8e poussé et comparé au distant,
CI40238273752PASS. Les trois branches locales183de source/correctif/reçu sont
retirées après intégration ; aucune de ces branches n’existe au distant. Le
checkout serveur183reste détaché au tagv0.6.3pour la reprise d’activation.
Le serveurHTTPdepreuve privé est arrêté, PID67209libéré. L’émulateur reste ouvert.

Passage de nettoyage final :7,67Gio logiques d’intermédiaires Linux supprimés
par scripts/build.py,3,66Gio reconnus restants. Mac :2,22Gio restants. Les
refus ponctuels d’observation précédents restent tracés. Aucun gain physique
APFS revendiqué. Retours, preuves et tâches actives conservés.

## Complément de maintenance — 26 octobre 2026

Après autorisation explicite de l'utilisateur, core/mobile/worker Cartae ont été activés sur le binaire 0.6.3 `183-admission-7478e9e7`, source `7478e9e756dced0e0f85d9519229c279644d362c`, qui conserve la livraison mobile `e2987fb7`. Les exécutables chargés, le statut MCP et HTTP mobile 200 ont été vérifiés. L'admission OVH qui retenait cette livraison est maintenant refusée durablement sans tâche. Aucun état SQL n'a été forcé.

Mac et l'émulateur restent en 0.6.3 ; le Pixel est toujours absent. Auto-update actif, Mac et Cartae annoncés à jour. Le retour Cartae conservé est 0.6.2. Six copies obsolètes Cartae ont été retirées après observation complète, avec reçus archivés. La tâche historique `e5959986` reste non réconciliée : ce complément ne prouve pas qu'une future activation automatique serait immédiatement autorisée. Voir le [reçu technique](../../183-flotte-ovh/validation/admission-replay.md).

Le worktree détaché de livraison 183 a ensuite été retiré : code intégré, aucune tâche gardée, aucun fichier ouvert. Ses preuves ignorées (rapports, résultats et sorties) ont été archivées et vérifiées avant retrait des intermédiaires. Les anciens worktrees mobiles 177 et sécurité 178 ont été traités de la même façon. Cartae ne conserve qu'un checkout `main`.
