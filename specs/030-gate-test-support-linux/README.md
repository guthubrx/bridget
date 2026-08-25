# Session 030 — Gate `test-support` jouable sous Linux

**Base** : `29824c8` (`origin/main`)  
**Objectif** : `82de1383-1f49-4980-97af-30cf1253c4bc`  
**Délégation** : `484eab09-96b0-4239-8d58-a4b7799d9f86`  
**Constat** : `constat/le-gate-test-support-est-injouable-sous-linux-et-aveugle-la-moitie-de-la-flotte`

## Décision (mesurée)

1. **Garde fine** autour de `watch_marker` (kqueue Darwin/BSD), pas exclusion du fichier.
2. **Pas d'équivalent inotify** dans ce lot (autre périmètre / autre revue) — à proposer séparément.
3. Les **7** tests kqueue-dépendants portent `#[cfg_attr(target_os = "linux", ignore = …)]` — visibles, pas silencieux.
4. Les **5** tests sans kqueue restent exécutables sous Linux avec `--features test-support`.
5. Stub hors Darwin/BSD : panique qui **nomme** la condition (kqueue requis, ignore attendu, fichier:ligne).

## Hors machine

Auteur sur macOS : la preuve `--no-run` sous Linux est à jouer par un agent cartae avant jury. Non mesurée ici.
