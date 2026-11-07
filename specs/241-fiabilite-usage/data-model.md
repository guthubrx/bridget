# Modèle de données 241

Réutiliser AutoAdmissionReceipt, Task et NativePermissionSnapshot.
L'identité de reprise demeure owner+request_id et le canon demeure immuable.
Une autorité durable concerne une seule Task, son propriétaire, sa politique
effective, ses sources, son placement et sa durée. Aucun credential T3 dans le
payload. Une tâche historique garde son contrat historique. Le marqueur de
contrat est lisible par le retour qualifié avant activation.

Les faits d'outillage sont observés sur la cible, datés et limités à la sonde.
Ils ne sont ni des droits ni un cache permanent de disponibilité. Ils ne
modifient pas la définition ou la connexion fournisseur. Aucun nouveau magasin.

Les mesures dérivent des échanges et états existants. Pas de compteur durable
redondant lorsqu'une preuve de recette ou un journal suffit.

Task.admission_authority_version : Option<u16>, absent par défaut et omis du
JSON historique ; seule valeur comprise : 1. admitted_parent vérifie version,
propriétaire, instance courante ou initiale, cwd et instantané. Le marqueur n’est
pas un paramètre MCP. Les nouveaux lecteurs le conservent à la réouverture.
CandidateProbeV2.required_tools et CandidateProbeV2Result.tool_observations :
listes fermées optionnelles vides pour l’enveloppe historique. WorkloadToolsV1
est négocié ; les faits sont bornés au lot et ne sont pas des droits.

CandidateProbeV2.requires_workspace_write : bool, false par défaut et omis
quand faux. Présent seulement sous WorkloadToolsV1 ; true exige une liste
d’outils non vide. Il décrit une opération identifiée, jamais un droit accordé.
Le profil effectif est relu pour conserver les restrictions de l’appelant.
