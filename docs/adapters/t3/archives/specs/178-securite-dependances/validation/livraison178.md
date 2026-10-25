# Archives techniques de l’adaptateur T3

Document d’origine : [specs/178-securite-dependances/validation/livraison178.md](../../../../../../../specs/178-securite-dependances/validation/livraison178.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Preuves : diaporama vérifié. Trois étapes et deux touchers relevés. Curseur et cercle orange vus sur la première action. html_preview sans erreur console ; navigation suivante/précédente, dernière étape, lecture/pause et retour à la première étape vérifiés dans le navigateur T3. Originaux, manifestes et traces restent privés.

<a id="extrait-2"></a>

## Extrait 2

- Linux : /home/moi/.local/lib/bridget-communication/178-c49d2766/bin/bridget. SHA-256 170f7c0225bee0a7eebd4703a1d2a9f5ef53f74cd740019c807b58ca0a9d8839. Core 1712494, mobile 1713392, worker 1712497 : liens /proc/PID/exe vers le nouvel exécutable.
- Mac : /Users/moi/.local/lib/bridget-communication/178-c49d2766/bin/bridget. Release 4 min 44 s, signature ad hoc vérifiée. SHA-256 459919c1e706a8d595c03e097969a0c1476037f352f2a334369e009c1a4669da. Pont 460, worker 464 ; application T3 650 et serveur 975 inchangés. Client who et pont status conformes. Les anciennes sessions MCP restent chargées ; aucun autre travail n’a été interrompu pour les remplacer.
- Caddy : /usr/bin/caddy. PID 1687157. SHA-256 aeaf63fa4c9bbfafbb9c62db6db27a59c4823b47da471855bcd024f40b13f128.
- APK signé depuis 3edd5f69 : SHA-256 8d4a872d3b9c389af7ddee7a1e7bb73daa165077edfd1cfbc8c266b2fcec4cf3. Même certificat que la version précédente : 0c43c621254d9a6e0dbe148466f453f1aaba1a62bf23bd7332c61086f16a6588. Mise à jour en place, paquet relu depuis Android et comparé byte-identique, association conservée.
