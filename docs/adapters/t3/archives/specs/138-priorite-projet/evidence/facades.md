# Archives techniques de l’adaptateur T3

Document d’origine : [specs/138-priorite-projet/evidence/facades.md](../../../../../../../specs/138-priorite-projet/evidence/facades.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

L'annonce propriétaire wrapper/T3 est détaillée dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/transport-facades.md.

<a id="extrait-2"></a>

## Extrait 2

Commande : `/Users/moi/.cargo/bin/cargo test -p bridget-daemon --lib spec138_ -- --nocapture`.
Première vague intégrée : 20 PASS, 1 FAIL. Tous les tests façades et les cinq
tests wrapper/T3 passent. Le seul échec est une fixture auxiliaire du noyau
avec un issuer_scope plus court que sa borne contractuelle. Le noyau a depuis
corrigé cette fixture ; cette vague ne revendique pas un PASS global.
Log : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/facades-green-138.log.

<a id="extrait-3"></a>

## Extrait 3

L'index T3 a été corrigé sur demande du principal : une HashMap locale est
construite une fois pendant la lecture des projets. Les identifiants dupliqués
restent inconnus. Le coût ajouté est O(P+T), et non O(P×T). Les tests de
référence manquante et ambiguë restent présents et passent.

<a id="extrait-4"></a>

## Extrait 4

Le test fixe en plus BRIDGET_HOME, BRIDGET_SOCKET et TMPDIR au namespace du
harnais privé et désactive les caches Python. Il retire l'identité T3 héritée
via le véritable `bridget_background_environment` du script.
