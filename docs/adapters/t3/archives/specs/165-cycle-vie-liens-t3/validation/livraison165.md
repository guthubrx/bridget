# Archives techniques de l’adaptateur T3

Document d’origine : [specs/165-cycle-vie-liens-t3/validation/livraison165.md](../../../../../../../specs/165-cycle-vie-liens-t3/validation/livraison165.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Source validée : `3b4f101e0732cc2c857e644469707fe8b5528618`.
- Preuve runtime : `22e8ad7141509b55e85f84f506171a2550adc783`.
- Les deux commits sont intégrés et poussés sur `github/main`.
- Le worktree `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3` a été retiré sans force.
- La branche `session-165-cycle-vie-liens-t3` a été supprimée après intégration.

<a id="extrait-2"></a>

## Extrait 2

- 13 tests SPEC PASS.
- 102 tests du module T3 PASS.
- 22 tests de contrat PASS et 1 ignored.
- Suite complète : 1 944 PASS, 91 FAIL hérités, 16 ignored.
- Clippy : RC 101 sur 18 alertes héritées dans `bridget-transport`.
- `rustfmt` et `git diff --check` : RC 0.
- Contre-revue finale : APPROVE, aucun défaut bloquant.

<a id="extrait-3"></a>

## Extrait 3

- Binaire signé SHA-256 : `45bc4836c874273012af9c3fffa52d5f676a9147809737548d59120c256dc1a4`.
- Ancien pont : PID 66786.
- Nouveau pont : PID 42156, démarré le 2026-10-14 à 20:59:39 CEST.
- T3 : PID 30234 inchangé, démarré le 2026-10-14 à 15:50:32 CEST.
- Liens : 1 130 avant, 32 après.
- Fichiers ouverts : 4 539 avant, 167 après.
- Sockets Unix : 3 390 avant, 128 après.
- Le fil courant `5b199e2d-3647-416c-a622-54d7a3059f79` reste attaché.
- Aucun `auth_failed`, HTTP 401, `EMFILE` ou `Too many open files` depuis le redémarrage.

<a id="extrait-4"></a>

## Extrait 4

Trois anciens fils gardent un avertissement T3 `champ response limit`. Ce point
est distinct du cycle de vie des liens. Les compteurs de ressources restent
stables malgré cet avertissement.

<a id="extrait-5"></a>

## Extrait 5

`/Users/moi/.cache/bridget-install165.UOCBU3/com.bridget.t3.plist.before`

<a id="extrait-6"></a>

## Extrait 6

- `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/165-cycle-vie-liens-t3/validation/runtime-before.md`
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/165-cycle-vie-liens-t3/validation/tests.md`
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/165-cycle-vie-liens-t3/validation/code-review.md`
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/165-cycle-vie-liens-t3/validation/runtime-after.md`
