# Archives techniques de l’adaptateur T3

Document d’origine : [specs/172-resolution-projet-distant/validation/reconnexion-wrapper-20261022.md](../../../../../../../specs/172-resolution-projet-distant/validation/reconnexion-wrapper-20261022.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Seul le worker est remplacé, KillMode=process et SendSIGKILL=no. Le maître82087,
le pont898 et T3(28528/28591) gardent PID et date de naissance ; ASAR inchangé.
Le processus distant déjà actif conserve PID2027384 et naissance217334127.
Son journal avant/après livraison conserve la sessionclaude-stream-json-2027660.
Les wrappers antérieurs gardent leur ancien binaire jusqu'à leur fin ; le maître
n'est pas redémarré pour cette livraison. Les nouveaux wrappers utilisent le build corrigé.
