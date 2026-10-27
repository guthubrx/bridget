# Validation — 27 octobre 2026

## Commande métier existante
Tâche f0b0a469-9c74-4748-bc3e-061df740141a, clé opus2d-20261027-ks-finition-lot-0432. Le parent conserve son contrôle. Aucun rejeu, remplacement ou arrêt demandé par le correctif.
Le journal de l'enfant 26dee445-1d46-49a1-a0e7-54b53f20fef2 confirme la remise métier : séquences 485 à 488, 04:42:36–04:42:40 UTC, lectures réelles des images attendues sur KS-STOR.

## Tests ciblés
Tous les appels Cargo utilisent scripts/build.py, avec le cache du checkout principal Cartae.
- spec184_late_release_reply_preserves_worker_connection : ROUGE avant le changement de production ; VERT après. La version finale fait expirer une vraie attente RPC, puis livre l'accusé tardif. Cas actif et annulé, état durable inchangé ; autre instance et ancienne époque refusées.
- spec184_slow_release_confirmation_keeps_existing_task : VERT ; réponse Release après six secondes, même tâche, aucun doublon.
- spec172_untracked_worker_refusal_keeps_known_command_channel : VERT.
- worker172_release_apres_attestation_lente_garde_revision_et_expiration : VERT.
- spec174_remote_release_uncertain_message_preserves_task_and_replay : VERT.
Dernier passage spec184 : 2 PASS, 6,13 s. Aucune suite globale ni fournisseur réel lancé par les tests.

## Revue indépendante
Bridget : requête bridget-spec184-review-20261027-01, tâche 24f07b2d-3174-4d45-936c-342f3abca2c5, source de base 9d7184b1589a4e6824d9ad46dbafdd117c0655ea, diff intégral fourni.
Aucun défaut bloquant ni affaiblissement des droits détecté.
Deux remarques traitées après lecture : test du vrai délai RPC avec état annulé conservé ; une même durée locale pour Start et Release.
Un accusé valide d'une commande annulée est ignoré sans réactivation ; le rejeter fermerait inutilement le canal des autres commandes.

## Portée
Le permis et son expiration restent inchangés. Un dépassement de trente secondes conserve une issue incertaine et la réconciliation existante.
Correction technique : aucun diaporama d'interface requis.
Nettoyage automatique appelé avant/après Cargo ; suppression différée par le garde d'observation des processus incomplet. Aucun nettoyage forcé.

## Livraison
Correctif source 1caabb9ecf7f25b68b295286e8c535ba0cdd0ae8 intégré à main et poussé.
Compilation release optimisée réussie en 1 min 08 s, via scripts/build.py.
Candidat installé et vérifié :
/home/moi/.local/lib/bridget-communication/184-1caabb9ecf7f/bin/bridget
Build : 184-confirmation-1caabb9ecf7f ; version : 0.6.3.
SHA-256 : 7344d44652f33e997ce0d44ade2cfd7c7a02ea2e1eece86f0ef602b721e9c75e.

L'appel natif updates prepare a été réellement exécuté. Refus : update_missions_active_or_unconfirmed. Aucune barrière contournée, aucun lien actif modifié, aucun service redémarré.
Le daemon reste actif sur 183-admission-7478e9e7. Core, mobile, worker Cartae et liaison KS-STOR relus actifs après ce refus.
Le correctif n'est donc PAS actif. La livraison runtime reste en attente d'une maintenance native admise.

L'appel installed a été exécuté après vérification du candidat hors cache ; il a refusé le nettoyage pour observation des processus incomplète. Les caches ne sont pas supprimés de force.
Le worktree et la branche 184 sont intégrés et propres, sans fichiers ignorés signalés. Ils sont conservés pour la livraison encore active et parce que le garde d'usage reste incomplet. Aucun retrait de worktree, branche ou ancien binaire n'est revendiqué.
Le candidat s'ajoute temporairement à la version active et au retour existant. Ces exécutables sont conservés. Les preuves locales restent dans /home/moi/.cache/bridget184, hors Git.

Prochaine action : refaire la demande native de maintenance, puis activer et vérifier ce même candidat si elle est admise. Ne pas annuler de missions ou modifier une base pour faire passer ce refus. Nettoyer les ressources de 184 après activation et vérification des usages.

## Complément SYS-LE-2 et instruction sur les refus
Observation du 27 octobre : wrapper115301 et Codex115521 ont pour cwd le worktree58459353 supprimé, device2052/inode2490404/nlink0. Le chemin courant est /mnt/storage/opus2d/worktrees/58459353, device2065/inode127664278. Le propriétaire a été informé ; aucun arrêt, changement de mapping ou remplacement métier exécuté par ce correctif.
Le contrôleur conserve son refus. La réponse précise la preuve manquante, la machine et le dossier attendu, les vérifications possibles et la décision humaine à demander si le déplacement était voulu.
Six tests ciblés PASS via scripts/build.py : trois spec184, matrice spec172 des états/preuves (dont rapport absent pour worktree), renouvellement du tour avec contexte et absence de mutation, projection spec174 de l'ancienne incertitude.
Revue initiale du complément : tâche fdf5a11f-9a44-48fe-a47d-5d68ebdbe18c, clé bridget-spec184-guidance-review-20261027-02. Deux objections ont été traitées : ancien incident après fin de tâche, codes non couverts.
Contre-revue finale : tâche42b9dfbd-47fd-4458-bb27-53a42aed9a91, clé bridget-spec184-guidance-review-20261027-03. Verdict APPROUVÉ, aucun bloquant identifié. Aucun modèle métier n'a été lancé.
Les six tests ciblés ont été rejoués après ces ajustements : PASS. Le cas terminal failed reste visible, tandis que l'incertitude initiale reste historique. Les codes inconnus déclarent leur diagnostic manquant et un recours humain sûr.
Complément source intégré et poussé : 7899d0bdab0112d804af04ef2966484a8ed07702.
Compilation release du candidat actualisé : PASS en56,62s.
Candidat vérifié et installé : /home/moi/.local/lib/bridget-communication/184-guidance-7899d0bdab01/bin/bridget.
Build184-guidance-7899d0bdab01, version0.6.3, SHA-256621101901c8fc6c3df278cc82f94133a02132d501bd1b4a57f55a07785753483.
Nouvel appel natif updates prepare : refus update_missions_active_or_unconfirmed. Aucun lien actif modifié ni service redémarré. Le daemon relu est toujours183-admission-7478e9e7 ; core/mobile/workerCartae et liaisonSYS-LE-2 sont actifs.
L'appel installed a de nouveau différé le nettoyage pour observation des usages incomplète. Le candidat initial1caabb9e est conservé avec le candidat actualisé et les exécutables actifs/retours : aucun retrait forcé pendant la livraison ouverte. Le worktree/branche184 restent propres, intégrés mais conservés jusqu'à activation et contrôle d'usage. Aucun travail Opus2D touché.
Les refus améliorés et la projection historique sont donc livrés en source et dans le candidat, PAS dans le runtime actif.


## Nouveau cas KS confirmé par le propriétaire — 27 octobre 2026
Sources : messages de Politique sol (3de3f2a6-6b4a-4415-8a1b-a870fe5a010f), mcp-60861-6ae0788d-1 puis mcp-60861-6ae07d4d-4.
Demande : opus166-industrialisation-capacite-ksstore-20261027-01.
Tâche : a61dbc4e-b492-4869-a2bc-025c1764ecb9 ; mission : f92fc599-0f81-42a3-96ab-000fa747c4fc ; enfant : 8f67503d-1d2b-42a4-9121-8bbd73b2a38e.
Choix déclaré : claude-ksstore / sonnet, development, shared, base ed3fa1c93956c8db0c3abd4680e0ebd645aad454.
Source sur la machine du parent : /Users/moi/Nextcloud/10.Scripts/69.opus2D.
Retour initial : remote_start_failed:remote_release:WorkerUnavailable ; statut suivant working avec erreur initiale encore affichée.
Le second message du propriétaire confirme ensuite un résultat complet, result_available, error=null, lu puis acquitté, avec exécution réelle de commandes. Aucun doublon ni repli créé.
La tâche SYS distincte 63ba20d3-7cbf-45d9-ad13-6431aaee371f a aussi rendu son diagnostic selon ce même message.
Le blocage métier de ces deux tâches est levé. Aucune relance ni nouvelle lecture demandée ou effectuée ici.
Le délai initial Release reste une preuve utile pour184. Il ne prouve pas un arrêt de la mission, ni l'activation du correctif.
Ces faits sont des observations du propriétaire, pas une nouvelle inspection runtime par le mainteneur.
La livraison du correctif184 dans le runtime reste ouverte. Aucun changement des services ou de la maintenance.


## Cas KS dépendances locales — 27 octobre 2026
Source : Politique sol, message mcp-60861-6ae088e3-8.
Clé : opus166-sol-code-dependances-locales-ksstore-20261027-01.
Tâche et commande : b0f614ff-b4c4-47bf-b5b7-4b3f9ce649a8.
Enfant : 9e8cdb9c-1c58-4b9f-bdfe-d94194f1f6cd ; instance : 631d88e1-b7a7-43eb-a90b-fa4bec35ce93 ; génération : 1070.
Base déclarée : 5bdcf4a7636142ca433a777228b1e5ff9dec7732.
Retour propriétaire initial : remote_start_failed:remote_release:WorkerUnavailable, status starting, work_state working. Aucun doublon ni fallback.
Lecture technique ciblée du reçu de cette commande dans le store worker KS : phase released, native_ready=true, startup_failed=false, startup_failure=null, terminal_exit_code=null ; enfant et identités concordants. Dernière écriture du reçu : 1793099870.
Le groupe marqué 3084164 est présent : processus bridget, même starttime 3101978941 que le marqueur, même groupe. Aucune action sur le processus.
Ces preuves établissent une libération enregistrée, une préparation native attestée et un wrapper encore vivant à la lecture. Elles ne prouvent ni la remise du travail au fournisseur ni un résultat métier.
Le journal Bridget disponible pour cet enfant est vide ; aucun avancement ou échec déduit de cette absence.
Le maître est encore183-admission-7478e9e7. Le correctif184 inclus dans le candidat186 n'est pas actif. La livraison runtime reste ouverte, sans nouvelle correction identique ni redémarrage.
Une seule projection propriétaire a été demandée, message mcp-79687-6ae0890b-5. Aucune réponse reçue pendant les attentes bornées ; pas de sondage périodique ni d'emprunt d'identité.
La confirmation de l'état métier reste au propriétaire via le statut natif de cette même tâche. Aucune nouvelle mission, continuation, modification de store ou réécriture de garde.


## Projection propriétaire et refus de chargement — même tâche KS
Source : Politique sol, message mcp-60861-6ae08c10-a.
Projection déjà lue pour b0f614ff-b4c4-47bf-b5b7-4b3f9ce649a8 : status mission_pending, work_state working, error remote_start_failed:remote_release:WorkerUnavailable, detail null, round1, result_available=false, result null, mailbox null.
Cette projection complète le reçu released/native_ready. Elle ne prouve ni fin métier ni remise du travail au fournisseur. La demande de projection mcp-79687-6ae0890b-5 avait expiré avant démarrage ; ce fait de transport est distinct de la mission KS.
Nouvel appel natif, explicitement demandé pour appliquer le correctif ou nommer le blocage :
/home/moi/.local/bin/bridget updates prepare 2A68A218-BBF9-4434-8A66-6B143179B5A8
Refus effectivement reçu : update_missions_active_or_unconfirmed. Aucun bail de maintenance accordé.
Le statut natif relu confirme encore183-admission-7478e9e7. updates status annonce0.6.3 up_to_date : cette égalité de version stable ne signifie pas que le build184 est chargé.
La garde de préparation vérifie notamment DelegationStore.pending(). Une tâche mission_pending entre dans cette collection ; elle suffit donc à empêcher la préparation, indépendamment des autres activités. La lecture du code explique ce critère ; aucune lecture globale des tâches ni modification directe du store n'a été faite.
Le correctif184 est déjà qualifié et inclus dans le candidat186-guidance-35123122672a. L'installation de ses octets ne modifie pas le code du maître183 vivant. Aucun parcours de chargement à chaud de ce correctif n'est livré.
Blocage concret : obtenir une maintenance native admise alors que cette tâche reste pending et que le propriétaire impose la conservation de son instance. Un redémarrage forcé ou une annulation pour vider le verrou ne répond pas à ce mandat.
La remise de cette tâche reste non confirmée. Aucune relance, continuation, fabrication, modification Opus2D, réécriture d'erreur ou de garde. La livraison runtime reste ouverte.


## Annulation KS confirmée et garde restante — 27 octobre 2026
Décision propriétaire : Politique sol, message mcp-60861-6ae09bac-b. Annulation native de sa seule tâche b0f614ff-b4c4-47bf-b5b7-4b3f9ce649a8 ; aucune autorisation d'annuler les autres missions pour forcer la maintenance.
Lecture worker : même commande/instance/génération, phase terminal, terminal_exit_code=0, updated_at=1793104811 ; ancien PID3084164 absent. Aucun arrêt manuel.
Worktree conservé : /home/moi/.local/share/bridget-projects/opus2d/repository/.worktrees/b0f614ff.
Lecture Git : HEAD5bdcf4a7636142ca433a777228b1e5ff9dec7732, branche166-sol-code-local-ksstore-20261027,0entrée de statut,0fichier non suivi à cette lecture. Aucun contenu effacé ou changé par le mainteneur. Ce constat n'est pas un inventaire antérieur ni une preuve exhaustive des sorties hors dépôt.
La vue native de contrôle, control delegations list --state all --limit100 --json, contient cette même tâche en state cancelled et le bon enfant9e8cdb9c-1c58-4b9f-bdfe-d94194f1f6cd. Sa fin n'est donc plus seulement un reçu worker.
La vue native active est complète : total_active=2, deux éléments. Tâche c99a257f-ba09-491c-98e0-51e5255ad464 de monuments en result_available ; ancienne tâche e5959986-e426-422d-956f-e035dbcf89be en cancelling. b0f614ff n'y figure plus. Aucune action sur ces deux tâches.
Nouvelle préparation native après ce changement d'état :
/home/moi/.local/bin/bridget updates prepare B27D32B5-CFDB-44AC-8079-45AF179D9903
Refus actuel : update_missions_active_or_unconfirmed. Aucun bail obtenu, aucun chargement ou redémarrage. Le maître reste183-admission-7478e9e7.
L'annulation demandée de b0f614ff est confirmée. La livraison runtime184/186 reste bloquée par d'autres travaux ou états à réconcilier ; la liste active n'exclut pas les autres gardes internes.
La tentative de retrait de la demande de lecture propriétaire mcp-19961-6ae09bf2-1 est refusée : demande déjà terminale timed_out. Aucune réponse supplémentaire n’est attendue ; la confirmation native est maintenant disponible. Aucun réveil périodique, rejeu ou mission de substitution.


## Confirmation propriétaire KS et diagnostic du dernier verrou historique
Source : Politique sol, message mcp-60861-6ae09cd5-c. Sa lecture native confirme b0f614ff-b4c4-47bf-b5b7-4b3f9ce649a8 cancelled/work_state cancelled/error null. Le worktree est kept/report_unavailable ; ce rapport natif reste inconnu malgré la lecture physique Git propre précédemment rapportée. Aucun retrait autorisé ni effectué.
La demande de lecture expirée et sa notification de délai ne rouvrent pas cette tâche. Aucune nouvelle mission n'est créée.
Contrôle courant : maître183-admission-7478e9e7 ; la vue active native contient toujours les deux autres tâches c99a257f-ba09-491c-98e0-51e5255ad464 result_available et e5959986-e426-422d-956f-e035dbcf89be cancelling.
Cette dernière appartient au mainteneur de ce fil. Son statut natif porte native_cancel_unverifiable:5022fa13-52e5-48a1-beba-2d6e27ff79f2, continuable=false, sans résultat.
Enfant déclaré unreachable, connexion vide, persistent null, lien native_delegate réservé ; aucun journal disponible. Ancien dossier /home/moi/10.Scripts/64.bridget/.worktrees/e5959986 absent. Ces absences ne prouvent pas l'arrêt du fournisseur et ne justifient aucune réécriture d'état.
Une seule reprise de l'annulation native propriétaire, bridget_task_cancel sur e5959986-e426-422d-956f-e035dbcf89be, restitue cancelling et le même refus d'arrêt invérifiable. Aucun arrêt manuel, adoption d'identité, modification du store ou action sur la tâche de monuments.
Les gardes n'ont donc pas changé favorablement. Pas de nouvelle préparation identique de mise à jour après le refus réel déjà conservé ci-dessus. Le correctif184/186 reste préparé, non chargé. La tâche KS reste annulée ; son worktree est conservé.


## Maintenance : clôture propriétaire et blocage d’amorçage vérifié
Demande : Politique sol, mcp-60861-6ae0a6f6-d. Résoudre la maintenance sans arrêter les tâches d’autrui, forcer un redémarrage ou modifier le store à la main.
Une demande ciblée au propriétaire monuments (mcp-8847-6ae0a7c5-1) a obtenu la clôture native de c99a257f-ba09-491c-98e0-51e5255ad464. Réponse t3-8253ab4d-9dda-4995-b9bb-e04a83d9f32d : result_available, continuable=false, kept/commits_to_review, error=null ; stop_unconfirmed levé. Cinq commits, résultat, worktree et 24 fichiers sauvegardés par empreintes sont conservés selon le propriétaire. Aucun arrêt de cette tâche par le mainteneur.
La tâche historique propre au mainteneur e5959986-e426-422d-956f-e035dbcf89be reste cancelling. Son identité distante persistée est cartae / worker-cartae / révision1, instance1ce61044-88d0-41d9-a78e-7c5458df0cad, génération739, enfant5022fa13-52e5-48a1-beba-2d6e27ff79f2. Clé d’origine : review172-worktree-parent-permissions-cartae-20261022-r1.
Lecture SQLite strictement en mode ro : saga spawn starting ; aucune entrée correspondante dans la flotte désirée courante. Aucune donnée modifiée. Le dossier absent et la présence unreachable ne constituaient pas le diagnostic suffisant : cette tâche utilise le parcours worker distant, même si le worker se trouve sur Cartae.
L’appel natif stop de ce seul enfant propre au mainteneur (commande0c1c46f5-b85e-465a-b2f1-fb87a9e33e77) retourne : STOP INCOMPLET, délai dépassé dans l’état « worker arrêt non attesté ». Aucun signal manuel ni second fournisseur.
Le worker conserve un reçu de préparation du workspace pour cette tâche, avec instance vide / génération0. Aucun reçu de commande de cette tâche ou de cet enfant n’existe dans le store worker courant. Une recherche bornée des noms dans les racines Bridget de cache/share/state (16 820 dossiers, profondeur7, périmètre explicitement limité) ne trouve que ce reçu de préparation. Ce n’est pas une recherche exhaustive de sauvegardes externes, ni une preuve que la commande n’a jamais été lancée.
Le code Cancel du worker exige un reçu de commande exact ; son absence produit UnknownExecution. Le texte générique du maître ne permet pas, seul, d’établir à quelle étape l’appel réel échoue. La saga et la preuve de préparation ne remplacent ni un reçu de lancement ni une preuve terminale.
Contre-revue indépendante en lecture seule, source d3e61273130580b981cbc172049b6ed677373d1c : tâche4ad0548f-b6ed-4712-9520-f2169ffe95a0, demandebridget-maintenance-cycle-review-20261027-01. Elle confirme le blocage circulaire : Cancel ne clôt pas sans preuve worker ; Updates Prepare refuse une tâche pending ; aucun port de chargement à chaud ne modifie183 vivant. Elle rejette explicitement l’inférence Terminal depuis une absence de reçu et le rejeu Prepare/Start comme réparation. La revue est lue, acquittée, puis sa garde clôturée nativement : continuable=false, error=null. Aucun build ou test relancé pour cette investigation de lecture seule.
Résultat opérationnel : la garde métier de monuments est fermée, mais aucune preuve nouvelle n’autorise la clôture de e5959986 ni l’activation. Les candidats184–187 qualifiés restent inactifs. Aucun nouveau Prepare identique n’est lancé alors que le dernier obstacle est connu. Aucun store écrit, tâche recréée, service redémarré ou correction déclarée chargée.
Prérequis restant : retrouver une preuve autoritaire de cette commande et son arrêt pour permettre la réconciliation native ; si elle est irrécupérable, faire décider une procédure exceptionnelle de migration contrôlée. Cette procédure n’est pas disponible dans183 et ne doit pas être présentée comme prête ou couverte par un simple nouvel essai de maintenance. Le mandat courant exclut le forçage ; la livraison runtime reste ouverte.
