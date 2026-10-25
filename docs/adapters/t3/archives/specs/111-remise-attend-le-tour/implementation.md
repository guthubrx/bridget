# Archives techniques de l’adaptateur T3

Document d’origine : [specs/111-remise-attend-le-tour/implementation.md](../../../../../../specs/111-remise-attend-le-tour/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- `t3code.rs` : la condition `received.elapsed() >= turn_wait` est retirée des deux points de
  filtrage. Les autres motifs, tous fondés sur une échéance portée par le message, sont inchangés :
  échéance métier, délai de réponse, expiration de saga, demande suivie close.
- Nouvelle borne `QUEUE_BOUND = 64` appliquée à l'insertion. Un dépassement écarte la plus ancienne
  avec un avertissement distinct, pour ne pas confondre saturation et péremption.
- `turn_wait` reste employé par le cache des annulations, dont la sémantique est différente.
