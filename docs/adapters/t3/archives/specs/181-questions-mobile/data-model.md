# Archives techniques de l’adaptateur T3

Document d’origine : [specs/181-questions-mobile/data-model.md](../../../../../../specs/181-questions-mobile/data-model.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Demande : id, owner_agent, owner_instance, project_id, origin, canonical_questions, revision, callback, created_at, updated_at, deadline_at, state. Clé unique sur origine/callback exact et demande du producteur. Index agent/état et rétention.
Réponse : command_id unique, request_id, actor humain/appareil attesté, revision, réponses par question {selected:[],text:""}, payload canonique, statut submitted/applied/unknown et reçu producteur. Champs figés à la transaction de décision. Aucune dernière écriture gagnante.
Brouillon local : liaison session/device, agent, question et révision ; sélections, texte et position du formulaire. Suppression seulement après confirmation ou action explicite, jamais par absence dans une page tronquée.
Producteur : identité native Bridget, capacité réelle de réponse et callback exact ; T3 n’est qu’une origine facultative. Fin de session/révocation ferme le droit de remise.
Données vocales : aucun enregistrement audio stocké par Bridget ; texte renvoyé par le moteur Android et placé dans le brouillon. Pas d’envoi automatique.
