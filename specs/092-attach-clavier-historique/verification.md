# Vérification indépendante 092

Le pilote contrôle le travail de l'équipier ; il n'a pas modifié le code Rust.
Environnement de validation : Rust 1.92.0, rustfmt 1.8.0, PATH préfixé par
`/Users/moi/.cargo/bin`, TMPDIR privé `/tmp/b92.LVqy1o`, cache debug partagé
`/Users/moi/Nextcloud/10.Scripts/64.bridget/target`.

## Itérations ciblées réellement exécutées

| Commande Cargo | Résultat | Durée rapportée par Cargo |
| --- | --- | --- |
| `test -p bridget-daemon --lib attach::tests::spec092 -- --nocapture` | 10 réussis, 1 échec BrokenPipe dans le nouveau test : pair socket détruit | compilation 8,78 s ; tests 0,06 s |
| `test -p bridget-daemon --lib attach::tests` | 77 réussis, même échec ; anciens tests préservés | compilation 1,72 s ; tests 0,76 s |
| `clippy -p bridget-daemon --lib -- -D warnings` | deux erreurs : 8 arguments de drive_interactive et emprunt superflu | non mesurée |
| `test -p bridget-daemon --lib attach::tests` après corrections fonctionnelles | 82 réussis, 0 échec, dont 15 spec092 | compilation 3,62 s ; tests 0,87 s |

Les corrections ont été commandées à l'équipier, pas appliquées par le pilote.
Les trois témoins supplémentaires de restauration et le regroupement du contexte
d'entrée sont postérieurs à cette dernière passe : elle n'est donc pas présentée
comme la validation finale de ces derniers hunks.

## Limites de preuve à conserver

- Le rouge initial TDD de Shift+Entrée n'a pas été observé avant implémentation :
  la compilation de l'équipier était bloquée par son TMPDIR. Les rouges ci-dessus
  sont réels, mais ne remplacent pas rétroactivement cette étape procédurale.
- Les séquences sont injectées dans des pseudo-TTY réels et dans le gestionnaire
  d'entrée ; aucune frappe physique dans le terminal de l'utilisateur n'est attestée.
- Le test nommé `apres_eof` injecte Ctrl-D (EOT), distinct d'un `read == 0`.
  Le scénario préexistant `pseudo_tty_polin_hup_livre_le_dernier_send_et_restaure_le_terminal`
  exerce, lui, la fermeture réelle de l'entrée avec le chemin `drive_interactive`.
- La revue est indépendante de l'auteur du code, mais pas cross-provider : aucun
  autre fournisseur n'était disponible dans l'annuaire.
- L'historique représente des saisies écrites sur la connexion, pas des messages
  durablement livrés. Les refus tardifs ne réécrivent pas ce passé local.

## Consolidation finale

Diff figé et contrôlé le 2026-09-06 vers 14:35–14:39 CEST.

| Commande | Résultat | Durée |
| --- | --- | --- |
| `cargo fmt --all --check` | succès, exit 0 | 0,78 s outil |
| `cargo clippy --workspace --all-targets -- -D warnings` | succès, aucune erreur/alerte | 7,18 s Cargo |
| `cargo test --workspace --quiet` | 1 218 réussis, 0 échec, 47 ignorés | 187,54 s cumulées des suites ; compilation et attente outil exclues |
| `cargo test -p bridget-daemon --lib attach::tests::spec092 --quiet` | 18 réussis, 0 échec | 0,06 s tests |
| `cargo build --release -p bridget-daemon --bin bridget` | succès | 42,66 s Cargo |
| `./target/release/bridget --help` | succès, aide rendue, exit 0 | moins d'une seconde |
| `git diff --check` | succès, exit 0 | moins d'une seconde |

Les deux résultats imbriqués de sous-processus fsutil (1 test chacun, 264 filtrés)
sont exclus du total 1 218 pour ne pas compter deux fois le même scénario parent.
Les 47 ignorés ne sont pas présentés comme passés ; aucune gate fournisseur
facturée ou manuelle ignorée n'a été forcée pour ce changement local.

Pour le release, CARGO_TARGET_DIR est distinct du binaire installé :
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/091-communication-agent-ux/target`.

- Source attach.rs SHA-256 inchangé entre vérification et build :
  `acbc3e690457a27a933e3ab174d6a33dd278efca6894722fb9a881bec22e3f31`.
- Candidat `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/091-communication-agent-ux/target/release/bridget` :
  `43296aefe68aa137a97390c870292ec29c7800ada709e09e2f1f0aae6db1d1fa`.
- Binaire installé `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget`, conservé :
  `f87be334b377bfadc57f214de9d918f58c97e7ec2f61c53fc47d382de9dbee16`.

Aucun daemon redémarré, aucun commit ni merge effectué pendant cette session.
Les résultats locaux historiquement bloqués de implementation.md sont complétés
par ce relevé indépendant, et non effacés.

Audit final local : audits/2026-09-06/session-2026-09-06-spec-092-01.
`python3 /Users/moi/Nextcloud/10.Scripts/00.Generic/audit-code-v14/scripts/validate_session.py audits/2026-09-06/session-2026-09-06-spec-092-01`
retourne exit 0, 0 erreur, 0 warning. La notation est limitée au diff, pas au
produit entier. Les cases tasks.md ont été closes seulement après convergence
et validations ; l'écart procédural T001 est conservé explicitement.

## Mise en service explicitement demandée ensuite

Le 2026-09-06, après validation, l'utilisateur a demandé l'installation.
Le candidat a remplacé atomiquement le binaire installé, sans redémarrer le
daemon ni les agents. `/Users/moi/.local/bin/bridget` conserve son symlink vers
`/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget`.
L'empreinte vérifiée via ce symlink est celle du candidat :
`43296aefe68aa137a97390c870292ec29c7800ada709e09e2f1f0aae6db1d1fa`.
`bridget version` termine avec exit 0. Un attach déjà ouvert doit être quitté
puis rouvert pour exécuter cette version ; les agents restent vivants.

Ancien binaire sauvegardé dans
`/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/.install-092.3AfYKQ/bridget.previous`.
L'installation n'a pas committé ni fusionné la branche source 092.
