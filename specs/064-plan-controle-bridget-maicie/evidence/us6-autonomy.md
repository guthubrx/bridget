# Preuve US6 - autonomie gouvernee

Date de validation : 2026-08-29.

## Faits verifies

- Les usages sont attribues a execution et generation, puis agreges par source bornee. Une absence usage reste inconnue et aucun zero ne fut invente.
- Les descendants sont lus depuis ascendance durable. Duree est calculee depuis execution racine.
- Les issues paused, blocked, usage_limit, budget_limit et terminated sont distinctes.
- Continuation exige etat parent inactif, preuve inactivite fraiche, revision et generation attendues, puis reservation SQLite atomique. Execution concurrente sur cible la refuse.
- Limites delegation Maicie sont validees et persistees mais aucun etat runtime, y compris limite atteinte, ne clot ni ne rouvre mission.

## Oracles executes

```text
CARGO_TARGET_DIR=/tmp/bridget-spec064-target-2 /home/moi/.cargo/bin/cargo test -p bridget-daemon --test execution_budget_test -- --nocapture
CARGO_TARGET_DIR=/tmp/bridget-spec064-target-2 /home/moi/.cargo/bin/cargo test -p maicie --test contract execution_projection::les_faits_runtime_ne_cloturent_ni_ne_rouvrent_une_mission
```

Verdict : 2 succes Bridget et 1 succes Maicie.

## Limites nommees

Politique est implementee et testee mais reste inactive par defaut. Aucun daemon ni mission de production ne fut modifie, redemarre ou clot par cette preuve.
