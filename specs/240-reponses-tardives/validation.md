# Validation 240 — en cours

Base929e09ef. Signalement mcp-61249-6aeebcc2-1. Aucun rejeu métier.

Preuve runtime : journal maître et worker KS du7novembre05:48:24UTC :
correlation62e012c1-463f-4d09-9e73-63ffcd951c69,
commande provisoire b6d51346-bd9e-48b3-86d8-d899398c5bb7, génération0,
instance vide, rejected_PrepareExpired puis command_mismatch et fermeture.

Reçus : A2 chosen/révision2/tasknull/cibleKS, machine non imposée dans le canon.
C-jonction admitted/révision3/task737767cf, puis failed, erreur de preuve
avant remise.5cd9 result_available/sans erreur/cleanup_done. Ces états sont
des lectures techniques ; aucune mission ou annulation n'est émise.

Snapshot5cd9 : Claude discovery impose tools=[Read,Glob,Grep] et disallowed
Bash/Edit/Write/NotebookEdit/WebFetch/WebSearch. L'absence de git/wc via shell
est une restriction de posture, pas une incapacité propre au modèle Haiku.

Le nouveau cache de corrélation est limité à4096 entrées, vidé sur la perte
du lien. Seules les préparations initiales réellement écrites et expirées
sont inscrites. Réponses tardives exactes ignorées, jamais appliquées.
Les codes fermés de capacité/lien restent des refus ; aucun droit ajouté.

Tests et revue en attente. Les délais de raccordement du parent restent à
expliquer ; aucun lien causal avec la maintenance n'est annoncé.
T3 reste ouvert et son rechargement reste reporté par François.
