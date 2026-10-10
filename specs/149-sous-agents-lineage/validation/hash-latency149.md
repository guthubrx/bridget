# Latence du hachage du CLI Claude - avant/après sha2 `asm` (session 149, r6)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 high (claudeAgent). Aucun modèle réel.
Données brutes : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/hash-latency149-data.json`.

## Résultat en une ligne

Le backend matériel ARM64 de sha2 est actif et donne les mêmes SHA-256. En **release**, un contrôle complet coûte 118 ms (lanceur `gclaude`) ou 236 ms (CLI direct) : la cible de moins d'1 s est tenue. En **debug**, le gain est de 3,5 fois, mais un hachage coûte encore 2,1 s : la cible n'est pas tenue.

## Ce qui a été mesuré

- CLI réel : `/Users/moi/.local/share/claude/versions/2.1.296` (lien `~/.local/bin/claude`), 240 664 432 octets.
- SHA-256 attendu `c9b5341637becbd423ddffc5b254afb645682a3868cb708bbc6cc0e7bb419937`. Les 5 lectures de chaque exécution (9 exécutions, 45 lectures) ont rendu exactement ce SHA : le test échoue sinon.
- Source de la mesure : le vrai `bridget_transport::protocol::permission_source_revision(cli, 512 Mio)`, puis le vrai `capture_context_with_env` / `recheck_context_with_env` du daemon, puis le vrai Observer avec le vrai client du hook (`bridget __native-permission-observer`, délai de lecture 3 s non modifié).
- Banc : test `#[ignore]` ajouté **seulement dans des copies jetables** de l'arbre (`/tmp/b149t-r6/ws-r5`, `ws-r6`), source conservée dans `hash-latency149-bench.rs.txt`. Aucune ligne ajoutée à l'arbre de travail.
  - Copie r5 : `Cargo.toml` de HEAD (`sha2 = "0.10"`) + `Cargo.lock` d'avant (sha 8dfbec1d…). Le reste est identique à l'arbre actuel. Preuve : l'empreinte de source recalculée avec ces deux fichiers rend `e9b0eb9f…`, égale à l'empreinte r5 du reçu.
  - Copie r6 : manifeste et lock actuels. `cargo test --locked --offline` dans les deux copies.
- Même CLI, même machine (Apple M3 Max, 26.2), même environnement. Cache disque jamais vidé. Deux passes dans des ordres inverses (A : r5, r6, release r6, release r5 ; B : ordre inverse). Dispersion de moins de 3 %.
- La machine est partagée : la charge moyenne était voisine de 13 pendant la session. Les écarts min-max restent faibles.
- **Mode simulé** : le fournisseur n'existe pas. L'ACK du daemon part dès que le fait est publié (délai 0). Les chiffres du hook ne contiennent donc que le calcul du recheck et le transport local. Le délai client (3 s) et le délai d'ACK (2 s) sont inchangés.

## Un hachage complet du CLI (5 lectures, médiane de 10 lectures)

| Build | Médiane | Min - max | Débit |
|---|---|---|---|
| debug r5 (sha2 logiciel) | 7 339 ms | 7 301 - 7 403 | 33 Mo/s |
| debug r6 (sha2 `asm`) | 2 103 ms | 2 086 - 2 186 | 114 Mo/s |
| release r6 (sha2 `asm`) | **118 ms** | 117 - 118 | 2,0 Go/s |
| release r5 de référence (logiciel) | 539 ms | 538 - 558 | 447 Mo/s |
| debug r6 + `sha2` en opt-level 3 (variante privée) | **115 ms** | 114 - 119 | 2,1 Go/s |

Gains : debug 3,5 fois ; release 4,6 fois ; release r6 contre debug r5 : 62 fois. Référence externe du r4 : `shasum -a 256` natif, 1,2 s.

## Contrôle complet (`recheck_context_with_env`, médiane de 6 mesures)

Équivalent en hachages complets, lu dans le code : `capture_context_with_env` hache `launcher` (ligne 79) puis `cli` (ligne 159) de `crates/bridget-daemon/src/native_permissions.rs`.
- CLI direct (`command = claude`) : lanceur = CLI, donc **2 hachages** du même fichier de 240 Mo.
- Lanceur `gclaude` (script de 1 385 octets, SHA contrat `dd8dee56…`) : **1 hachage** du CLI.
- `recheck` = `capture` + révisions de petits fichiers. Les mesures confirment le rapport 2,00.

| Build | Direct (2 hachages) | `gclaude` (1 hachage) |
|---|---|---|
| debug r5 | 14 709 ms | 7 345 ms |
| debug r6 | 4 199 ms | 2 127 ms |
| release r6 | **236 ms** | **118 ms** |
| release r5 de référence | 1 090 ms | 540 ms |
| debug r6 + sha2 opt-level 3 (privé) | 226 ms | 115 ms |

## Hook complet (fait publié, client réel, délai client 3 s)

| Build | Fait publié (direct / `gclaude`) | Décision du client réel |
|---|---|---|
| debug r5 | 14 663 / 7 330 ms | **deny 6/6 et 6/6** (dépassement de 3 s) |
| debug r6 | 4 213 / 2 100 ms | direct **deny 6/6** ; `gclaude` **allow 6/6** (marge 0,9 s) |
| release r6 | 238 / 120 ms | allow 6/6 et 6/6 |
| release r5 de référence | 1 083 / 543 ms | allow 6/6 et 6/6 |
| debug r6 + sha2 opt-level 3 (privé) | 228 / 118 ms | allow 3/3 et 3/3 |

Lecture :
1. Le défaut r4/r5 est reproduit : en debug r5, le hook dépasse 3 s dans les deux cas.
2. En debug r6, le parcours `gclaude` (profil GLM) passe, avec 0,9 s de marge. Le parcours CLI direct (parent Claude lancé sans `gclaude`) échoue encore (4,2 s).
3. En release r6, les deux parcours passent largement. Un hachage de 118 ms tient dans la cible de 1 s.

## Preuve que le backend matériel est actif

- Source locale `sha2-0.10.9` : `Cargo.toml:49` (`asm = ["sha2-asm"]`) ; `src/sha256.rs:24` choisit `aarch64` si `asm` et `target_arch = "aarch64"` ; `src/sha256/aarch64.rs:13` détecte `sha2` à l'exécution avec `cpufeatures` et retombe sur `soft` sinon. Machine : `hw.optional.arm.FEAT_SHA256 = 1`.
- Symboles du binaire debug r6 : `sha2::sha256::aarch64::{sha2_hwcap::get, sha256_compress, compress}`. Le binaire r5 n'a que `sha2::sha256::soft::compress`.
- Instructions (`llvm-objdump --mattr=+sha2,+crypto`) : r5 = 0 `sha256h2`/`sha256su0`/`sha256su1` ; debug r6 = 8/4/4 ; release r6 = 16/12/12.
- `sha2-asm 0.6.4` est compilé mais **non utilisé sur aarch64** : le backend matériel est de l'assembleur inline dans `sha2` lui-même. Le symbole C `sha256_compress` n'est pas lié.

## Constats pour Sol et le principal (rien n'est modifié par le testeur)

1. **Debug** : ajouter `[profile.dev.package.sha2] opt-level = 3` au `Cargo.toml` ramène un hachage de 2 103 ms à 115 ms en debug (mesuré dans une copie privée, `Cargo.lock` identique). Cela rend les recettes avec un binaire debug viables pour le parcours direct. Décision et édition : Sol / principal.
2. **Double hachage** : en CLI direct, le même fichier est haché deux fois par contrôle (`native_permissions.rs:79` et `:159`). Réutiliser `launcher_revision` quand `cli == launcher` diviserait le coût par deux sans changer le résultat. Édition de production : Sol. Aucun garde, cache, délai ou refus n'a été touché.
3. Pas de cache fondé sur la taille ou la date : le test `native149_recheck_refuse_un_contenu_de_meme_taille_a_mtime_restaure` verrouille ce point.

## Limites

- Trois mesures par scénario de hook, 5 lectures par exécution ; pas d'étude de charge. Pas de mesure avec un vrai fournisseur ni un vrai modèle.
- Le client du hook est le vrai binaire `bridget` : `bridget-ec6b18b5d468` pour debug r5, `bridget-620e729fca53` pour debug r6, `bridget-abfb346e23cc` pour les trois mesures release (dont la référence release r5). Le serveur (Observer) est celui du binaire de test de chaque copie, compilé avec le même profil. Le temps du client lui-même est négligeable (moins de 15 ms).
- La variante `sha2 opt-level 3` est un essai privé jamais livré.

## Fichiers

- Banc : `hash-latency149-bench.rs.txt`, `hash-latency149-tool-*` (scripts de matrice, de build et d'empreinte).
- Journaux de build des copies : `hash-latency149-build-ws-*.log`.
- Données brutes : `hash-latency149-data.json`.
