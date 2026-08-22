# Implementation Plan : Équipiers gérés par le daemon

**Branch**: `session-09-daemon-spawn` (après la 008) | **Date**: 2026-08-22 | **Spec**: [spec.md](./spec.md)
**Input**: spec contre-revue (3 rounds, 19 objections intégrées ; feu vert plan
conditionné au traitement explicite de deux verrous — D-502 et D-503 ci-dessous)

## Summary

Le daemon devient propriétaire du cycle de vie des équipiers : `spawn` par
ordre client à machine d'états explicite, équipiers en **groupes de processus**
marqués et réconciliables, `stop` de groupe synchrone, état désiré déclaratif
à écriture durable, reprise au démarrage. Le cœur de la réutilisation : **le
daemon lance le wrapper 007 existant** comme enfant détaché — la parité de
garanties est obtenue par construction (même code), pas par réimplémentation.

## Technical Context

**Language/Version**: Rust 2021, workspace `bridget`
**Primary Dependencies**: aucune nouvelle (spawn standard + sous-mode
`managed-bootstrap` + `setsid`/pipes via `libc` présent ; SQLite via
`rusqlite` présent — aucun fork nu)
**Storage**: état désiré `~/.config/bridget/fleet.json` (schéma versionné,
0600) ; issues de commandes dans le store SQLite existant (table bornée) ;
marqueurs de groupes `~/.cache/bridget/managed/` (0700/0600) ; stderr par
équipier `~/.cache/bridget/managed/<nom>/stderr.log`
**Testing**: `cargo test` ; tests à barrières et à points de crash ; matrice
de parité FR-008 (même corpus, deux modes)
**Target Platform**: macOS et Linux
**Project Type**: extension daemon/CLI existants
**Constraints**: zéro régression wrapper-terminal ; réconciliation avant toute
reprise ; réponse spawn/stop synchrone bornée
**Scale/Scope**: 3 modules nouveaux aux responsabilités isolées
(orchestration / processus / durabilité — découpage exigé en revue, pas de
monolithe), ~900-1200 lignes au total, 10-12 tâches prévues

## Constitution Check

| Article | Statut | Preuve |
|---|---|---|
| I — français | ✅ | messages, erreurs, StopOutcome en français |
| VII — ADR | ✅ | ADR 006 « le daemon lance le wrapper, pas l'équipier » en première tâche |
| XVIII — complexité | ✅ | réconciliation = un scan du répertoire de marqueurs au démarrage ; supervision = wait par enfant ; aucune boucle imbriquée sur collections |
| XIX — minimalisme | ✅ | zéro dépendance ; la boucle 007 du wrapper est réutilisée **avec pour seule modification le hook `managed-status`** (D-501) ; `launch_acp_with` injectable existe déjà (007-T708) ; store SQLite réutilisé pour les issues |
| XX — responsabilité | ✅ | les deux verrous de la contre-revue ont chacun leur décision (D-502, D-503) avec mécanisme et tests de crash nommés |

**Point de vigilance** : le daemon devient un superviseur de processus — chaque
promesse (« zéro orphelin coopératif », « réconciliation ») doit être
falsifiable par un test qui tue vraiment des processus. Aucun test « simulé »
pour le cycle de vie.

## Project Structure

```text
specs/009-daemon-spawn/
├── plan.md, spec.md, adversarial-review-cxbridget.md
├── data-model.md        # fleet.json, marqueurs, table d'issues, StopOutcome
├── contracts/
│   └── ordres-cycle-de-vie.md   # Spawn/StopOutcome/machine d'états sur le wire
├── quickstart.md
└── tasks.md             # après reuse-audit (sur la base 008 finale)

crates/bridget-daemon/src/
├── fleet.rs             # NOUVEAU : orchestration et invariants (machine d'états, générations, quota)
├── managed_process.rs   # NOUVEAU : bootstrap/groupes cross-platform (spawn, RELEASE, killpg, polling)
├── desired_state.rs     # NOUVEAU : lecture/écriture durable de fleet.json (schéma versionné)
├── daemon.rs            # ordres de cycle de vie, phase Recovering
├── cli.rs               # sous-commandes spawn/stop + sous-mode managed-bootstrap
├── store.rs             # table d'issues de commandes (bornée, rétention)
└── wrapper.rs           # boucle 007 réutilisée ; modifié localement pour le seul hook managed-status

docs/decisions/006-daemon-spawn.md   # NOUVEAU : ADR
```

## Approche par exigence (résumé)

| Exigence | Mise en œuvre | Fichier |
|---|---|---|
| FR-001 machine d'états | états en mémoire daemon + réservation atomique nom/slot sous le verrou d'état ; `command_id` → D-503 | `daemon.rs`, `fleet.rs` |
| FR-002 survie | enfant détaché (setsid) dans son groupe — indépendant du client | `fleet.rs` |
| FR-003 StopOutcome | handshake D-501 : ordre d'arrêt ciblé au wrapper → chemin d'arrêt complet du wrapper (annulation, drain file **et** notifications, `Unregister`) → attente bornée → escalade `killpg` si délai → réponse typée | `fleet.rs` |
| FR-005/007 échecs + stderr | stderr redirigée par équipier, rétention alignée journaux ; zéro état opérationnel sur échec (libération de réservation) | `fleet.rs` |
| FR-006 mort spontanée | `waitpid` non bloquant au tick du daemon → mêmes chemins 007 (DeliveryRejected, états) | `fleet.rs`, `daemon.rs` |
| FR-008 parité | matrice versionnée : corpus commun exécuté dans les deux modes, comparaison d'observables et de frames attach | tests dédiés |
| FR-010/010bis état désiré | D-503 ; stop-gagne par génération | `fleet.rs`, `store.rs` |
| FR-011 réconciliation | D-502 ; scan des marqueurs avant reprise | `fleet.rs` |
| FR-011bis env/cwd | politique de construction d'environnement (liste blanche daemon + registre), cwd client validé | `fleet.rs` |
| FR-011ter abonnements | `End` typé par abonnement à stop/mort (mécanisme 008 réutilisé) | `daemon.rs` |
| FR-012 quota | compteur sous le même verrou que la réservation | `daemon.rs` |
| FR-013 chemin partagé | `fleet.rs` appelle le chemin de lancement 007 (`launch_acp_with`-équivalent processus) sans dupliquer `AgentDefinition` | `fleet.rs` |

## Décisions de conception

**D-501 — Le daemon lance le wrapper 007, pas l'équipier directement.** Le
processus enfant est le **wrapper existant** (chemin de lancement rendu
injectable en 007-T708), démarré détaché dans son propre groupe. Le wrapper
garde son rôle (spawn de l'adaptateur, transport, journal, reconnexion) ; le
daemon supervise le wrapper. Parité FR-008 par construction — le code exécuté
est le même qu'en mode terminal, **au hook près** : le wrapper reçoit un petit
hook `managed-status` (round 5 — un wrapper strictement inchangé ne pourrait
rien émettre après `exec`). **Discipline des descripteurs** : le pipe
`RELEASE` est `CLOEXEC` (mort à l'`exec`, réservé au bootstrap) ; le **canal
de statut est un unique FD volontairement hérité** à travers l'`exec` par le
wrapper — tous les autres FDs de supervision sont fermés. Événements nommés
distinctement, corrélés `instance_id`/`command_id` : **`BootstrapReady`**
`{ pid, pgid, naissance }` (groupe créé, émis par le bootstrap **avant**
`RELEASE`) ≠ **`Connected`** (après le `Register` réel du wrapper) — aucune
réponse de succès n'est possible sur le seul `BootstrapReady` ; et
`StartupFailed { kind, reason }` émis par le wrapper via le hook. Le « motif
exact » de FR-005 vient de ce canal, **jamais** d'un parsing de stderr (qui
reste purement diagnostic). **Handshake d'arrêt défini** : `stop` = message d'arrêt ciblé au
wrapper → le wrapper exécute son chemin d'arrêt complet (annulation du tour,
drain de la file **et des notifications**, `Unregister`) → le superviseur
attend, puis escalade `killpg` si le délai expire. `CancelDelivery` seul ne
suffit pas. Alternative rejetée : absorber la boucle du wrapper dans le daemon
(duplication d'une machine d'états éprouvée, risque maximal pour zéro gain).

**D-502 — Verrou n°1 : marqueur durable avant exécution (bootstrap dédié,
protocole à octet).** (Réécrit au round plan : la fermeture de pipe comme
signal était ambiguë — EOF volontaire et EOF de crash sont indistinguables —
et un `fork` nu dans un daemon Rust multithread n'est pas
async-signal-safe.) Mécanisme :

1. le daemon lance **normalement** (spawn standard, pas de fork nu) un
   sous-mode exécutable minimal `bridget managed-bootstrap`. **Discipline des
   FDs** : le pipe `RELEASE` et tout FD auxiliaire sont `CLOEXEC` ; **seul le
   FD `managed-status` a `CLOEXEC` désactivé avant l'`exec`**, puis est fermé
   explicitement par le wrapper quand il n'en a plus l'usage ;
2. le bootstrap fait `setsid`, **annonce `BootstrapReady { pid, pgid,
   naissance, instance_id }`** sur le canal de statut (preuve que le groupe
   existe — jamais un succès), ferme sa copie du write-end du pipe RELEASE,
   puis **attend `read_exact(1)`** : octet `RELEASE` reçu → `exec` du
   wrapper ; **EOF avant l'octet** (crash du daemon) → `_exit` ;
3. le daemon, sur `BootstrapReady`, écrit et fsync le marqueur (`pgid`,
   naissance, `instance_id`, `command_id`, génération), **puis** écrit
   `RELEASE` et ferme.

L'ambiguïté disparaît : seul l'octet libère ; tout EOF est un abandon. Tests :
octet-vs-EOF explicites, crash aux trois frontières (avant marqueur, après
marqueur/avant RELEASE, après RELEASE), et **discipline des FDs prouvée** :
`managed-status` utilisable après l'`exec`, `RELEASE` mort après l'`exec`.

**D-503 — Verrou n°2 : idempotence ordonnée et durable, sans réponse
synthétique.** **AMENDEMENT (exigé par la 012, FR-009/D-601 — appliqué avant
génération des tasks 009)** : la 009 ne crée plus son propre mécanisme
d'idempotence — `spawn_commands` devient un **consommateur du socle 012**
(`idempotency.rs`) : la clé est (`issuer_scope` du daemon superviseur,
`operation_kind = "spawn"`, `idempotency_key = command_id`), le canon, le
rejeu, `EnvelopeMismatch` et `IdempotencyExpired` viennent du socle ;
`spawn_commands` ne conserve que l'état métier de la saga (génération, fleet,
liaison marqueur) — création et finalisation dans la même transaction que le
socle (FK). La « portée éphémère à vie du daemon » des spawns non persistants
devient une rétention courte du socle. Le reste de la décision est inchangé : `command_id` **et** `generation` figurent aussi dans `fleet.json`
et dans le marqueur. Ordre de linéarisation d'un spawn persistant : (1)
`rename` durable de `fleet.json`, (2) issue durable **après le `Register`
réel**, (3) réponse au client. Un retry du même `command_id` : issue
enregistrée → la rejouer ; entrée `fleet.json` sans issue (crash entre 1 et
2) → le retry **se rattache à la génération existante et attend son vrai
terminal** (`Connected`/`Failed`/`Cancelled`) sous le délai absolu — jamais de
`Connected` synthétisé depuis `fleet.json` (une reprise peut encore être
`Starting` ou échouer). Issue expirée par la rétention →
`IdempotencyExpired` explicite, jamais un second ordre silencieux. Une **table
de vérité** des couples (`fleet` présent/absent × issue présente/absente ×
marqueur présent/absent) avec la règle de récupération de chacun figure dans
`data-model.md` — la routine de recovery reconstruit **une seule** issue.
Éphémères : idempotence bornée à la vie du daemon, documentée. Tests : retry
après chaque point de crash (dont crash 1→2 puis reprise réussie, négociation
échouée, stop concurrent, timeout), y compris après redémarrage.

**D-504 — Réconciliation avant reprise, avec phase `Recovering` visible.** Au
démarrage : phase daemon **`Recovering`** (visible dans `who`/statut) — scan
de `managed/`, validation naissance+instance (anti-pid-recyclé), terminaison
des groupes périmés : `killpg` puis **polling borné** `kill(-pgid, 0)` (ces
processus ne sont plus nos enfants — `waitpid` rendrait `ECHILD` ; `waitpid`
est réservé aux enfants du daemon courant) ; le marqueur n'est supprimé
qu'après **disparition confirmée** ou échec terminal consigné. Puis lecture de
`fleet.json` et **réservation/enfilement de toutes les générations de reprise
dans l'ordre déterministe avant l'ouverture normale** : les `stop` sont
traités pendant l'exécution des reprises (FR-010bis), les nouveaux `spawn`
sont **refusés** avec l'erreur typée `DaemonRecovering` (règle fermée, round
5 : pas de seconde file de commandes, la réponse synchrone bornée est
préservée — le client réessaie après la phase).
Test à barrière : `stop` du deuxième élément pendant que le premier démarre.
Échec de reprise consigné, jamais de retry en boucle.

**D-505 — Environnement construit, garde appliquée à la source.** **Ordre
corrigé au round plan** : `forbidden_env` est vérifiée sur l'environnement
**source du daemon, avant toute construction** — sinon le filtrage ferait
silencieusement disparaître une clé interdite au lieu de produire le refus
T710 attendu. Puis construction : baseline portable (`HOME`, `PATH`, `USER`,
`LANG`, `TMPDIR` — sans prétendre à une allowlist universelle) + **`pass_env`
déclaratif borné par entrée de registre** pour les besoins réels des CLIs
(`XDG_*`, `CODEX_HOME`/`CLAUDE_CONFIG_DIR`, `SSH_AUTH_SOCK`, `NPM_CONFIG_*`,
proxies/certificats…), **validé par les vrais quickstarts Codex/Claude dans un
environnement nettoyé**. `cwd` = valeur absolue capturée chez le client,
validée à l'ordre et revalidée au spawn. Daemon à `PATH` minimal et `cwd`
disparu : échecs typés testés.

**D-506 — `stop` résolu par la table superviseur, marqueurs en secours.**
(Corrigé au round plan : une résolution par les seuls marqueurs raterait un
équipier en `Reserved`/`Starting` avant marqueur durable, et perdrait
stop-gagne-sur-reprise.) Résolution **primaire** : la table superviseur en
mémoire `nom → génération` couvre `Reserved`/`Starting`/`Running` ; le `stop`
**invalide d'abord la génération**, puis agit sur le groupe s'il existe. Les
marqueurs `managed/` restent la preuve du groupe `Running` et le **secours
après crash**. Un nom wrapper-terminal n'apparaît dans aucune des deux sources
→ refus structurel. Tests : stop avant marqueur, pendant bootstrap bloqué,
après `Register`, sur marqueur périmé.

## Complexity Tracking

Aucune violation : zéro dépendance, trois modules nouveaux aux
responsabilités isolées, réutilisation du wrapper/du store/du registre/des
variantes de protocole. Le mécanisme spawn standard + sous-mode
`managed-bootstrap` + protocole à octet (D-502) est la primitive la plus basse
qui satisfasse l'invariant de couverture sans fork nu — un superviseur externe
(launchd/systemd) aurait été plus lourd et non portable.
