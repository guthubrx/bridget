# Implémentation — contrat client idempotent (012)

## Validation

Les commandes de validation de la session sont :

```bash
cargo test --workspace -- --test-threads=1
cargo clippy --workspace --all-targets --features test-support -- -D warnings
```

Le banc long SC-001 est une gate explicite, ignorée dans la suite courante :

```bash
cargo test -p bridget-daemon --features test-support \
  --test idempotency_crash_test -- --ignored --test-threads=1
```

Mesure de validation : **50/50 prompts uniques en 167,57 s**. Le banc couvre
les quatre frontières (avant réservation, après `Prepared`, après remise avant
issue, après issue avant accusé client), redémarre le daemon à chaque crash et
le possède dans un groupe de processus nettoyé par garde RAII.

## Critères de succès

| Critère | Preuve |
|---|---|
| SC-001 | `matrice_crash_sc001_redelivre_cinquante_prompts_uniques` : 50 cycles, quatre frontières, compteur ACP et issues rejouées. |
| SC-002 | Tests `lookup` et `retry_en_vol_rejoue_unknown_puis_accepted_apres_accuse`. |
| SC-003 | `projection_cli_et_reference_partagent_le_canon_du_daemon_reel` : corps, cible, `reply` et échéance divergents, record inchangé. |
| SC-004 (gate 012) | Même test : client socket de référence puis CLI sur le daemon réel, même issue et canon stocké. La gate MCP reste **déférée à la session 010**. |
| SC-005 | Tests `purge_uses_each_record_expiry_not_a_new_configuration` et `first_send_outside_its_horizon_is_expired`. |
| SC-006 | Tests de négociation client et non-régression workspace des chemins 007/008. |

## Dépréciations

`docs/DEPRECATIONS.md` a été relu : la session 012 n'ajoute ni ne retire de
chemin déprécié. Les envois historiques `Send` et `Register` restent
compatibles ; l'idempotence est une projection additionnelle négociée.
