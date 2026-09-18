# Recherche 107 — Journal WAL

## Faits observés (2026-09-18)

- Production : `sqlite3 -readonly ~/.cache/bridget-core/bridget.db "PRAGMA journal_mode"` → `delete` ;
  `page_size` 4096, 11 659 pages (≈ 47 Mo).
- Aucune occurrence de `journal_mode`, `synchronous` ou `-wal` dans `crates/*/src` (hors scripts de test
  Python `scripts/test-bridget-idle.sh` qui créent une fixture WAL pour un autre usage).
- Un seul fichier de base, quatre ouvreurs en écriture dans le processus daemon :
  `Store::open` (`store.rs:65`, premier ouvert au démarrage, `daemon.rs:2919`), `IdempotencyStore::open`
  (`idempotency.rs:471`), `ExecutionStore::open` (`execution_store.rs:213`), `AgentProfileStore::open`
  (`agent_profile.rs:174`, à la demande), `ArtifactStore::open` (`artifact_store.rs:179`, par requête).
  Lecteurs en lecture seule : `store_schema::validate_existing` (préflight), `Store::open_read_only`
  (104), tests de barrière (`daemon.rs:1489`, `:9922`).
- Tests simulant la contention : écrivain-écrivain via `BEGIN IMMEDIATE` (`store.rs:513`,
  `daemon.rs:9416`) — inchangés en WAL ; lecteur bloqué via `BEGIN EXCLUSIVE`
  (`tests/search_104_test.rs:997`) — à adapter.
- `scripts/deploy-remote.sh` exclut `*.db` de la synchronisation ; `bridget.db-wal`/`-shm` ne
  correspondent pas à ce motif : ajouter `*.db-*`.

## Alternatives

| Option | Verdict |
|---|---|
| `PRAGMA journal_mode=WAL` posé par `Store::open` (premier ouvreur), persistant dans le fichier | **Retenu** : une ligne de contrat, propriété du fichier, héritée par toutes les connexions |
| Poser le pragma dans chaque ouvreur | Rejeté : redondant (persistant), 4 modules touchés pour rien ; à revoir seulement si un ouvreur peut précéder `Store::open` sur une base neuve en production (ce n'est pas le cas : `daemon.rs:2919`) |
| `synchronous=NORMAL` en plus | Hors périmètre : gain d'E/S mais dernière transaction perdue sur coupure de courant ; 099 promet la durabilité des remises |
| Augmenter `busy_timeout` des lecteurs | Rejeté : masque la cause, allonge la latence |

## Sémantique SQLite utile (documentation officielle, sqlite.org/wal.html, consultée de mémoire et vérifiée par test)

- WAL : les lecteurs ne bloquent pas l'écrivain et l'écrivain ne bloque pas les lecteurs ; un seul écrivain
  à la fois (`BEGIN IMMEDIATE` conserve l'attente bornée entre écrivains).
- `BEGIN EXCLUSIVE` en WAL équivaut à `IMMEDIATE` : il ne bloque plus les lecteurs. Pour forcer un
  `SQLITE_BUSY` côté lecteur en WAL : connexion en `PRAGMA locking_mode=EXCLUSIVE` ayant écrit.
- Le mode est persistant dans le fichier ; la bascule exige qu'aucune autre connexion n'ait de
  transaction ouverte, sinon la commande rend l'ancien mode : d'où la vérification de la valeur rendue.
- Fichiers `-wal` et `-shm` créés avec les droits du fichier principal ; une connexion en lecture seule
  fonctionne si le répertoire permet la création de `-shm` (même utilisateur : oui).
- Point de contrôle automatique tous les 1000 pages ; un lecteur long retarde le point de contrôle mais
  ne bloque rien (nos lecteurs 104 ferment leur transaction avant tout travail CPU).
- Une base `:memory:` rend `memory` et ignore WAL.

## Découvertes d'implémentation (2026-09-18)

- La bascule `delete → wal` exige un verrou exclusif bref ; sur cette transition SQLite **n'appelle pas** le
  gestionnaire d'attente (`busy_timeout`) pour éviter un interblocage. Huit `Store::open` simultanés sur une
  base neuve (test `core_089_concurrency_test`) échouaient donc en `DatabaseBusy`. Réponse : relire le mode,
  puis réessayer la bascule (10 ms, borne 2 s) ; une base déjà WAL est confirmée sans verrou.
- Le pragma doit venir **après** le contrôle de schéma (`store_schema::validate`) : sinon l'en-tête d'une base
  d'une version future est modifié avant son refus (test `core_089_migration_test`, comparaison octet à octet).
- Une connexion WAL ouverte, même inactive, garde un marqueur partagé sur `-shm` : `locking_mode=EXCLUSIVE`
  n'est obtenable qu'en l'absence de toute autre connexion (impact : test 104 S14).

## Risques

- Copie à chaud de `bridget.db` seul (sans `-wal`) : données récentes absentes → documenter (US3).
- Outils externes en `-readonly` : fonctionnent sur WAL si `-shm` existe ou est créable.
