# Preuves d'implémentation

## Références

- Base : `b6eea777facf929d99a9c4f9ae75fb50e06dc2fd`
- Branche : `session-022-tests-workspace-permanents`
- Objectif : `ef7ca2f8-99da-4d69-8261-dae23286518a`
- Délégation : `1ead5568-1e42-459b-9644-cecd14acb703`

## Mesure avant correction

Hôte `cartae`, Linux `6.8.0-94-generic` x86_64, 16 CPU, 62 Gio de RAM. La
charge à une minute est restée entre 1,26 et 1,32 pendant la campagne.

| Témoin | Passés | Échoués | Taux d'échec |
| --- | ---: | ---: | ---: |
| restauration raw après EOF | 0 | 20 | 100 % |
| reconnexion à `last_seq + 1` | 0 | 20 | 100 % |
| matrice SC003 | 0 | 20 | 100 % |

## Causes et correctifs

- P1 : après fermeture du dernier maître PTY, `strace` observe `EIO` sur
  `TCSETS` puis `TCGETS`. Le banc emploie désormais Ctrl-D sur un terminal
  vivant et vérifie la restauration après la sortie logique.
- P2 : `/dev/null` produit `POLLIN` puis `read == 0`; la boucle posait alors
  `reconnect = false` avant sa garde non-TTY. Elle désactive désormais le fd
  d'entrée et poursuit l'observation socket sans boucle active.
- P3 : l'attente SC003 précédait l'ajout natif de `cursor`. Elle dérive désormais
  le catalogue de la fixture. Le mutant `lifecycle -> known_types = []` donne
  0 passé / 1 échoué, tandis que l'arbre restauré donne 1 passé / 0 échoué.

## Commits séparés

- `ebf5bf9` — banc lifecycle P3.
- `5b0f3c6` — banc PTY P1.
- `4627316` — correctif de production attach P2 et son témoin déterministe.

## Mesure après correction

Même hôte, mêmes commandes, 20 passages par témoin. La charge à une minute est
restée entre 1,22 et 1,44.

| Témoin | Passés | Échoués | Taux de succès |
| --- | ---: | ---: | ---: |
| restauration raw après EOF | 20 | 0 | 100 % |
| reconnexion à `last_seq + 1` | 20 | 0 | 100 % |
| matrice SC003 | 20 | 0 | 100 % |

## Gates

- `cargo test --no-run` : succès, avant les deux campagnes.
- `cargo test --no-fail-fast -- --quiet` : 933 passés, 0 échoué,
  16 ignorés. Aucun rouge à imputer ; les quatre bancs instables hors lot ne
  sont pas tombés pendant cette exécution.
- `cargo fmt -p bridget-daemon -- --check` : succès.
- `cargo clippy -p bridget-daemon --all-targets -- -D warnings` : succès.
- `cargo fmt --all -- --check` : échec hors périmètre sur des écarts déjà
  présents dans des fichiers Maicie non modifiés par la session.
- `--features test-support` : non mesuré ; ce gate est connu non compilable
  sous Linux à cause de `kqueue`/`kevent` sans garde de plateforme.
