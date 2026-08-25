# Preuves d'implémentation — Session 023

## Références

- Base : `b6eea777facf929d99a9c4f9ae75fb50e06dc2fd`.
- Projection P1 : `f8ca6ae`.
- Diagnostic P2 : `5ba014d`.
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

## Mesures restaurées

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
- Le contrôle de format global Maicie reste rouge sur la dette préexistante de
  `routines.rs`, `store.rs`, `main.rs` et du contrat `routines.rs`. Les ajouts
  de 023 ne figurent pas dans les hunks proposés par `rustfmt`.
- Le formatteur Python propose uniquement les longues lignes déjà présentes
  sur la base ; le lint Python est vert.

## Composition

| Lot | Tête mesurée | Résultat |
| --- | --- | --- |
| 018 activation | `2e23ee17e2370beb5ff497b81903b76a4a85f1f9` | fusion Git propre ; activation de 023 à faire par cette règle après admission sur `origin/main` |
| 019 interactions | `07ea1e8ba68299cf58e0ad211d7cf3fa793483ad` | fusion Git propre ; fichiers de production disjoints |
| 021 verdict | `dedb5e39d1ad700d3dbc63ca443784e313e0036f` | fusion Git propre ; workspace compilé ; témoins 023 à 3/0/0 |
| 022 bancs permanents | `3a09f51aaa4e3df48b2d313f6a29005c6a4e2739` | fusion Git propre ; fichiers de production disjoints |

La composition 021 a été matérialisée dans un worktree temporaire puis
supprimée. Les compositions 018, 019 et 022 n'ont pas été compilées ; seule
leur fusion arborescente a été mesurée.

## Limites déclarées

- Aucun outil actif n'est remplacé depuis cette branche.
- La politique de refus d'une cible `busy` n'est pas modifiée.
- Aucun appel Maicie réel n'est lancé sans chemin de configuration explicite ;
  les tests emploient des fixtures et des copies de bases.
- Les wrappers interactifs `unix` et `ssh-unix` ne produisent pas encore de
  borne terminale. Hors `state=busy`, leur activité reste donc indéterminée.
