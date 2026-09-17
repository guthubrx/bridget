# Contrat101

Outils publics existants conservés : bridget_events et bridget_journal.
Aucun argument `from`/identité native ajouté aux outils privés.

`events types` annonce les sources et les types effectivement observables.
`events sub` conserve event/agent/file/once/ttl_secs et refuse nommément
source introuvable, indisponible ou incompatible. Sans agent précis, couverture
limitée aux sources compatibles déclarées ; aucune promesse universelle.
`events list` expose état de surveillance, instance daemon et limites de remise.
La déconnexion d'une source n'est pas un turn_ended.

Les wrappers annoncent leurs capacités sur la connexion primaire attestée ;
un client auxiliaire ne peut ni annoncer des capacités ni produire des faits.
Les clients anciens sans annonce sont explicitement non compatibles.

T3 : contrat HTTP 098 conservé, complété par lecture locale minimale de
`userdata/state.sqlite`, colonnes `thread_id, provider_name, resume_cursor_json`
de `provider_session_runtime`. Jamais d'écriture dans cette base.
Formes inconnues, IDs multiples, processus périmés → refus de rattachement.

Fenêtre100 conservée : événements reçus par le daemon après abonnement.
Une fin de tour n'atteste ni succès ni fin de projet. Notifications système
exclues de l'observation afin de ne pas créer de cycles.

États listés : active, source_unavailable, interrupted. Restart du daemon :
trace conservée interrupted, nouveau sub requis. Types : available et UUID sources.
T3 : seule fin latestTurn connue ; activités500 avant compression ; chemins12
par activité. ÉcrituresCodex attestées uniquement, ClaudeT3 explicitement non
compatible avec file_written/file_collision. Permissions = demandes observées,
éventuellement déjà traitées. Catalogue global annonce une couverture partielle.
