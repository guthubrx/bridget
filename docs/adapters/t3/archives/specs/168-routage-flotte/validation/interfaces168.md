# Archives techniques de l’adaptateur T3

Document d’origine : [specs/168-routage-flotte/validation/interfaces168.md](../../../../../../../specs/168-routage-flotte/validation/interfaces168.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- bridget_auto_delegate construit V2 et exige auto_delegate_v2 ; V1 reste un chemin distinct.
- bridget delegate transmet les axes et résout le SHA de la source locale avant l’enveloppe.
- delegate --catalogue [--machine] [--json] lit le catalogue sans lancement ; le filtre est local à la CLI.
- Le maître impose même UID Unix, rôle Client et capacité routing_catalogue_v1. Aucune autorité T3 n’est requise pour cette lecture.
- Le plan V2 possède son snapshot typé ; ses faits métier sont conservés dans metadata v2 opaque pour E. metadata.decision: Option<LineageDecisionV4> vaut None avant choix, puis l’objet complet fermé source/combinaison au choix.
- Les constructeurs externes AutoAdmissionReceipt doivent renseigner metadata_version:u8 ; les reçus163 portent1, les nouveaux snapshots168 portent2.
- Le reçu C laisse explicitement à raccorder : probes B, score/saturation, chaîne service→Claude→score, CAS de choix, Prepare.base_commit et reprise ; A raccorde désormais la projection Lineage.
- Pour chaque candidat Git Shared/Worktree, expected_base_commit source exact doit atteindre probe et Prepare. Sans SHA, exclure ce candidat par workspace_base_required ; sans candidat restant, refuser globalement. Hors Git Shared général peut omettre le SHA ; un SHA fourni hors Git refuse mismatch. Une erreur technique ne signifie jamais hors Git. Worktree exige Git.
