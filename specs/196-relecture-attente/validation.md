# Qualification196 — Lecture d’attente

## Incident réel et portée du diagnostic
Tâche9cc87183-7fd2-4338-8605-705f5a2f0d28, ronde2, propriétaire87515984-f549-49ae-ab9c-a6c47dada0e5. Le Wait45s a rendu « réponse daemon indisponible : budget JSONL dépassé ». Le statut suivant a retrouvé le résultat stable (808octets,20testsPASS). La tâche n’était pas perdue. Lecture durable ciblée : résultat disponible, cleanup_done et guard_released vrais. Aucune mutation de cette tâche.

Le lecteur JSONL émet ce texte pour un délai expiré ; une taille excessive produit un autre refus. Le budget de Wait tient déjà compte de l’attente demandée dans le client chargé183. Le précontrôle CallerScope est une lecture distincte. La trace historique ne permet pas d’identifier laquelle a expiré, ni pourquoi. Ne pas attribuer la panne à la taille, au compte ou au réseau sans preuve.

Demande d’admission distincte opus2d-revue-recherche-r42-20261028-14 : le retour propriétaire donne not_recorded/task_id=null et quatre familles de motifs. La lecture ciblée n’a trouvé ni reçu durable ni tâche de cette clé. Cela ne rend aucun candidat éligible et ne prouve pas une panne de compte. Aucun rejeu, nouvelle sonde ou remplacement exécuté.

## Correction commune
Seul bridget_task_wait récupère un échec daemon_unreachable. Une nouvelle lecture native des mêmes task_ids attend1s, avec même socket et identité. Tous les contrôles et attestations sont refaits. Aucun cache de droits, mutation, nouvelle mission ou augmentation des délais. Réponse réussie marquée read_recovery ; second échec guidé vers task_status, cause exacte explicitement inconnue. Refus métier et interruption hôte conservés. Le délai initial est validé avant de le remplacer, y compris si le précontrôle a échoué.

Complexité : au plus deux parcours pour16tâches. Aucun service, stockage ou dépendance ajouté. Auto-revue : le remplacement d’un délai initial invalide a été fermé par réutilisation du parseur existant et un test. Le wrapper ajoute une règle de récupération limitée, et non une abstraction générale de retry. Les fixtures utilisent le vrai protocole Unix/inscription auxiliaire aux frontières.

## Contrôles
Contre-épreuve initiale avant correction : exit101, daemon_unreachable au lieu de l’observation récupérée. Après correction :7tests196PASS. Scénarios : coupure avant Wait et après Wait45, relecture1s/mêmesIDs, nouveau refus de propriété conservé, double échec guidé, refus initial non rejoué, délai invalide non corrigé silencieusement, mutation non rejouée. Logs privés : /home/moi/.cache/bridget196/baseline.log et /home/moi/.cache/bridget196/spec196-final.log.
Guides32pages/522liens/13pairesPASS ; git diff --check PASS. Première régression151 :58PASS et un ancien test de schéma ACK en échec sur le nombre global d’outils (34au lieu de32), sans lien avec la récupération. Suppression des deux assertions de compte global/pour délégation devenues périmées ; les assertions utiles du schéma ACK restent. Régressions151 :59PASS ; plafond158 :7PASS ; déconnexion174 :1PASS. Attestation172 :1PASS. Total75contrôles ciblés PASS, aucune FULL. Revue indépendante5007923b favorable, aucun défaut concret identifié. Résultat lu et ACK ; clôture native confirmée cleanup_done=true, guard_released=true, enfant30424a6e dans cancel_stopped.

## Livraison et limites
Ce code appartient au client commun, compilable pour les plateformes prises en charge ; il ne dépend d’aucun nom de serveur ou projet. Une future installation contenant ce commit bénéficie de la correction. Le client Mac vivant183-e2987fb7175f ne la possède pas. Le maître183 reste bloqué en maintenance par une clôture historique non attestée ; aucun restart forcé, remplacement de store ou intervention sur les missions tierces. Le chargement du client et une qualification réelle restent ouverts. Construire un candidat Linux ne prouve pas la qualification Mac ni le rechargement du MCP vivant.

La réparation195 du cache Git KS est distincte : elle a rétabli l’usage natif mais n’est pas un automatisme livré dans Bridget. Ajouter une écriture au diagnostic ou à chaque sonde casserait leur lecture seule. La prévention générique demande un parcours d’entretien autorisé et qualifié séparément. Elle n’est pas annoncée comme disponible.

## Candidat et état effectif
Source a42d133cd3d21f31c07c146a8d8bf8bc0418c685 intégrée et poussée sur main. Build release Linux PASS, build196-wait-a42d133cd3d2. Candidat séparé /home/moi/.local/lib/bridget-communication/196-wait-a42d133cd3d2/bin/bridget, SHA25683ee70322c50329cd8bd9a1866f3c70fa8409f124738847a4d9dc4dc507350c4, version0.6.3. Copie source/destination identique et entretien installed exécuté ; manifest eligible_for_activation=false. Aucun nouveau daemon ou fournisseur lancé pour la qualification. Le maître351206 reste183-admission-7478e9e7 ; le client Mac68740 reste183-e2987fb7175f. T004 reste ouverte : aucun artefact Darwin qualifié/rechargement natif du client vivant ni usage réel de cette récupération attesté. La tâche initiale reste préservée, aucun rejeu métier.

Preuves privées : /home/moi/.cache/bridget196/ (tests/build/closure-review.json) et /Users/moi/.cache/bridget196/review-result.json. La source de revue sur SYS est conservée provisoirement : /home/moi/.local/share/bridget-projects/bridget/repository/.worktrees/5007923b. Garde close, HEAD/branche conformes, Git propre et aucun ignoré ; scan du compte sans usage observé mais quatre processus partiellement illisibles empêchent de conclure absence totale d’usage. Aucun forçage. Référence légère SYS peut rester sous son main ancien ; ne pas supprimer de force.

La version officielle déjà publiée n’est pas modifiée par un push de main. Une future installation ne bénéficie des sources196 que si son binaire est construit depuis ce commit ou un descendant. Le bilan distingue sources partagées, candidat et processus actif.

## Nouveau signalement280b — 29octobre2026

Message propriétaire opus167-lecture-resultat-budget-jsonl-20261029-01. Trace T3 native confirmée le29octobreà05:08:54.892UTC : bridget.bridget_task_wait, tâche280b29fa-5263-4377-95d4-b7ece5e5c3ef, wait_timeout=1, réponse brute « réponse daemon indisponible : budget JSONL dépassé ». C’est une expiration de lecture ; aucune preuve de dépassement de taille ou de perte du résultat.

Lecture native de l’autorité sans mutation :280b=result_available/errornull/cleanup_done,2436octets conservés. Les tâches4e8f6675 et1c33ef1f également result_available/errornull/cleanup_done,2326et3785octets. Résultats signalés au propriétaire sous mcp-50239-6ae2d9f4-13, acceptéin_flight ; récupération par task_status surlesmêmesIDs, aucune nouvelle mission.

Le correctif196 est déjà dans les sources et dans le binaireDarwin201 installé. SHA201 0f24f3afa881354ae89d8247aa45beb4bfea46fb4cc704ad94a7779135c912a5, chaîne task_wait_read_unavailable présente. Plusieurs MCPvivants, dontroot50239, chargent201. Un MCP183 PID66991, parentCodex63300, reste vivant ; son binaireSHAe3136509cbc3f6dfefe123cfe1e793d988a849fe00d0d68caa9b74f012eebeef ne contient pas la récupération196. Cela ne prouve pas encore qu’il a servi CET appel de Politique. L’erreur brute historique est compatible avec un ancien client, mais la cause détaillée du délai et son client exact restent à qualifier. Aucun second correctif196 ajouté.

Le contrôle natif exposé ne fournit pas de rechargement MCP. L’entrée installée /Users/moi/.local/bin/bridget et la configuration Codex bridget pointent vers201, mais cela ne recharge pas un processus ouvert. Outil graphique indisponible (CUA_REPL_ENABLED_SURFACES is required). Aucun processus/client/fournisseur terminé à l’aveugle ; aucun serviceOpus2D ou calculNE02 touché.

Demande ciblée au propriétaire mcp-50239-6ae2dbc8-14/delivery77c213bb-2562-4b53-861f-c8ebc58281a8 : un message natif sansidfourni permet de lire le PIDdansl’idgénéré (format vérifié dans mcp.rs), sans nouveau lancement. Requête encoreopen à l’attente native unique de30s ; ne pas multiplier les réveils. T004resteouverte : ne pas annoncer touslesclientsactualisés ni l’incidentcorrigé sans preuve dechargement duclientconcerné.
