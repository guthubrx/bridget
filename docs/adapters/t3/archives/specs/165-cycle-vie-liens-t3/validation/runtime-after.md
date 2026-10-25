# Archives techniques de l’adaptateur T3

Document d’origine : [specs/165-cycle-vie-liens-t3/validation/runtime-after.md](../../../../../../../specs/165-cycle-vie-liens-t3/validation/runtime-after.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Ancien pont : PID 66786, démarré le 2026-10-14 à 19:23:19 CEST.
- Nouveau pont : PID 42156, démarré le 2026-10-14 à 20:59:39 CEST.
- T3 : PID 30234, démarré le 2026-10-14 à 15:50:32 CEST avant et après l'activation.
- T3 n'a pas été relancé.
- Seul le service `com.bridget.t3` a été retiré puis chargé avec `launchctl`.
- L'override `SoftResourceLimits.NumberOfFiles=16384` a été retiré du plist.
- Le journal confirme `RLIMIT_NOFILE: soft=4096` au démarrage du nouveau pont.

<a id="extrait-2"></a>

## Extrait 2

- `/Users/moi/.local/bin/bridget t3 status` rend `running`.
- Aucun `auth_failed`, HTTP 401, `EMFILE` ou `Too many open files` n'apparaît
  depuis le démarrage du PID 42156.
- Trois anciens fils produisent encore l'avertissement HTTP T3 `champ response
  limit`. Cet avertissement existait hors du cycle de vie des liens. Il ne fait
  pas croître le nombre de fichiers ouverts.
- Le binaire installé est signé et sa vérification stricte réussit.

<a id="extrait-3"></a>

## Extrait 3

`/Users/moi/.cache/bridget-install165.UOCBU3/com.bridget.t3.plist.before`
