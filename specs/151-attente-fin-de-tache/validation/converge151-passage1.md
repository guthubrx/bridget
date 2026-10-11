# Converge 151 - passage 1

Date : 2026-10-11. Verdict communiqué et confirmé par le principal.
Résultat : une tâche T027 ajoutée. Aucun bug de production prouvé.
Statut courant : In Progress, 26 tâches cochées sur 27.

Le seul manque retenu est la preuve SC-009 après ACK normal : RequestWait
doit rendre `answered` en 2 s au plus après le vrai traitement DeliverAcked.
L'attente commence sur une demande `open`. Le test vérifie le résumé exact
avec deux réponses de scopes différents partageant message_id et destinataire.
Les demandes initiales ont des clés distinctes. Les tests déjà verts prouvent
séparément la stabilité entre scopes, le retour sur un état answered et le
réveil guichet. Ils ne mesurent pas cette borne après ACK normal.

T027 ne demande pas de modification du contrat ou de production. Elle réutilise
les fixtures et observateurs existants. Un mark_answered direct ou un bump
manuel ne peut remplacer le passage par DeliverAcked. La façade possède ce
test. Son résultat reste en attente dans ce reçu ; aucune réussite n'est déduite.

Le principal confirme les 47 fichiers du gel inchangés hors tasks. Le fichier
de tâches courant commence par les 21424 octets historiques exacts. La section
Convergence commence à l'offset 21425, sans modification rétroactive.

SHA256 du préfixe historique :
`a12fc911af41a58fa1dd161275bd24cb82d56051384d2eb2021d4fcdcf40f08d`

SHA256 des tâches après ajout de T027 :
`77ea535b1a1dad065eccbf79dc28146daddd00b9c94181068e24dbe6fd18697b`

Fichier contrôlé en lecture seule :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/tasks.md

Les trois preuves T023 et les trois snapshots finaux avant Converge restent
byte-identiques. Aucun snapshot n'est écrasé. Les captures r2 attendent le
résultat T027 et le GO du principal. Aucun audit ni Cargo n'est lancé par
l'owner documentaire ; aucune source ou tâche n'est modifiée par lui.

Estimations conservées pour le rapport final : 140–320 min initialement,
puis 130–290 min après recalage. Après ajout de T027 : 240–290 min,
fin centrale annoncée à 12:44 CEST.
