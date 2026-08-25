# Session 022 — Tests workspace permanents

## Contexte

Deux exécutions indépendantes de la suite workspace ont signalé les trois mêmes
échecs déterministes. Leur présence permanente masque les nouvelles régressions
et oblige chaque livraison à refaire leur imputation.

## Propriétés

### P1 — Sortie logique d'un terminal raw

Toute sortie logique d'un terminal raw encore vivant, notamment Ctrl-D, restaure
les bits `ICANON`, `ECHO`, `ISIG`, `VMIN` et `VTIME` modifiés par Bridget. Un
hangup qui détruit le terminal n'est pas un oracle de restauration portable.

### P2 — EOF non-TTY sans perte d'observation

L'EOF d'une entrée non-TTY retire seulement la capacité de saisie. La vue attach
continue d'observer la socket, se reconnecte après une coupure et reprend à
`last_seq + 1`. Une entrée épuisée ne doit pas provoquer de boucle active.

### P3 — Catalogue relayé sans copie littérale

Un refus `UnknownType` relaie l'instantané ordonné du registre effectivement
chargé. La matrice lifecycle compare ce relais à sa fixture de registre ; elle
ne recopie pas la liste des agents natifs, détenue par `registry.rs`.

## Critères d'acceptation

- Les trois noms de tests sont confirmés par `cargo test -- --list`.
- `cargo test --no-run` réussit avant toute campagne de fréquence.
- Chaque témoin est mesuré sur le même hôte et sous charge déclarée, avec le
  même nombre de passages avant et après correction.
- P1 corrige uniquement le banc ; P2 possède un commit de production distinct ;
  P3 corrige uniquement le banc et tue un mutant qui supprime le relais.
- La suite workspace complète ne contient plus ces trois rouges permanents ;
  tout autre rouge est nommé et imputé.

## Hors périmètre

- Les quatre bancs instables attribués au lot jc2.
- Le gate Linux `--features test-support`, connu non compilable à cause de
  `kqueue`/`kevent` sans garde de plateforme.
