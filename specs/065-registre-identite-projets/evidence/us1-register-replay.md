# Preuve US1 - enregistrement et reprise

Date: 2026-08-30

## Parcours prouvé

Le test d'intégration `spec_065_reprise_apres_crash_et_collision_alias_ne_creent_qu_un_actif`
ouvre un daemon Bridget temporaire avec une politique de racines privée. Il prouve :

1. Maicie persiste l'identité `pending_binding`, la commande et l'outbox avant toute I/O;
2. un crash après cette écriture reprend les octets canoniques exacts;
3. un crash après le commit Bridget rejoue le même `command_id` et relit l'issue durable;
4. le gagnant devient `active` avec la génération 1;
5. une seconde identité visant un lien symbolique de la même racine devient
   `registration_conflict`, avec `existing_project_id=project-winner`;
6. une seule identité Maicie est active.

## Contrats et identités

- Version de contrat registre: `1`;
- capability négociée: `project_registry_v1`;
- backend initial: `host`;
- commande gagnante: `project-command-crash`;
- commande perdante: `project-command-alias`;
- identité active: `project-winner`;
- identité en conflit: `project-loser`.

## Commandes et résultats

```text
/home/moi/.cargo/bin/cargo test -p bridget-daemon --test project_registration_e2e
1 passed, 0 failed, 0 ignored

/home/moi/.cargo/bin/cargo test -p bridget-daemon --lib spec_065_
11 passed, 0 failed, 0 ignored

/home/moi/.cargo/bin/cargo test -p bridget-transport spec_065_registre_projet_est_ferme_directionnel_et_negocie
1 passed, 0 failed, 0 ignored
```

## Comptes de lignes observés

| Fichier | Lignes |
|---|---:|
| `crates/bridget-transport/src/protocol.rs` | 4306 |
| `crates/bridget-daemon/src/daemon.rs` | 18388 |
| `crates/bridget-daemon/src/store.rs` | 4245 |
| `crates/bridget-daemon/tests/project_registration_e2e.rs` | 340 |
| `plugins/maicie/src/bridget_client.rs` | 3045 |
| `plugins/maicie/src/main.rs` | 5044 |

Les comptes sont descriptifs et ne sont pas des critères de qualité. Les
oracles sont les issues, les états et les rejeux ci-dessus.
