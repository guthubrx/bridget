# Archives techniques de l’adaptateur T3

Document d’origine : [specs/159-edition-file/validation/livraison159B.md](../../../../../../../specs/159-edition-file/validation/livraison159B.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Build RC0 en 121.66 s, signature vérifiée. Binaire installé :
/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget
SHA256 b34fa7171d697b708e43eb802df9f106450b0f12e1f0a4eaa0a4b00d85d29304
Activation attestée à 2026-10-12T22:16:48.203247+02:00. Daemon 34126 né le
Mon Oct 12 22:16:41 2026 ; pont 34291 né le Mon Oct 12 22:16:44 2026.
Zéro mission native non terminale avant remplacement et chaque SIGTERM.
T3 60116/60228 et leurs naissances restent inchangés. Schéma7, quick_check=ok,
aucune violation FK. La sauvegarde privée 159A et le binaire rollback sont
conservés ; aucun snapshot ni corps privé n’est inclus dans ce reçu.

<a id="extrait-2"></a>

## Extrait 2

held_queue reste fermé. SC004 B : proxy p95 41,385→1026,412 ms, pas mesure du
repos du daemon. La reprise cancel est pausée ; SC007 commence à Applied.
La promotion prouve le commit, pas l’injection, avec garanties réduites après
libération. Aucune certification T3 ni modèle réel déduite.
