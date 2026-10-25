# Archives techniques de l’adaptateur T3

Document d’origine : [specs/176-observation-bornee/validation/nettoyage-20261024.md](../../../../../../../specs/176-observation-bornee/validation/nettoyage-20261024.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Les corrections de la session 176 étaient déjà intégrées et poussées :
  code `98e187e264d7c9136896ed0803ba2b1d4327ea9b`.
- La livraison documentaire correspond à
  `78d5278d764f1a81b7a1182fec0e33f5a0f64a08`.
- La consigne T3 encore non commitée dans AGENTS.md a été enregistrée dans
  `33d0103e50cbce919d83043ed5d0adc1ac8a18a1`, puis poussée sur main.
- Les deux checkouts principaux étaient propres et alignés sur ce commit
  avant l'ajout de ce reçu.
- Deux branches distantes fusionnées ont été supprimées :
  `session-172-identifiants-t3` et
  `session-172-reconnexion-wrapper-distant`.

<a id="extrait-2"></a>

## Extrait 2

Aucun service, fournisseur, daemon ou T3 n'a été redémarré.
La projection Bridget après nettoyage indique running=true,
build_id=175-10a375e3 et 323 agents. Le lot 176 modifie le script d'observation,
pas le binaire Rust.
