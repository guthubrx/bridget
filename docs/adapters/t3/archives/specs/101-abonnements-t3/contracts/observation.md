# Archives techniques de l’adaptateur T3

Document d’origine : [specs/101-abonnements-t3/contracts/observation.md](../../../../../../../specs/101-abonnements-t3/contracts/observation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

T3 : contrat HTTP 098 conservé, complété par lecture locale minimale de
`userdata/state.sqlite`, colonnes `thread_id, provider_name, resume_cursor_json`
de `provider_session_runtime`. Jamais d'écriture dans cette base.
Formes inconnues, IDs multiples, processus périmés → refus de rattachement.

<a id="extrait-2"></a>

## Extrait 2

États listés : active, source_unavailable, interrupted. Restart du daemon :
trace conservée interrupted, nouveau sub requis. Types : available et UUID sources.
T3 : seule fin latestTurn connue ; activités500 avant compression ; chemins12
par activité. ÉcrituresCodex attestées uniquement, ClaudeT3 explicitement non
compatible avec file_written/file_collision. Permissions = demandes observées,
éventuellement déjà traitées. Catalogue global annonce une couverture partielle.
