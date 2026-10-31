# Complément 221 — dates, efforts et garde du catalogue

État : complément serveur livré et lu par le port natif. Programme global In Progress. T181 vérifiée ; client MCP214 ancien de ce fil traité séparément par T177. C01–C07, T055, C19 et T177 ne sont pas soldés par ce complément.

## Résultat
Le catalogue expose séparément date de lecture et date de réception, âges et expiration locale estimée. Les écarts entre efforts configurés et annoncés restent des observations ; ils ne donnent aucun droit et ne changent aucune admission. Le contexte machine/connexion/compte est déjà porté par l'objet parent ; aucune répétition de ces identifiants par modèle.
Le signalement d'arrêt incertain survit aux erreurs du registre, à une politique désactivée et au retrait/rétablissement d'une connexion. Seule la preuve d'arrêt du groupe ou de récupération du marqueur libère la garde. Le cache reste borné à32 clés. Le conseil de définition divergente nomme la vérification requise. Une vue globale indisponible demande de vérifier les marqueurs avant toute bascule.

## Contrôles source
Commande via scripts/build.py : cargo test --workspace --lib spec22 -- --nocapture.
R9 :34 tests daemon et2 transport réussis,3 recettes privées ignorées. Les petits sous-processus internes ne sont pas additionnés comme tests indépendants. Log privé /home/moi/.cache/bridget221/l2a-review-fixes-r9.log.
Cas propres : horloges source/réception, recul de l'horloge locale, date future, observation ancienne reçue récemment ; efforts annoncés/configurés ; modèle identique sur deux serveurs ; aucune mutation des tâches/catalogue déclaré ;128connexions x256modèles, trame complète13047160octets sous16Mio ; registre invalide/off, retrait/réintégration, définition changée,33connexions dont32inactives, saturation du cache des gardes.
Le test220 existant utilise un vrai processus qui ignore TERM : garde conservée lors de refresh/reconnexion, arrêt constaté après sortie autorisée, puis libération explicite. Aucun SIGKILL ni nouveau privilège.
Guides :32pages/522liens/13pairesFR-ENPASS ; git diff --checkPASS. Deux tests catalogue212 réussis en complément, soit38testsdistincts sur la source finale.
Échecs conservés :R3 de mesure utilisait une borne inventée de demi-trame, remplacée par le vrai plafond ;R5 perdait les mots « marqueurs privés » exigés par un test existant, conseil corrigé sans affaiblir l'assertion. Aucun de ces échecs n'est caché ou compté comme réussi.

## Contre-revues
Réemploi :2ceacc8c. R1 :2824b7b8 ; R2 :f9611342 ; R3 :2272ec55. Lectures seules d'extraits, pas qualification runtime indépendante.
Les défauts confirmés ont été corrigés : perte de garde sur registre invalide ou retrait, conseil définition insuffisant, guard pouvant être remplacé par un résultat sans preuve d'arrêt, absence de conseil global quand la politique est fermée. La suggestion de libérer sur Observed n'a pas été suivie : seul le groupe récolté ou les marqueurs récupérés font preuve.
Hypothèses non retenues après lecture : limite1Mio par cycle (elle est par instance de Rpc/MetadataSocket d'une sonde) ;256modèles déclarés par connexion au maximum global (la politique borne le total à128). Le modèle est déjà corrélé au compte/serveur par l'objet connexion. Aucun rejet de dossiers Codex volontairement partagés.
Résultats lus, ACK et clôture demandés une seule fois. Worktrees conservés tant que les usages ne sont pas tous vérifiés.

## Analyze et Converge
Analyse manuelle comparant spec/plan/tasks/code/tests puis relecture ; primitive projet absente déjà constatée. Aucun CRITICAL retenu dans ce diff. Réemploi220, sans nouveau store ni autorité. Converge source du complément : aucune catégorie supprimée ; écarts de livraison encore ouverts dans T181. Aucun succès runtime annoncé ici.

## Limites
Les adaptateurs de découverte autres que Codex et le parcours complet multi-comptes ne sont pas qualifiés. La liste d'un programme ne prouve ni la fraîcheur de son cache fournisseur ni l'accès au modèle. Un login externe sans signal non secret attend un contrôle périodique/explicite. La cible120s n'est pas garantie.
Correction technique dans les données natives, pas modification d'une interface graphique. Pas de diaporama de clics fabriqué. Les preuves d'activation et la restitution native seront ajoutées après la bascule. Aucun modèle ou mission métier relancé.


## Audit source final
V15 /home/moi/.cache/bridget221/audit-l2a-r3 : finalize pass/release_blockedfalse, validate true,12/12contrôles applicables vérifiés. Semgrep pack figé et Gitleaks fichiers exécutés, zéro candidat. Historique et CVE globaux exclus car aucun commit historique ni dépendance modifié. R1/R2 conservées sans les présenter comme l'audit de la source finale.
Grille42critères :35notés/35applicables ;7hors ce périmètre source. Sécurité75(3/3), sécuritéIA75(3/3), fiabilité75(1/1), tests69(4/4), ressources69(4/4), complexité75(3/3), qualité67(3/3), architecture75(3/3), responsabilité75(3/3), minimalisme75(3/3), métier75(3/3), traçabilité75(2/2). Les points démontrés égalent ces notes sur le périmètre examiné. Détail des sous-contrôles, preuves et limites : /home/moi/.cache/bridget221/audit-l2a-r3/axis-scoring.md.
Les notes ne sont pas une mesure de tout Bridget : pas de benchmark de latence, de qualification graphique ou de runtime encore chargé. La décision source permet de poursuivre la livraison ; elle ne prouve pas son activation.


## Livraison et résultat réellement chargé
Source exécutable66a3ba74393ae97e6587d2a48740e37acbdc2d1f intégrée àmain et poussée sur GitHub. Build221-catalogue-66a3ba74, version commerciale0.6.5 inchangée. MacARM64 SHA256655f3020e7f97e575c9e5a51acf80f22a05a2fa0cefd22c412d09e9b9716cdf4,32992832octets. Linux SHA2562b9ffbb5d33d1be56e2f9c3fe440e11b578a5e8af7a4ae40f773aa86c322931c,39997224octets, GLIBCmax2.34 vérifiée.
Compatibilité des programmes configurés vérifiée avant bascule et validée par l'ancien binaire :2surCartae,2surSYS,1surKS,4surMac. Aucun programme, compte ou modèle changé.

Images vérifiées par chemin réel etSHA256 : maîtreCartae294363, workerCartae295246, mobile294384, workerSYS817187, workerKS1216780. Macworker9646 et pont9765 vérifiés par mappinglsof ; watcher relancé10230. Les processus peuvent changer ensuite : ces PID sont des observations datées dans les reçus privés, pas des identifiants permanents.
Coupure maître2,34s ; sauvegardesSQLitecohérentes et config avant/après arrêt. WrapperCartae2500061/birth270585460 conservé ; aucune mission rejouée, aucun service métier touché. Actif et retour221-stop-guide-342754e6 conservés. Pointeur updaterCartae/Mac enregistré via atomic_json natif sous verrou et sauvegardé.
Premier essai Mac refusé par le verrou déjà détenu du watcher, avant modification du lien. Deuxième parcours : unité watcher retirée nativement, contrôle d'arrêt, verrou, écriture puis réinscription en finally. Aucun arrêt forcé. L'extraction SYS a été adaptée à son Python ancien après lecture des membres réels ; seul lien documentaire interne autorisé, aucune extraction arbitraire.

Port natif CLI chargé :15connexions, nouveaux champs présents. Après stabilisation, CodexCartae annonce3modèles par connexion et SYS8 ; aucune contrainte StopUnconfirmed. KS reste désactivé selon la décision existante, sans reconnexion ni réactivation. MacCodex reste not_observed : aucune observation inventée.
Le clientMCP de CEfil reste214/PID58296, confirmé par mappinglsof. Une seule réattestation a réussi, puis capabilities a refusé t3_session_unavailable/binding_registration_refused/credential_mismatch. Aucun nouvel essai en boucle, aucun remplacement de clé ou copie de secret. Aucun port de rechargement MCP n'est exposé. Action humaine précise : quitter complètement T3Code parCmd+Q puis rouvrir le même fil ; une simple reprise ou réattestation ne recharge pas le client. T177 reste ouverte. Les services sont chargés ; tous les clients MCP ne le sont pas, et le lien appelant n'est pas annoncé rétabli.

## Git, caches et information
MainCartae etMac alignés sur la source ; main des dépôts et miroirsSYS/KS avancé par fetch sansforce, HEADpréparatoires4d89cfab/16304623 conservés et propres. Preuve mirror.json sur chaque serveur.
Entretien build.py avant/après et installed après installation vérifiée : cacheSYS0,57Gio, Mac5,36Gio logiques. Pas de cache de test par scénario. Caches et sources de C19 en cours conservés. CopiesLinux, archives et anciennes images encore sous usage ou observation incomplète conservées, sans retrait forcé ; les worktrees de revue restent gardés car leur clôture/usages ne sont plus tous relisibles depuis le client214. Aucun privilège de nettoyage ajouté.
Quatre responsables prévenus parBridget avant la coupure, reçus conservés. Avis après activation remis dans leurs fils natifs existants, car MONclientBridget refuse credential_mismatch : monuments, Politique sol et Roadie ont reçu une nouvelle information ; trajetsbus est mis en file sans interruption. Aucune nouvelle mission métier, remplacement, bascule de moteur ou réponse demandée.
Preuves privées : /home/moi/.cache/bridget221-catalogue/ surCartae/SYS/KS ; /Users/moi/.cache/bridget221-catalogue/ surMac. Reçus core-loaded, worker-loaded, images-loaded, catalogue-settled, mac-loaded, coordination. La remise d'un message ne prouve pas sa lecture.

ConvergeT181 : CONVERGED sur ce complément, zéro fonction hors périmètre ajoutée ; T181 est une extension de convergence distincte, pas la clôture deL2a.34/181tâches vérifiées ; aucune égalité globale ni50/50revendiquée. Prochaine tâche : T174/C19, puis l'ensemble des lots restants.
