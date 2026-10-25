# Archives techniques de l’adaptateur T3

Document d’origine : [specs/111-remise-attend-le-tour/plan.md](../../../../../../specs/111-remise-attend-le-tour/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Dans `crates/bridget-daemon/src/t3code.rs`, retirer la condition `received.elapsed() >= turn_wait`
des deux points de filtrage de la file. Les autres motifs d'écartement, tous fondés sur une échéance
réellement portée par le message, restent inchangés.

<a id="extrait-2"></a>

## Extrait 2

Unitaires dans `t3code.rs` : une remise sans échéance survit à une attente supérieure à l'ancien
délai ; une remise avec échéance dépassée est écartée ; la borne de file écarte la plus ancienne.
