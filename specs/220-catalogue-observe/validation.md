# Validation220 — livraison active

Source initiale relue : 06dfbad1bad533e4c6f1e68861d2f64effe48bef. État historique avant la recette et la livraison consignées plus bas.

- Compilation daemon : PASS.
- Sept tests daemon220 : PASS sur le mécanisme bootstrap réel avec fournisseur simulé, refus compte changé/liste excessive, absence de compte, délai RPC, génération, chemins privés, époque/digest/TTL/absence de mutation, autorité de lecture, writer occupé/saturé. Sept tests repassés sur cache sans symboles de debug (0,56Gio au lieu de 20,59Gio avant entretien).
- Deux tests purs transport220 initiaux : PASS ; reprise finale et régressions en cours.
- Guides : PASS, 32 pages, 522 liens, 13 paires.
- Diff source exporté pour revue sans shell : 131517 octets, SHA256 8d238926b34d1143470ca6112d80d1b12753e4aade7ba2d7f15dd838ed3fd226.
- Revue1 520cb663-bd8d-4664-855f-fd44f92eabcc : aucun verdict, shell absent et diff non fourni. Résultat lu, ACK et annulation propriétaire demandés. Une seconde ronde reçoit le diff exact par fichier, sans changer les droits.

La cause historique de chaque ancien refus n’est pas reconstruite par ces tests. Aucune mission métier, connexion ou abonnement n’est modifié.

## Revue et lecture réelle du protocole

Ronde2 884661dd-cc00-4cd6-97f2-e3116ab02a1a : patch complet lu, aucun P1 confirmé ; réserves P2 prises en compte. Les entrées retirées de la politique sont purgées du cache. Une émission du refresh non confirmée reste unconfirmed. Les types de compte changés sont refusés, les clés API sans identité restent non vérifiables. L’objectif120s est séquentiel et sans garantie sous charge ; le TTL180s ne présente aucune ancienne liste comme actuelle. Une sonde dont l’arrêt reste non confirmé conserve son besoin de maintenance au-delà du TTL. Les erreurs de stockage de la preuve restent distinctes d’un processus connu non arrêté.

Le rafraîchissement de métadonnées est volontairement disponible aux agents attestés comme état propre, et approuvé par la surface native correspondante. La négociation Client concerne le vocabulaire, pas un droit humain. Aucun tour, login, compte, modèle ou profil n’est changé.

La première recette contre le Codex0.163.0 installé sur Cartae a échoué à initialize avec SourceUnavailable. Une instrumentation ne donnant que les noms d’étape a confirmé READY → marqueur → RELEASE → socket connecté → initialize, puis fermeture. Le code officiel de cette version établit que le socket Unix attend une négociation WebSocket, pas un flux JSONL : https://github.com/openai/codex/blob/rust-v0.163.0/codex-rs/app-server-transport/src/transport/unix_socket.rs . Le lecteur et son faux serveur sont corrigés pour ce transport. La bibliothèque WebSocket déjà verrouillée est réutilisée, sans dépendance nouvelle, sans portTCP, avec échéance globale et volumes bornés. UID du pair vérifié. Le rendez-vous Codex peut être un lien symbolique : seul le lien privé est retiré, sans suivre ni supprimer sa cible. La recette réelle reste à repasser avant activation.

La recette réelle après correction WebSocket a réussi : connexion codex-cartae-normal, 3 modèles annoncés, identité de compte observable, sonde arrêtée, aucun thread/turn. Reçu privé : /home/moi/.cache/bridget220/metadata-candidate.json. Ce témoin qualifie le protocole du programme installé, pas un accès à tous les modèles ni la fraîcheur de son cache réseau. Les 7 tests220 passent avec le serveur simulé WebSocket ; la recette réelle est volontairement ignorée dans la suite courante et requiert une autorisation explicite. Le transport garde une échéance absolue couvrant handshake et fragments, un compteur de tous les octets reçus, des bornes par trame/message et une limite globale.

Régressions : 2 tests217, 4 scopes MCP, 1 catalogue MCP, 10 observer de permissions, 35 daemon168, 2 transport220 et 9 transport168 PASS. Le test de liste exacte a aussi été actualisé pour deux outils déjà présents avant220 (ask_user et réparation workspace), sans changement de leur comportement.

Complément de corrélation : une définition distante divergente laisse maintenant un diagnostic fermé sans aucun modèle ni nonce, au lieu d’un silence not_observed. Le test vérifie aussi la réparation par une observation suivante conforme. Sept tests220 repassent après ce complément. Le diff de 3739 octets (SHA256 82637565f3df9dc5285742db1b24e1390933e5f6a78e3682495275f5c57f94f2) a été transmis au relecteur de la ronde3 ; remise in_flight, lecture non déduite.

Ronde3 84ed2243-f8b2-4e03-876f-e6d5e45f945d : aucun P1, deux P2 confirmés. Correction de la conservation du diagnostic StopUnconfirmed dans les snapshots après activate/reconnexion, avec token de génération dans la déduplication d’envoi. Le maître conserve ce diagnostic au refresh. Test ajouté avec un vrai sous-processus simulé qui ignore TERM, reste détenu, puis sort uniquement après le signal de fichier privé du test ; aucun SIGKILL et une seule initialisation. Machine désactivée séparée de la source absente, test dédié dans la projection. Lecture du registre déplacée après les contrôles d’identité/époque/capacité. Sous contre-pression sans octet écrit, la publication est différée sans fermer le canal ; la fermeture après trame partielle reste documentée et testée.

Décompte : sept tests automatiques initiaux220 et une recette réelle explicitement autorisée, exécutée séparément de la suite ignorée ; les répétitions ne sont pas additionnées. Un huitième test automatique couvre le processus retenu. Les 63 régressions sont distinctes de ces cas.

Ronde4 cdb165e9-4fc2-411e-a03b-3e4aed6b9583 : deux derniers P2 de portée. La projection publie maintenant la contrainte d’arrêt à côté du statut machine/connexion, même si un statut précoce est prioritaire. Une définition divergente ne remplace plus cette contrainte connue par une cause moins précise. Le test passe par le vrai handler Hello : cache ancien connu, nouvel epoch/cache vide, statut not_observed transitoire, puis republication du worker et contrainte rétablie ; une machine désactivée garde cette contrainte séparée. La documentation nomme cette fenêtre volatile et ne promet plus une visibilité continue. Succès CLI JSON et erreurs JSON sur --json sont distingués. Neuf tests automatiques220 PASS après ce complément.

Retour opérationnel : le programme217 est conservé comme retour. Avant tout retour d’un worker220 vers217, constater la fin de ses sondes de métadonnées et qualifier leurs marqueurs privés. Un ancien worker ne sait pas reprendre leur supervision. Aucune suppression de marqueur ni preuve d’arrêt ne doit être fabriquée. Ce travail n’active aucun retour automatique.


## Livraison active — 30 octobre 2026

Source fonctionnelle : f5a4949fe06b258de0683031f019a75ce04a990d. Build220-catalogue-f5a4949f, version commerciale0.6.5. Ronde5 f22a202a-8596-4465-a124-d73e0fd2d058 : les deux P2 précédents sont levés ; aucun nouveau P1/P2 concret dans le complément. Les six tâches de conception/revue sont result_available, cleanup_done=true, guarded=false ; guard_released reste false et n'est pas réinterprété.

Contrôles distincts : 9 tests automatiques220, une recette réelle de métadonnées séparée, 63 régressions, soit73. Les répétitions ne sont pas additionnées. Guides32pages/522liens/13paires et git diff --check PASS. Aucun nouveau build, test ou essai métier n'est nécessaire à la clôture documentaire.

Les exécutables ont passé le contrôle local de compatibilité et le validateur installé217 avant bascule : Cartae2clients, SYS2, KS1, Mac4. Linux exige au maximum GLIBC2.34.

| Composant réellement chargé | PID constaté | Build |
| --- | --- | --- |
| Maître Cartae | 1421623 | 220-catalogue-f5a4949f |
| Mobile Cartae | 1421640 | 220-catalogue-f5a4949f |
| Worker Cartae | 1425319 | 220-catalogue-f5a4949f |
| Worker SYS | 498522 | 220-catalogue-f5a4949f |
| Worker KS | 914244 | 220-catalogue-f5a4949f |
| Worker Mac | 89015 | 220-catalogue-f5a4949f |
| Pont T3 Mac | 89882 | 220-catalogue-f5a4949f |
| Updater Mac | 90107 | 220-catalogue-f5a4949f |

SHA256 Linux : 157ca7d44916e61ff8f502eb26f65036ee2644e2574668594f389d5a9fe5a4b8.
SHA256 Mac : fa0cccf075f38a6307cba10ec513ed0c46623617b9575ddbc0a70446cfc2186d.
Installations : /home/moi/.local/lib/bridget-communication/220-catalogue-f5a4949f/bin/bridget et /Users/moi/.local/lib/bridget-communication/220-catalogue-f5a4949f/bin/bridget. Les images chargées et leurs empreintes ont été contrôlées. Reçu privé : /Users/moi/.cache/bridget220/activation-verified.json.

Maintenance native ciblée : sauvegarde cohérente SQLite/configuration, arrêt propre des seules unités Bridget connues, coupure maître2,29s. Les wrappers observés ont été préservés. Aucun service Opus2D/NE02/Roadie, compte, profil ou tâche métier n'a été relancé. Reçus privés : /home/moi/.local/share/bridget-maintenance/220-production-20261030T092005Z/receipt.json et worker-loaded.json sous /home/moi/.cache/bridget220 sur chaque serveur ; /Users/moi/.cache/bridget220/mac-loaded.json sur Mac.

## Preuve native après activation

Une bridget_session et une bridget_capabilities ont réussi. Le catalogue expose45options déclarées et15observations de connexions. Le CLI négocié --observed a rendu les données220. Un seul refresh explicite SYS a rendu requested ; ce retour n'a pas été présenté comme terminé. Une relecture bornée a ensuite constaté de nouvelles dates sur les deux connexions SYS.

- Cartae : codex-cartae-normal et codex-cartae-terra213, reported_models, 3modèles chacune.
- SYS : codex-sysle2 et codex-sysle2-terra213, reported_models, 8modèles chacune.
- KS : codex-ksstore reste disabled ; aucune connexion personnelle ni réactivation automatique n'a été effectuée.
- Claude : unsupported_provider décrit seulement la découverte automatique non implémentée. Ce statut ne refuse pas les lancements Claude déclarés.
- Mac : Codex not_observed. Le registre origine177 porte la machine local, alors que le worker est monordinateur. Ce contexte local ancien n'est pas modifié dans cette livraison limitée aux serveurs provisionnés ; aucune observation automatique Mac n'est qualifiée.

Les modèles annoncés ne prouvent ni l'accès effectif à chacun, ni la fraîcheur réseau du cache fournisseur. La politique d'admission reste séparée et inchangée. Le renouvellement vise120s par connexion, séquentiellement ; une donnée expire publiquement après180s. Les changements externes non observables restent explicitement inconnus.

Reçus privés sans publication des identifiants de compte : /Users/moi/.cache/bridget220/catalogue-active.json, /Users/moi/.cache/bridget220/refresh-sys.json et /Users/moi/.cache/bridget220/catalogue-refreshed-sys.json.

## Ancien connecteur de ce fil

Le processus MCP58296, parent61494, charge encore /Users/moi/.local/lib/bridget-communication/214-clients-30581503/bin/bridget. Une reprise native ne le remplace pas. Il lit déjà les champs220 de capabilities, mais n'expose pas le nouvel outil bridget_refresh_models. Aucun port de rechargement MCP n'est disponible dans ce fil. Ne pas boucler sur les reprises et ne pas tuer le fournisseur courant.

Geste humain : fermer puis rouvrir complètement l'application T3 Code, reprendre le même fil, puis faire une bridget_session. Après cela, contrôler une fois le processus MCP et capabilities. Les services220 ne nécessitent pas une seconde bascule. Les anciennes missions ne doivent pas être dupliquées.

La comparaison219 et ses autres écarts restent distincts. La prévention du drainage218 reste ouverte. Le retour217 exige toujours une preuve de fin des sondes220 avant bascule ; aucun retour automatique n'est activé.


## Preuves, entretien et reprise

Le bilan HTML a été prévisualisé à728px et360px, sans message console ni débordement constaté. Preuves : diaporama vérifié. Fichier : /Users/moi/Nextcloud/10.Scripts/64.bridget/preuves-corrections/2026-10-30-220-catalogue/preuves.html. Il présente des constats natifs, sans capture d'interface fabriquée ni donnée de compte.

La commande installed a terminé sur Mac/Cartae/SYS après vérification des installations : caches restants4,84/0,65/0,98Gio. Trois candidats jamais activés c482 ont été retirés sur Cartae/KS/Mac après contrôle d'usage ; deux copies de transfert finales Mac identiques aux exécutables installés ont été retirées. Actif220 et retour217 conservés. MCP214 et wrapper198 sous usage restent conservés. Le candidat c482 SYS et les six worktrees de revue220 restent conservés : Git propre, aucun fichier ignoré, tâches closes, mais plusieurs processus du même compte ne sont pas inspectables via /proc et sudo sans mot de passe est refusé. L'absence d'usage ne peut pas être attestée complètement ; aucun retrait forcé ni arrêt de processus tiers. Audit privé : /Users/moi/.cache/bridget220/worktree-audit.json ; retraits : /Users/moi/.cache/bridget220/cleanup-receipt.json.

Les quatre responsables concernés ont été informés après activation : monuments, Politique sol, trajets bus et Roadie. Messages natifs mcp-58296-6ae4674a-12, -6ae4674c-13, -6ae4674f-14 et -6ae46751-15 acceptés in_flight. Leur lecture n'est pas déduite. Aucun accusé ni surveillance périodique demandé.

Le commit documentaire final est intégré à main puis poussé. Les références main des miroirs provisionnés SYS/KS sont synchronisées ; leurs HEAD de préparation détachés sont préservés, car ils ne sont pas des checkouts main à déplacer arbitrairement. Les références main sont vérifiées égales au main distant. Le reçu final privé porte les SHA et les contrôles de propreté.
