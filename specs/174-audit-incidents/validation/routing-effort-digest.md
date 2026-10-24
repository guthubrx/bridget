# Routage : changement d'effort et empreinte du profil

## Incident et cause

Les deux demandes de revue mobile du 24 octobre sont enregistrées en refus
durable `routing_connection_changed`, sans tâche et sans choix durable.
Les journaux du worker montrent, pour chacune, un premier lot de dix candidats,
puis deux sondes d'un candidat. Ce second contrôle unitaire correspond à la
branche où l'effort choisi diffère de l'effort par défaut.

`WorkerRuntime::inspect_candidate` résout le profil avec les arguments du
modèle et de l'effort. Son empreinte change donc lorsque l'effort change.
`refresh_routing_selected` exigeait pourtant la même empreinte à cette étape.
Le contrôle confondait une variation autorisée de l'effort avec une dérive
de la définition. L'effort exact choisi lors des incidents n'est pas conservé
dans ces reçus refusés ; il n'est pas inventé ici.

## Correction

La première relecture du candidat reste inchangée. Si l'effort change, la
seconde requête contient deux candidats : l'effort choisi et l'ancien effort
comme témoin. Aucun autre modèle, compte ou hôte n'est essayé.

Le worker V2 charge déjà un registre unique pour le lot. Il clôt aussi une
opération commune de lecture des droits ; une source instable invalide les
verdicts du lot. Aucun champ ni version de protocole n'est ajouté.

Le témoin doit conserver définition, droits, époque, mapping et projet.
La sélection doit rester sur la même époque, le même mapping et le même projet.
Son empreinte de définition peut différer : elle décrit ses nouveaux arguments.
Ses droits restent attestés, puis contrôlés à Prepare. La disparition d'un
des deux verdicts ferme le contrôle ; le tri ne transforme pas le témoin en choix.

Coût : un candidat supplémentaire dans un lot borné à deux, sans aller-retour
réseau supplémentaire. Aucun changement de schéma, d'idempotence ou de permission.

Le refus réel garde son code. Son détail explique maintenant la vérification
du catalogue et du lien, le suivi de l'admission existante et la fermeture d'un
refus durable. Le texte n'invite pas à contourner les contraintes.

## Référence T3

Dans T3, `ProviderCommandReactor::ensureSessionForThread` vérifie l'identité
de l'instance. `CodexAdapter::sendTurn` transmet séparément l'option
`reasoningEffort` pour cette instance. Ce mécanisme confirme la distinction
entre identité du fournisseur et option du tour. T3 n'a pas la sonde distante
de Bridget ; il ne justifie aucune suppression de nos preuves de droits.

## Validation

Le test de routage avec sockets Unix utilise maintenant les vrais digests de
`AgentRegistry::pin_delegation_model` et `resolved_definition` pour high/low.
Avant correction : échec `routing_connection_changed` sur ce seul changement.
Après correction : même test réussi.

La fixture couvre aussi une dérive de définition, mapping ou droits, puis
la disparition du témoin et celle du choix. Toutes restent refusées sans tâche
ni remplacement. Les réponses périmées et le rejeu du choix durable restent testés.

Lot `spec168_` et `native174_projet` : 40 réussis, 0 échec, 0 ignoré.
Il inclut le routage, les choix HTTP, les quotas, l'idempotence et le précédent
correctif de projet. Les fournisseurs sont simulés ; aucune mission métier
mobile n'est rejouée. Les journaux RED/GREEN restent privés hors Git.

La contre-revue officielle initiale est elle-même refusée par l'ancien binaire :
`bridget174-routing-effort-review-20261024-01`, état refused, task_id nul.
Ce refus n'est pas une revue acquise. Aucun repli de moteur n'a été employé.

## Livraison

Correction préparée sur la base commune `fb415ad7`, dans une branche séparée.
Le runtime mobile est conservé pendant sa livraison par son responsable.
Activation et contrôle final restent à consigner après leur exécution.
