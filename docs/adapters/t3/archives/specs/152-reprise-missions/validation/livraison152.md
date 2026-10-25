# Archives techniques de l’adaptateur T3

Document d’origine : [specs/152-reprise-missions/validation/livraison152.md](../../../../../../../specs/152-reprise-missions/validation/livraison152.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Élément | Preuve acquise |
|---|---|
| Git | Commit afd0f891ca4b1c03ff5e616d0e497cb1f7fc61f7, fusionné dans main et poussé vers github/main ; référence distante comparée et égale. |
| Checkout durable | main ; 30 chemins du périmètre identiques aux sources finales. |
| Release depuis main | RC0, 126,37 s ; candidat signé, vérification stricte réussie. |
| Installation | Remplacement atomique le 11 octobre 2026 à 19:03:30.627369 UTC. |
| Daemon | Ancien PID30562 remplacé par PID8185, né à 21:03:30 CEST ; Build-id afd0f891ca4b vérifié par status. |
| Bridge | Ancien PID30974 remplacé par PID9805, né à 21:03:41 CEST ; launchctl running et réenregistrement des fils observés. |
| T3 | PID60116 et60228 conservés ; naissances à 01:13:35 et01:13:36 CEST inchangées. Aucune relance T3. |

<a id="extrait-2"></a>

## Extrait 2

Binaire installé : /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget
SHA256 : dc01b1b9c884c21492f3db885de36a32f0e7847f72f884bfc6e3daeac091a0a4
Le préflight relevait 22 projets enregistrés, dont plusieurs avec des conversations actives et aucune délégation native. La consigne de préserver les autres fils et de ne pas relancer T3 est respectée.
Sauvegarde151 et copie de la DB vérifiées avant activation. Les données utilisateur restent conservées.
Sauvegardes : /Users/moi/.cache/bridget-delivery152.kzq0fhzj/bridget-before152 et /Users/moi/.cache/bridget-delivery152.kzq0fhzj/bridget-before152.sqlite
Preuves : /Users/moi/.cache/bridget-delivery152.kzq0fhzj/git-delivery152.json ; /Users/moi/.cache/bridget-delivery152.kzq0fhzj/build152.json ; /Users/moi/.cache/bridget-delivery152.kzq0fhzj/install152.json ; /Users/moi/.cache/bridget-delivery152.kzq0fhzj/backup-before152.json ; /Users/moi/.cache/bridget-delivery152.kzq0fhzj/delivery-final152.json
