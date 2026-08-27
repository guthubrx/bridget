# Vérification de la session 041

## Réalisation

- Un parseur partagé exige un entier strictement positif et conserve le type de
  destination (`u64` pour le délai, `i32` pour les sauts).
- `send` et `reply` emploient cette frontière avant toute connexion au daemon.
- Pour `reply`, l'analyse des options précède la lecture du dernier expéditeur ;
  l'ordre historique des autres validations reste inchangé.
- La divergence du délai sans demande de réponse demeure hors périmètre.

## Oracles

Le harnais d'intégration lance le vrai binaire avec un `HOME` jetable et un faux
daemon sur une socket Unix courte. Il capture le JSON effectivement envoyé :

- une valeur invalide, nulle, négative, absente ou hors type doit produire le
  code 2 sans connexion ;
- les diagnostics nomment l'option et, lorsqu'elle existe, la valeur brute ;
- les valeurs de contrôle `9` et `2` sont sérialisées exactement par `send` et
  `reply`.

Univers listés avant exécution :

- base : 35 tests unitaires CLI + 3 tests d'intégration CLI = 38 ;
- tête : 35 tests unitaires CLI + 8 tests d'intégration CLI = 43.

Résultats : base `38 passed / 0 failed / 0 ignored`, tête
`43 passed / 0 failed / 0 ignored`.

## Mutants

- Restaurer l'ancien repli du délai dans `send` et `reply` donne
  `0 passed / 2 failed` ; les deux sorties montrent le JSON parti malgré
  `--timeout abc`.
- Restaurer l'ancien repli des sauts donne `0 passed / 2 failed` avec la même
  preuve pour `--hops abc`.
- Après restauration, les condensats SHA-256 des deux fichiers reviennent aux
  valeurs nominales et le harnais repasse intégralement.

## Portes et limites

- `cargo check --workspace --all-targets` termine des deux côtés avec 16 lignes
  d'avertissement chacune.
- Clippy strict échoue des deux côtés sur les mêmes 13 diagnostics préexistants,
  après normalisation des chemins et numéros de ligne ; aucun diagnostic n'est
  ajouté par ce lot.
- Les deux fichiers Rust modifiés passent `rustfmt --check` et le delta passe
  `git diff --check`.
- L'univers daemon complet compte 642 tests sur la base et 647 sur la tête,
  mais aucune campagne ne fournit de ligne finale en 60 secondes. Les deux
  montrent avant blocage les mêmes échecs
  `enregistrement_auxiliaire_mcp_ne_revendique_pas_la_presence_du_wrapper_vivant`
  et `sigkill_daemon_reconcilie_l_ancien_groupe_avant_une_reprise_unique`.
  Aucun compte global n'est revendiqué.
