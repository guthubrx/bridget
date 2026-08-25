# Preuves d'implémentation — Session 023

## Références

- Base : `b6eea777facf929d99a9c4f9ae75fb50e06dc2fd`.
- Projection P1 : `f8ca6ae`.
- Diagnostic P2 : `5ba014d`.
- Première tête refusée par le jury : `79bd1c5d31ac0968844e2bbed8475cf1e6dbfe63`.
- Correction de code après STOP : `888d4aa5c15a277c215c35a742126fd691ef152e`.
- Tête documentaire finale : fournie dans le rapport de livraison.
- Migration : aucune ; v19 reste inutilisée.

## Témoins avant correction et mutations

- Le premier essai P1, arrêté avant la création des fixtures, est écarté : il
  ne mesurait pas la propriété.
- Avec un lecteur permissif temporaire, le harnais P1 atteint son assertion
  métier puis échoue : 0 harnais passé, 1 rouge.
- Avant P2, les trois tests nommés atteignent leurs assertions : 0 passé,
  3 rouges, 0 ignoré.
- Mutation P1 : remplacer les terminaux reconnus par deux noms impossibles fait
  échouer le harnais sur `fin sans reprise non détectée` : 0/1.
- Mutation P2 : rendre seulement le message `busy` générique donne 2 passés,
  1 rouge, 0 ignoré ; le rouge est le témoin JSON `busy`.
- Un faux vert du témoin de corruption a été écarté : la recherche `json`
  matchait l'extension `.jsonl`. Le témoin final produit une ligne réellement
  invalide et exige le préfixe `journal-json-invalide:`.

## STOP rc3 et reprise

Deux témoins rouges sans mutation ont reproduit les charges sur `79bd1c5d` :

1. `turn_start`, puis une `error` ACP non terminale, puis une `update`
   corrélée : le lecteur concluait `ended` au seq de l'erreur et retirait le
   tour des `OCCUPES` ;
2. un `stop_reason` contenant ESC, retour chariot et U+202E : ESC et U+202E
   atteignaient réellement stdout.

Le faux ACP de production a également maintenu le prompt ouvert après
l'anomalie et l'`update`, puis produit un rejet terminal. Avant correction,
son test atteint l'assertion finale et donne 0 passé, 1 rouge : le dernier
`error` ne portait aucun `terminal_kind`.

Deux exécutions ne sont pas comptées : la première ne compilait pas car le
témoin ne qualifiait pas `UNIX_EPOCH` ; la seconde employait un nom court avec
`--exact` et filtrait les 110 tests. Après correction du montage, l'univers
mesuré contenait exactement un test et le rouge portait sur le marqueur absent.

La correction sépare désormais :

- le code fermé `turn_completed` dérivé de `turn_end` ;
- le code fermé `turn_failed`, écrit par les workers ACP et Codex ;
- `reason` et `stop_reason`, détails libres sans influence sur la décision.

Une ancienne `error` sans marqueur devient `INDETERMINE`. Une `update`
corrélée ultérieure prouve la continuation. Le texte échappe toutes les
catégories Unicode de contrôle ; le JSON ASCII échappé conserve la valeur
diagnostique après parsing.

## Mesures de la correction

Plateforme : Linux `6.8.0-94-generic` x86_64 GNU/Linux ; Rust
`1.92.0` ; Cargo `1.92.0` ; Python `3.12.3`.

- `cargo test --workspace --no-run` : vert avant le comptage complet.
- Harnais de ronde : retour 0, partition de 19 agents, anomalie ACP suivie
  d'une `update` conservée ouverte, ancienne `error` seule indéterminée, deux
  formes terminales positivement attestées.
- Sorties hostiles : ESC, retour chariot et U+202E attestés dans les fixtures
  `stop_reason` et `error.reason` ; aucun contrôle brut dans le texte ni dans
  le JSON sérialisé ; le JSON parsé restitue le même détail libre.
- Faux ACP réel : prompt encore `InProgress` après anomalie et continuation,
  puis fin uniquement après le rejet terminal : 1 passé, 0 rouge, 0 ignoré.
- Passerelle Codex : 1 passé, 0 rouge, 0 ignoré ; le code terminal est ajouté
  après les enrichissements du payload et absent des événements non terminaux.
- Transport complet : 110 passés, 0 rouge, 1 ignoré. Les deux résumés `1/0/0`
  des sous-processus du banc de noms temporaires ne sont pas additionnés.
- Témoins P2 : 3 passés, 0 rouge, 0 ignoré.
- Workspace, 58 cibles Cargo de premier niveau : 935 passés, 3 rouges,
  16 ignorés. L'ajout des deux tests Rust explique 933 → 935.

Les trois rouges sont identiques à la base, dans des fichiers non modifiés par
la reprise :

1. `attach::tests::raw_mode_restaure_le_terminal_apres_eof_du_pseudo_tty` ;
2. `attach::tests::reconnexion_socket_reprend_exactement_a_last_seq_plus_un` ;
3. `lifecycle::tests::matrice_sc003_couvre_les_onze_familles_sans_residu_operationnel`.

Ils sont imputés à la dette Linux préexistante de `bridget-daemon` : rc3 les a
reproduits isolément sur la base `b6eea777` avec les mêmes assertions, et la
reprise 023 ne modifie aucun fichier de ce paquet. Les quatre bancs instables
annoncés (`matrice_fr008`, `eof_pendant_un_tour`, redelivery après reconnexion,
`ignored_cancel_kills_transport`) sont verts dans cette exécution.

Trois mutations indépendantes sont mortes après atteinte de leur univers :

1. code terminal lu altéré : harnais 0/1 sur la fin non détectée ;
2. code terminal produit altéré : compilation verte, faux ACP 0/1 sur
   `terminal_kind` exact ;
3. filtre réduit de toutes les catégories `C*` à `Cc` : harnais 0/1 sur le
   contrôle bidirectionnel U+202E brut.

Gates verts : `cargo fmt -p bridget-transport -- --check`, `cargo clippy
-p bridget-transport --all-targets -- -D warnings`, `cargo clippy -p maicie
--bin maicie -- -D warnings`, `rustfmt --edition 2024 --check` sur le test
Maicie corrigé, `ruff`, syntaxes Python/Bash et `git diff --check`.

## Mesures de la première livraison

- `cargo test --workspace --no-run` : vert avant tout comptage Rust.
- Témoins P2 `test_023_` : 3 passés, 0 rouge, 0 ignoré.
- Suite Maicie : 303 passés, 0 rouge, 5 ignorés.
- Workspace : 933 passés, 3 rouges, 16 ignorés.
- Harnais P1 : retour 0 ; partition de 15 fixtures, contrôles de terminal,
  reprise, tour long, source incomplète, contradiction et incertitudes verts.
- `cargo clippy -p maicie --bin maicie -- -D warnings` : vert.
- `ruff check scripts/bridget-idle.py`, syntaxe Python, syntaxe Bash et
  `git diff --check` : verts.

Les trois rouges workspace sont préexistants et déterministes :

1. `attach::tests::raw_mode_restaure_le_terminal_apres_eof_du_pseudo_tty` ;
2. `attach::tests::reconnexion_socket_reprend_exactement_a_last_seq_plus_un` ;
3. `lifecycle::tests::matrice_sc003_couvre_les_onze_familles_sans_residu_operationnel`.

La session 023 ne modifie aucun fichier de `bridget-daemon`. La session 022,
sur la même base, documente ces trois rouges à 20/20 avant correction puis
20/20 verts après correction. Aucun des quatre bancs instables annoncés n'est
tombé pendant la mesure 023.

## Gates non verts hors périmètre

- `cargo clippy -p maicie --all-targets -- -D warnings` s'arrête sur deux
  `doc_lazy_continuation` préexistants dans
  `plugins/maicie/tests/contract/routines.rs`.
- Le contrôle de format global Maicie reste rouge sur une dette préexistante.
  Contrairement au rapport initial, un hunk 023 de
  `plugins/maicie/tests/integration/cli_delegate.rs` était aussi rouge : rc3
  l'a isolé sur la chaîne du profil absent. Cette affirmation initiale était
  fausse ; la reprise corrige ce hunk et le vérifie séparément.
- Le formatteur Python propose uniquement les longues lignes déjà présentes
  sur la base ; le lint Python est vert.

## Composition

| Lot | Tête mesurée | Arbre fusionné | Résultat de la reprise |
| --- | --- | --- | --- |
| 018 activation | `2e23ee17e2370beb5ff497b81903b76a4a85f1f9` | `0f1fc61af389df1edad9db8f47ad6233522c5ff2` | fusion Git propre ; activation de 023 à faire par cette règle après admission sur `origin/main` |
| 019 interactions | `ca81bf49e00489aa97f46c180ceb8d0ebd1574b7` | `9cc6f7f033bfca5aa9e2bb01c412ae76e9df62aa` | arbre matérialisé ; workspace compilé ; témoins 019 à 9/0/0, passerelle à 1/0/0 et harnais 023 vert |
| 021 verdict | `10cc5ed97e95b2ca8945f3a9c8668dcb7a626cfa` | `70ffae0971b3dc6fb5965c6dfba1565f6ffa76` | arbre matérialisé ; workspace compilé ; témoins P2 à 3/0/0 et harnais 023 vert |
| 022 bancs permanents | `3a09f51aaa4e3df48b2d313f6a29005c6a4e2739` | `f5faffbc31848ff645070b182aa75d817661311f` | fusion Git propre ; fichiers de production disjoints |

Les compositions 019 et 021 ont été matérialisées sans commit dans des
worktrees temporaires puis supprimées. Les compositions 018 et 022 n'ont pas
été compilées pendant la reprise ; seule leur fusion arborescente a été
remesurée.

## Limites déclarées

- Aucun outil actif n'est remplacé depuis cette branche.
- La politique de refus d'une cible `busy` n'est pas modifiée.
- Aucun appel Maicie réel n'est lancé sans chemin de configuration explicite ;
  les tests emploient des fixtures et des copies de bases.
- Les wrappers interactifs `unix` et `ssh-unix` ne produisent pas encore de
  borne terminale. Hors `state=busy`, leur activité reste donc indéterminée.
- La reprise n'a pas été exécutée sur macOS, contre un annuaire/journal réel,
  pendant une course lecture/append, ni sur une rotation couvrant plusieurs
  jours. Les témoins emploient des journaux v1 et un faux ACP sur Linux.
