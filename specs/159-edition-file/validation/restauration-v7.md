# T011 — répétition réelle de restauration v7

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/159-edition-file/validation/restauration-v7.md#extrait-1).

| Étape | Résultat |
|---|---|
| Copie cohérente pré-v7, puis ouverture debug 159A | Migration v6 vers v7, ouverture et arrêt RC0. PID 93760. |
| Ancien binaire 158B sur cette v7 | Refus fermé RC1 avant ouverture migrante ; octets DB inchangés, selon assertions de la recette. PID 93942. |
| Autre copie restaurée pré-v7, ouverte par le vrai binaire 158B | Version 6 conservée ; ouverture et arrêt RC0. PID 94193. |

Les trois états original/migré/restauré gardent exactement les mêmes comptages et empreintes logiques de toutes les lignes sur les colonnes pré-v7 : 11384 idempotency_records, 57 tracked_requests, 10892 remises et zéro lien d'exécution. Les phases présentes sont acked 9285 et indeterminate 1607. Les échantillons par phase et l'empreinte delivery_id/rang sont égaux. quick_check=ok et foreign_key_check=0 à chaque étape. Les versions maximales suivent 6→7→6. La recette vérifie aussi turn_gate=0 et acked_at=NULL des anciennes lignes ; aucune date d'ACK n'est fabriquée.

Limite : le snapshot réel n'a ni phase held/cancelled ni lien non vide. Les scénarios synthétiques T003 sur les autres phases et liens restent une preuve distincte. Cette répétition ne démontre pas de livraison, de rollback de production ou de compatibilité d'un futur binaire release.

Référence fraîche privée :
/Users/moi/.cache/bridget-restoration159A-reference-fresh-20261012/bridget158B-real-snapshot.db
SHA256 7eb91540b4e03b797caccc1ce51a736cb5741a45095a847717ba5b2ab9876e32

Binaire neuf réellement utilisé :
/Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug/bridget
SHA256 48c5ea41e10f4f0867d3754204df351bff548a2256309c5f48be3db520a2bebf

Vrai ancien binaire 158B sauvegardé avant reconstruction 159 :
/Users/moi/.cache/bridget-restoration159A-reference-20261012/bridget158B-reference
SHA256 cfb77a6552e97b28fd092ecbd73e82fbfcaa59694264b354fe593cb63e64018c
Source e52a96240b6168bf4466712e1bda8852b4ecb43f.

Copie migrée et copie restaurée réellement utilisées :
/private/tmp/b159.yppi0yse/v7/bridget.db
/private/tmp/b159.yppi0yse/restored-v6/bridget.db

Recette existante paramétrée, exécutée par ROOT :
/Users/moi/.cache/bridget-restoration159A-preparation-20261012/recipe-T011159A.py
SHA256 bc7e73dcc8f1557f08e4c2608e1a26e4cd333cb2133c9f4d38893ced88289c0a
Les commandes daemon et namespaces exacts figurent dans le JSON. Référence choisie par --snapshot et --snapshot-sha256 ; aucun timestamp ni contenu corrigé pour obtenir l'égalité.

Le premier FAIL demeure historique : la référence plus ancienne avait 62 tracked_requests ; l'ouverture en a conservé 57. Les cinq lignes retirées sont trois answered et deux timed_out terminées depuis plus de sept jours. Le diagnostic compare le prédicat de purge et les données ; le code de maintenance est byte-identique à 158B. Ce n'était pas une suppression par la migration T003. La nouvelle répétition utilise une nouvelle sauvegarde réelle v6 ; elle ne nettoie ni ne normalise la sauvegarde échouée.

Reçu PASS original :
/Users/moi/.cache/bridget-docs159A-T012-20261012/private/T011-PASS-original-private.json
SHA256 5853ac132170488bf260f34f31497feeac63a1aa096c304cfbe38aae5746aa11
Reçu FAIL original :
/Users/moi/.cache/bridget-docs159A-T012-20261012/private/T011-FAIL-original-private.json
SHA256 2f387ca9ba547c990e3812ae388cf24b215f5751bb621dce035690a9a95ea65e

Diagnostic de maintenance, lecture seule :
/Users/moi/.cache/bridget-restoration159A-preparation-20261012/T011-maintenance-diagnostic/diagnostic.json
SHA256 f44e738f0fc5a84978a5c17cc5897e8fcb1931a401965ca8cfbd6fb0aa575d91

La sauvegarde cohérente avant remplacement de production reste à créer et empreinter par ROOT. Le snapshot de préparation, même validé en isolation, ne la remplace pas. Aucun GREEN global n'est déduit. Aucune case tâche n'est modifiée par cet owner documentaire.

Les reçus exacts ci-dessus restent en CACHE privé et ne sont pas à committer. Le résumé canonique retire les corps : chaque message_bytes est remplacé par son SHA256, avec identité et phase. Les comptages, empreintes complètes des tables et comparaisons ne changent pas. Résumé public dérivé :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/validation/restauration-v7.json
SHA256 e556ba963ecd3cda48641a06789f68ee2309db29104829ce058cbcea0e88c8bb
