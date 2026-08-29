# Preuve US3 - reprise et bifurcation

Date de validation : 2026-08-29.

## Faits verifies

- CodexAppServer emet thread/start avec threadId pour la reprise native et threadId plus fork true pour la bifurcation. Les deux formes sont construites uniquement si la capacite observee les annonce.
- Une version ou une capacite incompatibles refusent operation native. Elles ne recoivent jamais une etiquette native par defaut.
- Le fallback existant reprise.rs cree une continuation reconstructed avec raison attestee, et conserve ascendance parent, generation et binding.

## Oracles executes

```text
CARGO_TARGET_DIR=/tmp/bridget-spec064-target-2 /home/moi/.cargo/bin/cargo test -p bridget-transport reprise_et_bifurcation_codex_emettent_la_requete_attestee_ou_refusent --lib
CARGO_TARGET_DIR=/tmp/bridget-spec064-target-2 /home/moi/.cargo/bin/cargo test -p bridget-daemon --test execution_resume_test -- --nocapture
```

Verdict : 1 succes pour la requete Codex attestee et 2 succes pour la persistance, ascendance et fallback declare.

## Limites nommees

La preuve exerce contrat et adaptateurs de fixture. Aucun fil reel agent existant ni daemon de production ne fut pilote. Le test binaire Codex reel de SPEC-063 reste une preuve separee de consommation de clientId.
