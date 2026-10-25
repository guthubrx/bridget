# Archives techniques de l’adaptateur T3

Document d’origine : [specs/177-bridget-mobile/research.md](../../../../../../specs/177-bridget-mobile/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Base Git Mac et Cartae : be00ace5c2dca3126e420214ec248c54083440d1, main propre.
- docs/federation-services.md décrit un maître unique, un tunnel SSH et aucune réplication.
- daemon.rs:18115 : la garde de lecture humaine exige une liaison à une conversation de bureau et une présence locale.
- referent_control.rs:24–38 : un principal administratif humain local possède une autorité différente d'un agent. Le modèle de menace admet les processus du même compte local.
- daemon.rs:13341 : annulation humaine existante avec contrôle d'acteur, identifiant de demande et human_cancel.
- communication/client.rs:15 : client Unix borné ; ne pas en créer un second.
- environment.rs:18 : validation du namespace et des fichiers privés.
- t3code_mcp.rs:39 : la réattestation consulte un runtime local et un endpoint validé. Un déplacement de maître ne peut pas être déclaré transparent sans adaptation ou preuve.
- Cartae utilise Caddy sur 80/443. Le domaine racine cartae.app a une entrée A vers ce serveur ; bridget n'a pas encore d'entrée dédiée.
- Aucun JDK, Gradle ou adb trouvé dans le PATH du serveur. L'outillage Android devra être installé dans un emplacement identifié, puis réutilisé.
- setup-plan.sh n'existe pas à l'emplacement fourni par la primitive. Le protocole de planification est exécuté directement ; ce constat ne dispense d'aucun artefact.
