# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/disconnect-guidance.md](../../../../../../../specs/174-audit-incidents/validation/disconnect-guidance.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Défaut confirmé par lecture : le client décodait Disconnect comme une réponse
normale, puis la classait daemon_protocol. Le protocole décrit cette trame
comme l’arrêt du démon. Le pont T3 de Bridget marque déjà ce lien inactif.
Les incidents signalés coïncident avec l’activation du maître ; leurs deux
tâches sont ensuite result_available, sans erreur. La trame réelle de ces
incidents n’a pas été capturée : cette corrélation ne constitue pas cette preuve.

<a id="extrait-2"></a>

## Extrait 2

Le conseil de rattachement T3 distingue lien absent/inactif ou transport
indisponible des preuves invalides. Une nouvelle attestation ne prouve pas la
reconnexion du lien. Aucun droit, secret ni identité n’est assoupli.

<a id="extrait-3"></a>

## Extrait 3

Aucun service ni modèle redémarré : maître, pont, processus T3 principaux et
worker conservés. Un processus de rendu T3 a disparu pendant la compilation,
sans signal envoyé par cette livraison ; ne pas prétendre que tous les
processus de l’application sont restés identiques. Les deux façades MCP des
parents incidents restent ouvertes avec leur ancien code. Le correctif client
prendra effet à leur rechargement ; installer le fichier ne prouve pas cette
activation. Le worker conserve également son exécutable chargé.
