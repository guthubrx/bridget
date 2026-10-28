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
Guides32pages/522liens/13pairesPASS ; git diff --check PASS. Première régression151 :58PASS et un ancien test de schéma ACK en échec sur le nombre global d’outils (34au lieu de32), sans lien avec la récupération. Suppression des deux assertions de compte global/pour délégation devenues périmées ; les assertions utiles du schéma ACK restent. Régressions151 :59PASS ; plafond158 :7PASS ; déconnexion174 :1PASS. Attestation172 :1PASS. Total75contrôles ciblés PASS, aucune FULL. Revue finale à compléter.

## Livraison et limites
Ce code appartient au client commun, compilable pour les plateformes prises en charge ; il ne dépend d’aucun nom de serveur ou projet. Une future installation contenant ce commit bénéficie de la correction. Le client Mac vivant183-e2987fb7175f ne la possède pas. Le maître183 reste bloqué en maintenance par une clôture historique non attestée ; aucun restart forcé, remplacement de store ou intervention sur les missions tierces. Le chargement du client et une qualification réelle restent ouverts. Construire un candidat Linux ne prouve pas la qualification Mac ni le rechargement du MCP vivant.

La réparation195 du cache Git KS est distincte : elle a rétabli l’usage natif mais n’est pas un automatisme livré dans Bridget. Ajouter une écriture au diagnostic ou à chaque sonde casserait leur lecture seule. La prévention générique demande un parcours d’entretien autorisé et qualifié séparément. Elle n’est pas annoncée comme disponible.
