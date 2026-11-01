# Validation222 — In Progress
État du1novembre2026 : code développé, services0.7.0/r3 chargés et parcours Android vérifié. Le client MCP de CE fil reste à recharger ; aucun statut global terminé ou parité avec un autre produit n'est annoncé. Programme221 indépendant.

## Fonctionnement réellement observé
- Profil technique proposé puis approuvé et activé par la CLI humaine avec confirmation explicite.
- Identité9a426071-1f98-49a9-80a9-97e74ca6f3a2 réveillée par le cycle natif ; demande qual222-role-20261101, soumissionf65b6eb4-39a3-8546-a714-ec4b692f94cf. Résultat completed, remise acked, réponse BRIDGET222_ROLE_WAKE_OK.
- Capture automatique relue : mémoire17b57583-ae1b-405b-aef1-263434dea681, reported_result, projetbridget, source de messageb2123c65321e4 et digest. Pas de succès métier déduit du marqueur.
- Profil de test ensuite désactivé ; arrêt propriétaire a7b63879-e77f-4c71-a492-e41fba6afbc4 : Équipier arrêté proprement. Aucun ancien travail relancé.
- CLI sur namespace isolé : proposition, approbation, versions, export de définition seule et import sous nouvelle identité inactive. Daemon de qualification arrêté, preuve conservée dans le cache privé.
- Définition utile Mainteneur Bridget conservée dans le catalogue, version1 non approuvée/non activée. Exemple reproductible : examples/mainteneur-bridget.json. Aucune désignation officielle implicite.

## Android réel
APK0.7.0/versionCode7000 signé avec le certificat existant, installé par mise à jour sans suppression des données. Association QR native et confirmation HTTPS normales ; aucun jeton injecté dans le stockage de l'application.
Circuit observé : catalogue authentifié, création d'un profil inactif, lecture/approbation d'une version, activation durable surbridget avec mandat explicite et plafond absent, fiche, écriture sourcée, rappel, correction, historique des deux versions, oubli, désactivation et relecture après fermeture/réouverture.
L'activation seule ne lance aucun processus. L'appareil technique5ac7df9a-72f7-4578-b5a7-3397ef837ab9 a ensuite été révoqué par la commande native ; les autres appareils sont préservés. La caméra de l'émulateur est restaurée. Le seul paquet updatefixture de test est désinstallé ; app.cartae.bridget et ses données sont préservés.
Preuves : preuves-corrections/2026-11-01-222-specialistes/preuves.html,17étapes dont15captures. Parcours Android vérifié ; dernière étape du diaporama garde explicitement le client MCP non vérifié. Les gestes sont natifs ADB ; aucun isTrusted DOM inventé. Les captures/manifestes originaux restent horsGit.
Les états horsligne, attente, ancien pair, révocation et perte de réponse sont contrôlés par les tests ciblés ; ils ne sont pas tous déclarés reproduits manuellement sur l'émulateur.

## Défaut découvert pendant ce circuit
R2 refusait le catalogue mobile : le client négociait exactement MobileOperationsV1+SpecialistsV1 alors que la garde du handler exigeait seulement MobileOperationsV1.
R3 accepte exactement la paire pour Specialists, garde la liste historique pour les autres opérations et conserve rôle, UID, absence d'agent/connexion auxiliaire, authentification, révocation et portées.
Test du vrai handler spec222_mobile_negotiation_reaches_scope_and_keeps_legacy_guard : PASS ; ancien pair refusé, scope limité refusé, opération historique avec mauvaise paire refusée, vrai appareil Read global accepté, révocation refusée, aucune remise.
Le circuit Android refait après activationr3 confirme le correctif. Pas de changement de compte, droits ou délais.

## Contrôles utiles et audit
Groupe final source :13daemon222+1transport222+1autorité181 PASS. Ajout ciblé du raccord mobile :1PASS. Android :9tests de modèle et assembleRelease PASS. Builds Mac et Linux portable GLIBC2.36 PASS ; signature Mac vérifiée.
Les274sources Rust/manifests sont identiques entre autorité, construction Linux et construction Mac : empreinte5755d4cff0bf703eb0f4f82f941fd435cff02f99be6f330e981365f97f1283fa. Aucun test général de2733cas annoncé exécuté.
Audit V15 révision2 calculé/validé : pass, release_blocked=false,14/14, source_drift vide. Rapports dans validation/audit. Le score indicatif100 n'est PAS une qualité100 : douze axes50–75, limites de runtime et de sinistre distinctes.
Après le delta mobile, le protocole a exigé ses quatre scanners : Semgrep0, historique0,56signaux génériques et10avis dépendances. Les57signaux déjà triés ont les mêmes sources/empreintes ;9avis hérités Caddy/Gradle horsdelta restent non résolus et transmis au responsable221. Aucun faux certificat global de propreté.
Revue Claude R1/R2 partielle et revue finale du responsable explicitement independent=false. La mini-revue mobile a été refusée sans tâche (voir plus bas), pas remplacée par un autre agent.

## Images réellement chargées
Voir validation/runtime.json : sept services0.7.0/build222-specialists-r3, PID et SHA relus.
Linux :14fca9f0a89e93c296bfdab3fb410d2ad409fecdd89c749d9c965cb14b0f863f.
Mac :128a54e63102cffd899826187ea7351934a3f3b21f8fbfecac8ed9a806742778.
Retour distinct reader222-r2 compatible dans les deux sens et pointeurs natifs Cartae/Mac sous verrou/écrivain atomique. Le retour conserve la limite de négociation mobile antérieure àr3 ; il n'est pas présenté comme portant ce correctif.
Coupure maître2,31s sur le dernier passage ; wrapperCartae2500061 et cinq wrappers SYS observés conservés. Aucun service métier redémarré.
Après activation : une session native et une capabilities,45options/gel=false. Ces faits ne prouvent pas un nouveau client MCP de ce fil.

## Client du fil : blocage restant exact
Client PID38771 identifié en image221-worker-probe-lane-r2 ; nouvel outil absent de la table de ce runtime. Configuration et lien pointent vers222-specialists-r3.
Une seule recharge ciblée autorisée : TERM du seul38771 après identification, parent moteur76095 préservé ; ancien PID absent après3s. L'hôte rend Transport closed et ne recrée pas ce client. Pas de boucle ni arrêt du moteur.
Action humaine : quitter complètement T3 Code avec Cmd+Q, rouvrir l'application, reprendre CE MÊME FIL. La reprise doit lire /Users/moi/.cache/bridget222-roles-profils/reprise-finale.md, vérifier une fois le client, une bridget_session et une capabilities, puis une lecture bridget_specialists(version1,action=list) si le nouvel outil est réellement exposé. Ne refaire aucun code/build/revue/qualification ou ancienne mission.
Si le client ne revient pas, conserver les preuves et nommer le composant hôte ; aucun redémarrage en boucle ni voie de délégation alternative.

## Admissions de revue et refus conservés
- bridget222-mobile-negotiation-review-20261101-01 : not_recorded, expected_base_commit_mismatch ; mainMac avait avancé. Pas de tâche.
- Nouvelle -02, après ce refus prouvé : réponse MCP permission_source_unavailable/unknown, pas de troisième demande. Lecture propriétaire SQLite en mode readonly du registre exact à la fin : state=refused, task_id=null, refusal_code=permission_not_inherited, owner du présent fil. Aucun Start ni copie d'état écrite à la main.
- Signalements au mainteneur221 remis in_flight, aucune lecture annoncée. Le dernier constat ne peut plus être envoyé après Transport closed ; conserver la note pour le client rétabli.
Les erreurs locales de syntaxe/TTY du responsable sont conservées dans les sorties de qualification ; pas de panne générale ou de révocation déduite.

## Git, publication et nettoyage
En cours de clôture documentaire : commit/intégration/push et publication0.7.0 à enregistrer dans validation/git-delivery.json.
build.py installed exécuté Cartae etSYS : pas de purge forcée, caches3,03Gio/0,60Gio conservés. Mac refuse le staging non enregistré : cache conservé, aucune suppression manuelle de garde.
Actif et retour, sauvegardes de maintenance, preuves et travaux d'autrui restent protégés. Les candidats222 éligibles sont traités par chemins exacts et leurs conservations justifiées dans validation/cleanup.md.
L'intégration du code validé suit la consigne utilisateur permanente de committer/pousser le travail validé et de poursuivre les actions sûres malgré un composant hôte bloqué. Elle ne change ni le gate de ce client ni le statut In Progress en Livré.
