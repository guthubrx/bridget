# Archives techniques de l’adaptateur T3

Document d’origine : [audits/2026-10-06/session-2026-10-06-spec-136-01/01_Decouverte.md](../../../../../../../audits/2026-10-06/session-2026-10-06-spec-136-01/01_Decouverte.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Découverte et limites
Rust + serde/rusqlite ; harnais Python externes existants. Pas d'UI dans ce
périmètre, pas de conteneur, pas de workflow CI repéré. Cargo.lock conservé.
La page T3 est une interface externe, non auditée ici. SLO proposé, non mesuré.
Baseline absente = vide. Première session pour le scope136.
Couverture de lignes et CVE non mesurées ; cargo-llvm-cov absent, cargo audit
non exécuté (scan CVE non requis par le diff sans dépendance).
L'analyse des migrations et des contrôles d'identité comprend une seconde
lecture ciblée. Aucun fichier de production modifié pendant l'audit.
