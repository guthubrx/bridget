# Archives techniques de l’adaptateur T3

Document d’origine : [specs/165-cycle-vie-liens-t3/quickstart.md](../../../../../../specs/165-cycle-vie-liens-t3/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

```bash
cargo test -p bridget-daemon t3code_contract
cargo test -p bridget-daemon 't3code::tests'
```

<a id="extrait-2"></a>

## Extrait 2

1. Relever le PID exact du pont avec `launchctl print gui/$(id -u)/com.bridget.t3`.
2. Vérifier que le PID correspond à `bridget t3 serve` et non à Firefox.
3. Compter les fichiers ouverts avec `lsof -p <PID>`.
4. Vérifier le statut avec `/Users/moi/.local/bin/bridget t3 status`.
5. Vérifier dans `/Users/moi/Library/Logs/com.bridget.t3.log` :
   - présence du fil courant ;
   - absence de `Too many open files` après la ligne de démarrage ;
   - absence de refus d’authentification ;
   - nombre de liens cohérent avec les conversations principales et le travail actif.
6. Vérifier qu'un enfant actif termine, publie son dernier résultat, puis disparaît du compteur après son drain.

<a id="extrait-3"></a>

## Extrait 3

Un message Bridget direct ne redémarre pas un sous-agent arrêté. Il est refusé
jusqu'à ce que T3 réactive le sous-agent. Un sous-agent sans premier tour suit
la même règle.
