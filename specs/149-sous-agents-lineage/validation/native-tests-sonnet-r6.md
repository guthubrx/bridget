# Tests natifs 149 - ronde Sonnet r6 (sha2 `asm`, latence du hachage)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 high (claudeAgent). Aucun modèle réel, aucun Git, aucune installation, aucun redémarrage.
Racine : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage`. Dossier de preuves : `…/specs/149-sous-agents-lineage/validation/`.

## Verdict

- **Binaires privés disponibles** : debug r6 et release r6 (candidat, non installé). Voir plus bas.
- **Benchmark avant/après fait** : `hash-latency149.md`. Release : 118 ms par hachage (cible < 1 s tenue). Debug : 2,1 s (3,5 fois mieux, cible non tenue).
- **Tests : 1805 PASS, 1 FAIL, 63 ignorés.** Le FAIL est un flocon de fixture sans lien avec SHA (voir plus bas). Il n'est pas déclaré vert.
- Aucun défaut de production trouvé. Deux constats d'optimisation pour Sol, aucun correctif appliqué.

## Cargo.lock : delta exact

Le manifeste (`Cargo.toml`, ligne 22, édité par Sol) demande `sha2 = { version = "0.10", features = ["asm"] }`. J'ai généré le lock avec `cargo fetch` (réseau, sans `cargo update`) à partir du lock existant. Sauvegarde de l'ancien : `Cargo.lock.r5-avant-sha2-asm` (sha `8dfbec1d…8737`).

`diff` complet : +10 lignes, 0 suppression.
- **Ajout** : `sha2-asm 0.6.4`, source `registry+https://github.com/rust-lang/crates.io-index`, checksum `b845214d6175804686b2bd482bcffe96651bb2d1200742b712003504a2dac1ab`, dépendance `cc`. Le fichier `.crate` téléchargé a le même SHA-256 (vérifié).
- **Modification** : le paquet `sha2 0.10.9` gagne la dépendance `sha2-asm`.
- **Inchangés** : `sha2 0.10.9` (a7507d81…), `cpufeatures 0.2.17`, `cc 1.4.2`, et tous les autres paquets. Un seul crate téléchargé (`sha2-asm-0.6.4`). Nouveau sha du lock : `df6a077c…02c7b3`.

## Binaires (copies stables, rien d'installé, rien d'écrasé)

| Build | Chemin | SHA-256 | Taille |
|---|---|---|---|
| debug r6 | `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-debug/bridget-620e729fca53` | `620e729fca53c621a4a7be784fa948ba813768764e3cf518e030ef60c0cf4fd2` | 54 046 648 |
| release r6 (candidat privé) | `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-abfb346e23cc` | `abfb346e23ccf51dad41b90658d475e5cbab321c9865dfd9e778a24d37c8c138` | 17 457 520 |

- Les deux : `bridget 0.1.3`, Mach-O arm64, signature ad hoc (`codesign -v` OK), mode 0700, **non installés**. Aucun `target/release` à la racine du dépôt. Le release vient du dossier cible externe.
- Commandes : `cargo build -p bridget-daemon --bins --offline` (31 s) et `cargo build --release -p bridget-daemon --bins --offline` (138 s). Environnement : `CARGO_BUILD_JOBS=2`, `CARGO_INCREMENTAL=0`, `TMPDIR=/tmp/b149t-r6` (0700), `umask 077`.
- Aucun `.rs` de production plus récent que les binaires. Le backend matériel est prouvé (symboles et instructions), voir `hash-latency149.md`.
- **Binaires immuables vérifiés après coup** : `bridget-ec6b18b5d468` (r5, SHA `ec6b18b5…8b0f`) et `bridget-823e8a5fab8a` (r4) ont le même SHA qu'avant.
- `target/debug/bridget` a été reconstruit par `cargo test` à 16:31 (hash différent). Seule la copie stable fait foi.
- Reçus : `native149-debug-receipt-r5.json` (archive identique à l'ancien reçu, sha `4890d8f7…932fe`), `native149-debug-receipt.json` (r6, actuel), `native149-release-receipt-r6.json` (nouveau).

## Empreintes de source

Algorithme r5 conservé (script `hash-latency149-tool-fingerprint.py`).
- Tout : 197 fichiers au build = `d74042b2…725f` ; 198 fichiers avec le test neuf = `ebed6dc3…edcf`. Avant (r5) : `e9b0eb9f…0d22`.
- Production : 111 fichiers, `6cc9a2be…367f` (r5 : `65403980…b049`). La différence vient uniquement de `Cargo.toml` et `Cargo.lock`.
- Preuve que rien d'autre n'a changé depuis r5 : avec le `Cargo.toml` de HEAD et le lock sauvegardé, l'arbre actuel redonne exactement `e9b0eb9f…`.
- `git diff HEAD -- crates` : sha `57268185…8f5`, identique à r5. `git diff HEAD -- Cargo.toml Cargo.lock` : sha `dde4ff76…c332`.

## Tests

Compteurs séparés. Journaux : `native-r6-*.log`.

| Suite | PASS | FAIL | Ignorés |
|---|---|---|---|
| `bridget-transport` complet (11 exécutables) | **328** | 0 | 2 |
| `bridget-daemon` complet (79 exécutables) | **1477** | **1** | 61 |
| Total | **1805** | **1** | **63** |
| Observer 149 en série (`--test-threads=1`) | 10 | 0 | 0 |
| Natifs 149 (lib, filtre `native149`) | 45 | 0 | 0 |
| Démarrage R9 (lib `native_delegation_permissions149`) | 13 | 0 | 0 |
| Mort du fournisseur 149 (vrai binaire, vrai daemon) | 2 | 0 | 0 |

Écart avec r5 (1803 PASS / 0 FAIL / 63 ignorés) : +3 tests neufs (vecteurs SHA-256) et 1 flocon.

Les 7 tests R9 (5 de démarrage/reconnexion + 2 de mort du fournisseur) passent, chaque témoin ordinaire inclus. Les tests des réglages (contenu modifié, source supprimée, source ajoutée, lien symbolique, PATH) de r4/r5 passent tels quels : aucun test affaibli, aucun test modifié.

### Le FAIL

- `modele_non_declare_est_refuse_avant_processus_et_ordre_durable` (`crates/bridget-daemon/tests/capabilities_integration_test.rs:151`) : `SqliteFailure DatabaseBusy (database is locked)`.
- Cause observée : le préalable de la fixture appelle `referent_control::set` sur une connexion rusqlite brute juste après `Store::open`. `crates/bridget-daemon/src/referent_control.rs:139` documente déjà ce `SQLITE_BUSY` malgré le `busy_timeout` quand deux ouvertures se croisent. La charge machine était d'environ 13 pendant le run.
- L'échec arrive avant tout hachage. Fichier et module non modifiés dans l'arbre (`git diff` vide). Rejeu isolé : **5 sur 5 PASS** (`native-r6-capabilities-reruns.log`). Passé en r5.
- Classement : flocon de fixture, non reproduit. Je ne le compte pas comme vert dans la suite complète. Je n'ai pas relancé la suite complète une seconde fois.

### Tests neufs (test seulement)

Fichier : `crates/bridget-transport/tests/permission_source_revision_vectors149_test.rs` (3 tests, formaté avec `rustfmt` sur ce seul fichier, 0 avertissement).
1. Vecteurs FIPS 180-2 : vide, `abc`, message de 448 bits, un million de `a`.
2. 16 longueurs (de 1 à 1 000 003 octets) autour des bords de bloc (64), de remplissage (55/56) et de lecture (65 536). Les références viennent de `hashlib` et de `shasum -a 256`, qui donnent le même résultat. Rien ne recalcule le hachage avec le même algorithme.
3. Un contenu de même taille avec date de modification restaurée est refusé par le recheck. Cela interdit un futur cache fondé sur taille et date.

Sensibilité : dans une copie jetable, j'ai tronqué le lecteur à 65 535 octets. Les tests 1 et 2 échouent (`native-r6-mutation-vectors.txt`). Le test 3 n'est pas concerné par cette mutation.
Ces tests passent aussi avec l'ancien backend logiciel : ils prouvent que le backend matériel donne les bons résultats, pas qu'il est actif. L'activité du backend est prouvée par les symboles, les instructions et le débit.

## Limites

- Les fixtures `claude` des tests sont des copies de `/bin/echo` ou de petits scripts : aucune preuve de modèle. **Aucune validation T036 à T039 n'est déduite.**
- Le banc de latence simule le fournisseur (ACK immédiat). Le client du hook est le vrai binaire.
- Je n'ai pas lancé `clippy`. Le seul fichier de test neuf est formaté. Aucun `cargo fmt` global.
- L'état Git n'est pas propre (arbre non committé) : aucun reçu ne prétend le contraire.

## Constats pour Sol et le principal

1. Debug lent : un hachage prend encore 2,1 s en debug r6. `[profile.dev.package.sha2] opt-level = 3` dans le manifeste donne 115 ms (essai privé, lock identique). Édition hors de mon périmètre.
2. Double hachage en CLI direct (`native_permissions.rs:79` et `:159`) : réutiliser `launcher_revision` quand `cli == launcher` diviserait le coût par deux. Édition de production hors de mon périmètre.
3. Flocon `capabilities_integration_test` : à surveiller, pas de lien avec ce lot.

## Pour les prochaines recettes r6

- Recettes avec un **parent GLM via `gclaude`** : le debug r6 passe (hook à 2,1 s pour un délai client de 3 s, marge de 0,9 s, donc fragile sous charge). Le release r6 passe avec 118 ms.
- Recettes avec un **parent Claude lancé en direct** : le debug r6 échoue encore (4,2 s). Utiliser le **release r6** (236 ms), ou attendre le constat 1.
- Choix recommandé pour T037/T038 : `bridget-abfb346e23cc` (release, candidat privé, non installé). Le code de production est identique à celui du debug r6.
- Ne pas utiliser `bridget-ec6b18b5d468` (r5) pour ces recettes : le hook y dépasse 3 s dans les deux parcours.

## Nettoyage

Mes copies de travail `/tmp/b149t-r6/ws-*` et leurs dossiers cible ont été supprimés. Aucun processus à arrêter : aucun daemon ni service lancé. Les outils de mesure sont conservés dans `hash-latency149-tool-*`.
