# Archives techniques de l’adaptateur T3

Document d’origine : [specs/146-bridget-panel-lisible/validation/integration-receipt.md](../../../../../../../specs/146-bridget-panel-lisible/validation/integration-receipt.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Build frais web privé25,5 secondes et serveur892 ms passent. Les6 sources/tests du delta T3 correspondent aux hashes finaux gelés ;2219 fichiers client sont comparés sans différence. Deux changements UI concurrents ont invalidé les premières validations ; ces artefacts antérieurs ne sont pas déclarés finaux.

<a id="extrait-2"></a>

## Extrait 2

Interop réelle hors réseau :8 enveloppes JSON produites par Rust sont décodées par les vrais schémas T3. Elles comprennent liste récente, détail145 et6 pages d'historique (24 ×5 puis17 messages), de séquence137 à1. Les137 corps sont comparés à une source indépendante ; snapshot 137 reste inchangé et exclut l'ajout138. Export Rust : `/Users/moi/.cache/bridget-build-146.EFWCYG/interop-146.json`.
