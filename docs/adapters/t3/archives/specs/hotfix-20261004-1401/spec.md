# Archives techniques de l’adaptateur T3

Document d’origine : [specs/hotfix-20261004-1401/spec.md](../../../../../../specs/hotfix-20261004-1401/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le fil `vehicules` recevait `identity_not_found` lors d’un envoi vers le fil
`Personnages`. Le même refus touchait aussi les autres fils T3.

<a id="extrait-2"></a>

## Extrait 2

Un échec temporaire de l’inventaire des fichiers ouverts supprimait tous les
marqueurs d’identité. La boucle T3 pouvait attendre plusieurs minutes avant le
cycle de reconstruction suivant. L’inventaire incluait aussi des processus
`codex app-server` internes lancés par les outils d’un agent principal.

<a id="extrait-3"></a>

## Extrait 3

- Exclure les fournisseurs internes quand un autre fournisseur T3 apparaît dans
  leur ascendance.
- Retenter une fois l’échec temporaire de l’inventaire avant toute révocation.
- Conserver la fermeture sûre après deux échecs consécutifs.
