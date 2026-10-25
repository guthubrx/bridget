# Archives techniques de l’adaptateur T3

Document d’origine : [specs/098-t3code-adapter/quickstart.md](../../../../../../specs/098-t3code-adapter/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

```sh
bridget t3 status                 # t3code détecté ? session valide ? service ? fils exposés
bridget t3 install                # session (t3 auth session issue) + service de pont ; --no-service pour un essai
bridget t3 serve                  # au premier plan, si --no-service
bridget who                       # chaque fil t3code : nom = titre, type claude|codex, transport t3code, mode cli
bridget send --to Alpha --reply -- 'Mission…'   # tour dans le fil « Alpha » ; réponse liée renvoyée par le pont
bridget attach <UUID-du-fil>      # journal du fil (sans l'historique antérieur à l'installation)
bridget t3 uninstall              # révoque la session, retire le service, efface l'état
```

<a id="extrait-2"></a>

## Extrait 2

Prérequis : t3code démarré (application, ou `t3 --mode web --no-browser <dossier>`) et CLI `t3` disponible (`npm i -g t3`).
Réglages d'essai : `BRIDGET_T3_POLL_MS` (sondage, 3000), `BRIDGET_T3_TURN_WAIT_SECS` (attente d'un fil libre, 120), `T3CODE_HOME`, `BRIDGET_T3_BIN`.

<a id="extrait-3"></a>

## Extrait 3

Tests sans t3code ni compte (environnement privé, umask 077) :
`cargo test -p bridget-daemon --features test-support --test t3code_098_test` (faux serveur HTTP + faux CLI `t3`), `cargo test -p bridget-daemon --lib spec098` (corrélation, identité, état).
