# Validation220 — en cours

Source initiale relue : 06dfbad1bad533e4c6f1e68861d2f64effe48bef. Aucune activation220 attestée à ce stade.

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
