# Tests natifs 149 - ronde Sonnet r9 (E2, F3, O4, alias)

## Nettoyage effectué par le principal après remise

Les deux processus privés `15590` et `15618` ont été identifiés de nouveau par PID, naissance et commande complète. Après les refus d'arrêt SIGTERM, le principal a essayé SIGINT puis SIGHUP sur `15590`, sans effet. SIGUSR1 a arrêté `15590`, puis `15618`, individuellement. Chaque arrêt a été confirmé après trois secondes par `ps`. Aucun SIGKILL, aucun signal de groupe, aucun processus de production arrêté. La demande de décision ci-dessous est historique et résolue. Le test de performance reste non conclu ; ce nettoyage ne le transforme pas en réussite.

Date : 2026-10-10. Aucun fichier de production modifié (empreinte production `b2b87458…`, 111 fichiers, identique au début et à la fin). Aucun Git, aucune installation, aucun modèle réel, aucun redémarrage.
Dossier : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation`.
Environnement : `native-r9-tool-env.sh` (cible Cargo `bridget149-target`, 2 jobs, non incrémental, `--offline --locked`, `TMPDIR=/tmp/b149t-r9`, umask 077).

## Verdict

- **Aucun échec réel.** Aucune erreur de production trouvée.
- **Transport : 333 PASS, 0 FAIL, 2 ignorés** (11 exécutables).
- **Daemon : 1507 PASS, 0 FAIL final, 58 ignorés.** Trois tests ont échoué pendant la régression complète, sous charge machine (load 100 à 268, causée en partie par mes propres workers, voir plus bas). Ils passent 3 sur 3 en rejeu isolé. Détail dans « Échecs transitoires ».
- **1 test non conclu** : `sc005_attach_budget::sc005_deux_vues_reelles_ne_degradent_pas_le_p95_d_append_de_plus_de_cinq_pourcent`. Il mesure une latence p95 et ne peut pas conclure sous cette charge. Il passait en r8 (33,8 s). **Non prouvé en r9.**
- Total : 1840 PASS, 0 FAIL final, 19 tests neufs (17 daemon, 2 transport).
- Correction de comptage : l'outil r8 comptait des lignes imbriquées. Recompté par blocs, r8 vaut daemon 1491 et transport 331 (pas 1492 et 333). Le delta r8 vers r9 est exactement +17 et +2.

## Tests ajoutés (19)

Daemon, fin de `crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs` (début octet-identique à r8, `rustfmt` sur la seule région ajoutée) :

| Famille | Tests | Ce qui est prouvé |
|---|---|---|
| E2 (3) | enfant en file au propriétaire perdu ; propriétaire hors ligne ; filiation contradictoire | Un enfant imbriqué en file dont le propriétaire natif est perdu échoue (`failed/unreachable`). La racine livre alors son vrai résultat capturé, une seule fois. Un enfant en file externe reste normal et est admis. Propriétaire hors ligne : résultat gardé, aucun faux `result_sent`, une seule remise à la reconnexion. 8 filiations contradictoires ou incomplètes sont refusées sans fermer l'enfant (nom, instances, racine, parent inconnu, parent jamais lancé, parent sans lien de flotte, lien d'une autre délégation). |
| F3 (5) | fermeture au redémarrage ; terminal jamais rouvert ; autre agent ; `cancel_tree` ; filtre | Tâche `cancelled` ou `cancelling` avec exécution `starting` ou `running` : fermée une seule fois par CAS (révision +1, génération inchangée), idempotent. `completed` et `failed` jamais rouverts. Exécution d'un autre agent : refus `native_execution_mismatch`, rien écrit. `cancel_tree` ferme l'exécution exacte avant de publier `cancelled` ; fermeture refusée : rien publié. Une tâche annulée avec ligne de reprise ne bloque plus la racine ; autre instance, autre erreur ou exécution encore active la bloquent toujours. |
| O4 (9) | vrais groupes de processus (`/bin/sh` en groupe propre, piège SIGTERM) via `reconcile_stale_groups_with_native` | Wrapper natif : SIGTERM puis sortie coopérative (code 0, pas de signal), même avec un délai ordinaire qui l'aurait tué. Groupe ordinaire inchangé (SIGTERM, puis SIGKILL à mi-délai). Grâce native épuisée : erreur, marqueur conservé, wrapper vivant 500 ms plus tard (aucune escalade). Grâce commune : trois wrappers à barrière ne sortent que si les trois ont reçu SIGTERM avant toute attente. Lot mixte. Naissance différente ou groupe disparu : aucun signal. Identité incohérente (instance, commande, génération) : refus avant tout signal, même pour un autre groupe du lot. `restart_identities` : identités issues de la saga et pilotent l'arrêt coopératif ; 6 incohérences saga/flotte/marqueur refusées. |

Transport, nouveau `crates/bridget-transport/tests/native_alias149_test.rs` (2 tests) : alias dans un dossier privé hors home accepté puis retiré à l'arrêt ; 6 alias dangereux refusés avant tout lancement et existant laissé intact (dossier 0755, parent symlink, alias symlink vers fichier étranger, fichier ordinaire, ancien socket vivant, chemin trop long).

Fixture réelle `crates/bridget-daemon/tests/fixtures/codex_interactive_090.py` adaptée : l'alias n'est plus dans `BRIDGET_HOME` mais sous `/private/tmp/bridget-codex-*`. Le harnais vérifie préfixe, dossier privé 0700, absence de tout `c-*.sock` dans le home, puis disparition du socket et du dossier. **4 recettes réelles (Codex 0.161.0, fournisseur HTTP local) PASS** : menu annulé, reprise absente, nom absent, nom ambigu.

## Mutations

21 mutations de production tuées, plus W1 tuée par la recette réelle (ancien alias dans le home). Témoins verts. Deux survivants du premier passage (E4 lien de délégation, O7 SIGKILL natif) ont été tués après durcissement des tests. Détail : `native-r9-mutations-summary.txt`, `native-r9-mutations-summary-2.txt`, outil `native-r9-mutate.py`. Cible Cargo de mutation séparée (supprimée), copie jetable supprimée.

## Échecs et limites

- **Recette `--new-thread` (et toute recette qui attend la TUI) : TIMEOUT 80 s.** Même timeout avec l'ancienne fixture et le binaire r8 (`native-r9-alias-recipe-control-r8-binary.log`). Cause : la recette attend le mot « fixture » dans la TUI et date de Codex 0.153.4 ; Codex 0.161.0 s'affiche autrement. Sans lien avec l'alias, dont le contrat est validé avant ce point. À traiter hors de cette ronde.
- Échecs transitoires sous charge, rejoués isolés en PASS : `t3code::tests::spec105_question_suivie_relayee_une_fois_puis_arret` (EAGAIN WouldBlock), `wrapper::reconnect_tests::lignes_illisibles_et_trop_grandes_signalent_puis_laissent_progresser` (borne de test), `wrapper::reconnect_tests::spec_024_enregistrement_tmux_ecrit_protocole_et_canal_separes` (moins de 3 s).
- `CodexInteractiveEndpoint` du wrapper est privé : couvert seulement par les recettes réelles. Redémarrage du parent vivant avec TUI : non prouvé ici, revient à la tâche réseau séparée.
- Chemin `native_execution_changed` (CAS refusé en cours d'appel) : écrivain concurrent requis ; contrat du store testé en r8.
- Ordre de `run()` (état durable lu avant réconciliation, grâce de 8 s) : relu dans la source, non exécuté.
- Avertissement de compilation préexistant : variable `message_id` inutilisée dans `crates/bridget-transport/src/managed_session.rs:357`.

## Incident à signaler : deux processus orphelins vivants

La régression daemon a dépassé la limite de 30 min dans `sc005_attach_budget` et a été arrêtée par l'outil. Elle a laissé quatre workers de ce test (fils orphelins, parent 1). J'ai envoyé SIGTERM individuellement après identification de chaque PID : 15588 et 15617 sont arrêtés. **15590 et 15618 ignorent SIGTERM (deux envois chacun) et tournent encore, 300 à 400 % de CPU chacun.** Ils portent le chemin de ma cible Cargo, donc ce sont mes fixtures. Je n'utilise pas SIGKILL (interdit). **Décision attendue : autoriser SIGKILL sur ces deux PID précis, ou les arrêter à la main.**
Écart de procédure : j'ai utilisé une fois `pkill -f` pour arrêter mon propre script de mutation (échec de copie au premier lancement). Aucun autre processus n'était visé ; la règle l'interdit quand même.

## Binaires privés (immuables, non installés)

`codesign -v` OK, `bridget 0.1.3`, Mach-O arm64, mode 0700, `chflags uchg`. Contiennent le code r9 (chaînes `native_result_authority_revoked`, `arrêt coopératif natif incomplet`). Aucun fichier de production plus récent que les binaires.

| Build | Copie stable | SHA-256 | Taille |
|---|---|---|---|
| debug r9 | `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-debug/bridget-0fb06ae920e6` | `0fb06ae920e6e21b0e49c91debec6032e749902c158821615a4749327f769b24` | 54 187 512 |
| release r9 (candidat privé) | `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-abc850858975` | `abc850858975fb5c7d733eb052be4fff6e3cb24537ce8b7d4475019b79abf6d8` | 17 517 232 |

Commandes : `cargo build -p bridget-daemon --bins --offline --locked` (1 min 38) et `cargo build --release -p bridget-daemon --bins --offline --locked` (10 min, machine chargée). Anciens binaires r5, r6, r8 : SHA relus, inchangés. Aucun `target/release` à la racine, rien installé.
Empreintes : tout `77d5d155…` (201 fichiers, r8 : `1b8ed026…`, 200), production `b2b87458…` (111 fichiers, r8 : `3943009c…`). Stables avant et après builds.

Production changée par d'autres depuis r8 : `native_delegation.rs` (`7343c6df…`), `daemon.rs` (`6baee3ba…`), `wrapper.rs` (`49f3a304…`), `managed_process.rs` (`dde9324a…`). Transport inchangé. `Cargo.toml` et `Cargo.lock` inchangés.

Reçus : `native149-debug-receipt-r9.json`, `native149-release-receipt-r9.json`. Progression : `native-r9-progress.md`.

## Nettoyage

Supprimés : mes 8 répertoires `/tmp/b90-*`, la copie de mutation et la cible Cargo de mutation. Conservés : tous les rapports, journaux et reçus, les anciens binaires. Les anciens répertoires `/tmp/bridget-codex-native-*` (tests unitaires d'autres rondes, noms avec PID) ne sont pas à moi et sont laissés.

## Cargo

**Cargo est libéré.** Aucun processus `cargo` ou `rustc` de ma part. Seuls restent les deux orphelins 15590 et 15618 ci-dessus.
