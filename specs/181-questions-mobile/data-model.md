# Données181

Demande : id, owner_agent, owner_instance, project_id, origin, canonical_questions, revision, callback, created_at, updated_at, deadline_at, state. Clé unique sur origine/callback exact et demande du producteur. Index agent/état et rétention.
Réponse : command_id unique, request_id, actor humain/appareil attesté, revision, réponses par question {selected:[],text:""}, payload canonique, statut submitted/applied/unknown et reçu producteur. Champs figés à la transaction de décision. Aucune dernière écriture gagnante.
Brouillon local : liaison session/device, agent, question et révision ; sélections, texte et position du formulaire. Suppression seulement après confirmation ou action explicite, jamais par absence dans une page tronquée.
Producteur : identité native Bridget, capacité réelle de réponse et callback exact ; T3 n’est qu’une origine facultative. Fin de session/révocation ferme le droit de remise.
Données vocales : aucun enregistrement audio stocké par Bridget ; texte renvoyé par le moteur Android et placé dans le brouillon. Pas d’envoi automatique.
