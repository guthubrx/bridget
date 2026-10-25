# Revue indépendante 180

Demande mobile180-review-20261025-r1, tâche b4022ecf-8f09-4354-8e7a-3ff55903df9b, moteur choisi par Bridget : Claude Haiku5.5, discovery. Résultat durable result_available, cleanup_done=true. Lecture directe des fichiers, sans shell ni exécution de tests ; limites conservées. Le montage MCP a ensuite rendu t3_session_unavailable/binding_transport_unavailable lors des lectures. Le résultat a été relu en lecture seule dans le magasin natif, sans recréer de mission.

Décisions :
- Liste32 : la rétention mobile est elle-même bornée à32 par destinataire. Le 33e retenu depuis ce chemin est donc impossible. Garde renforcée malgré cela : une phase serveur held ne devient jamais released par absence de la liste.
- Guidage après panne : défaut confirmé. La projection de remise relit le reçu durable queue_edits/promote/applied. Canon et octets initiaux restent inchangés conformément au contrat159. Test de reprise ajouté.
- Révision cancel/promote : la lecture et la mutation s’exécutent sous le même verrou exclusif DaemonState. Aucune mutation de file native ne peut s’intercaler. La phase est aussi relue sous transaction dans apply_cancel/apply_promote. Pas de course confirmée dans le modèle existant.
- Retour promu : déjà corrigé pendant la lecture ; état Interruption demandée conservé jusqu’à observation.
- expect du périmètre : remplacé par refus explicite. Empreinte hexadécimale : minuscules imposées.
- Rétention remplacements : harmonisée à7jours ; collisions de clés entre édition et autres actions refusées.
- Intrusion second téléphone : test explicite ajouté.

Minimalisme : aucun framework ni dépendance ajouté. Les composants et magasins natifs sont réemployés. Le calcul de positions est indexé. Le daemon atteste les phases ; le téléphone conserve l’état local incertain, nécessaire hors connexion. La séparation ajoute une charge modérée et justifiée par les reprises réseau. Les tâches et preuves restent ouvertes jusqu’à validation réelle.
