# Archives techniques de l’adaptateur T3

Document d’origine : [specs/165-cycle-vie-liens-t3/adversarial-review.md](../../../../../../specs/165-cycle-vie-liens-t3/adversarial-review.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Niveau | Finding | Preuve | Correction du plan |
|---|---|---|---|
| Bloquant | Le filtre initial fermait le lien avant le `Tick` terminal. Le dernier texte, `turn_end` et la corrélation pouvaient donc manquer. | `crates/bridget-daemon/src/t3code.rs:1187`, `:2351`, `:2999` | Garder le lien existant pendant un drain lié à `change_key()`. |
| Bloquant | Une réponse persistée mais non acquittée ne serait jamais réémise après redémarrage. | `crates/bridget-daemon/src/t3code.rs:217`, `:1690` | Scanner une fois les états enfants terminaux et rouvrir ceux qui portent une obligation. |
| Majeur | Joindre un lecteur sans fermer la socket depuis le propriétaire peut bloquer si le worker panique. | `crates/bridget-daemon/src/t3code.rs:1381`, `:1389`, `:1611` | `Link` garde un clone de fermeture. Il ferme le flux avant de joindre le lecteur. |
| Majeur | Filtrer `drive_queue()` laisserait une remise bloquée sans rapport. | `crates/bridget-daemon/src/t3code.rs:2018` | Ne pas modifier la règle de `drive_queue()`. Une file non vide interdit le drain. |
| Majeur | Cinq lectures du cas silencieux peuvent dépasser dix secondes. | `crates/bridget-daemon/src/t3code.rs:59`, `:3022` | Mesurer la fermeture après quiescence. Utiliser `turn_wait` comme borne dure. |
| Mineur | Le champ de filiation n'est pas encore porté par l'adaptateur. | `crates/bridget-daemon/src/t3code_contract.rs:252` | Présenter ce champ comme une extension neuve du contrat interne. |
