# Contrat 136

Post: kind optionnel history/action/blocker/decision ; supersedes_seq optionnel
entier positif, exige classe action/blocker/decision. Null explicitement refusé.
History: notify=[] obligatoire, pas supersedes_seq ; limite16Kio inchangée.
Autres classes: body1–2048 octets UTF-8, références des preuves dans body.
Remplacement: même fil/auteur/audience effective, cible antérieure non history
sans successeur. Refus invalid_supersession sans entrée/ACK/wake modifié.
Omission des métadonnées: legacy et canon inchangés. Rejeu divergent: envelope_mismatch.

Read: body actuels ; références compactes presentation=history_reference ou
superseded_reference. superseded_by_seq ≤ snapshot_seq. Rejouer une page
avant ACK garde son contenu. Nouvelle correction: page suivante après ACK.
History retourne toujours le texte exact, qui n'est pas une consigne actuelle.
Action de relecture: history, thread_id/from_seq/to_seq copiés depuis history_ref,
limit1–200 et60Kio ; mêmes droits, aucun déplacement du repère, aucun reçu créé.
Audience figée dans notify_json.targets au dépôt, y compris all. Les wakes ne
sont pas cette source ; les membres102 ne changent pas après création.
Contrôler les pages suivantes avant d'agir ; un ordre déjà remis ne peut être rappelé.

CLI: thread post ID --kind action --supersedes 4 --notify UUID --id UUID -- 'consigne'.
MCP: action=post, kind=action, supersedes_seq=4, notify=[UUID], body=consigne,
thread_id=ID, operation_id=UUID. Historique: kind=history, notify=[].
Messages directs non filtrés : aucun faux classement automatique.
