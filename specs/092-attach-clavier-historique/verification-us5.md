# Vérification US5 — raccourcis classiques et LF iTerm

État : validé et installé le 06/09/2026 ; les preuves ci-dessous sont propres à US5.

## Observation initiale

L'humain constate que Shift+Entrée envoie. La préférence iTerm relevée associe
Shift+Return à SendText LF. Le chemin précédent convertissait CR et LF en Send ;
la garde conservait également les transformations termios d'entrée. Les recettes
US4 injectaient CSI-u, pas cette configuration LF. Aucun réglage iTerm n'est modifié.

## Contrat à prouver

- Double TTY : CR envoie, LF insère ; CR/LF restent distincts dans le PTY réel.
- Entrée CSI-u, Shift+Entrée enrichie, Ctrl-J et Option+Entrée suivent le contrat.
- Ctrl-A/E/U/K/W/Y et Option-Backspace/D agissent au curseur UTF-8 ; limites LF
  logiques, registre de suppression unique, no-op et backspace ordinaire préservés.
- Hors double TTY : CR/LF envoient comme avant, commandes d'édition inertes.
- Termios restauré exactement ; aucune configuration, permission ou identité modifiée.

## Ordre de validation

1. Exécuter les oracles `spec092_us5_` rouges livrés par l'auteur avant son patch.
2. Exécuter ces mêmes oracles sur le candidat, puis tous les tests attach.
3. Contre-revue indépendante et grille terminal réelle (LF brut, édition classique,
   accents/emoji, saisie multiligne) sans envoyer une mission accidentelle.
4. Une consolidation workspace, fmt et clippy ; compilation release distincte du
   binaire actif ; remplacement atomique vérifié. Aucun redémarrage d'agent/daemon.

## Rouge initial observé

Commande `cargo test -p bridget-daemon --lib attach::tests::spec092_us5_ -- --nocapture`
depuis le worktree, PATH Cargo explicite et TMPDIR privé du pilote.
Compilation 4,16 s ; 6 tests exécutés, 6 échecs attendus : conversion termios,
LF ayant envoyé, absence des raccourcis de ligne/mot/registre, et repli Option-BS
qui effaçait encore un caractère hors double TTY. Code retour 101.
Log : `/tmp/b92classic.YpV88G/red.log`.

Une erreur d'attendu dans l'oracle registre a été signalée avant implémentation :
`alpha ` puis backspace puis deux réinsertions de `beta` donne `alphabetabeta`,
pas `alphbetabeta`. Ce n'est pas un changement de contrat de suppression.

## Candidat vérifié

- Première passe corrigée : 6 tests US5 PASS, compilation 7,40 s.
- Ajout du témoin Ctrl-U/K Unicode : remplacement exact du registre, sans LF.
- L'ancien oracle POLLIN/HUP injectait LF et exigeait Send en double TTY. Seul
  son terminateur est devenu CR ; ordre événement/envoi, fermeture et restauration
  restent inchangés. Première passe attach : 105 PASS / 1 échec sur cet attendu.
- Passe attach finale : 106 PASS / 0 échec, 0,89 s. Les 7 tests US5 y figurent.
- Revue indépendante : APPROVE, séparation double-TTY/Kitty, termios, CR/LF,
  décodage fermé, mutualisation des bornes de mots et registre UTF-8 vérifiés.
- Formatage workspace et clippy workspace/all-targets `-D warnings` : PASS.

## Recette grille réelle

Binaire debug candidat, observateur attach de l'agent vivant
`a4d12c75-5994-4c02-9acc-2db07ba817af`, terminal tmux privé 80×24.
Scénario `/tmp/b92classic.YpV88G/qa-grid.sh`, résultat
`/tmp/b92classic.YpV88G/qa-grid.log` : PASS.

Injection de LF brut, comme le raccourci SendText iTerm, sans aucun CR : première
ligne `alpha beta` conservée, seconde ligne Unicode éditée. Ctrl-A/E/W/Y/U/K,
Option-Backspace/D vérifiés sur les lignes de grille et positions de curseur.
Deux yanks, remplacements de registre et restauration exacte du texte observés.
Observateur fermé par Ctrl-C, serveur tmux privé absent ensuite ; aucun Send,
aucun arrêt/redémarrage de l'agent ni du daemon. Les tests PTY relient en plus LF
et CR lus après la garde termios au vrai handler et à une socket de test.

Ce témoin reproduit le signal configuré ; il ne prétend pas avoir appuyé sur le
clavier de l'utilisateur. Aucun réglage iTerm n'a changé.

## Consolidation et installation

- `cargo test --workspace -- --test-threads=1` : code 0, 71 suites,
  1251 PASS / 0 FAIL / 47 ignorés. Une seule consolidation finale, mode séquentiel
  pour les harnais partageant des options globales. Log `/tmp/b92classic.YpV88G/workspace.log`.
- Release optimisé compilé dans le target du worktree en 29,65 s.
- Même recette grille rejouée avec ce release : PASS, log
  `/tmp/b92classic.YpV88G/qa-release.log` ; observateur privé fermé.
- Remplacement atomique de
  `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget`.
  Comparaison octet pour octet avec le release testé, puis empreinte du chemin
  usuel `/Users/moi/.local/bin/bridget` :
  `dc802c1e8f904317411483464a5804900804d03ec540889a92e83ff08331f7c2`.
- Ancien binaire conservé à
  `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/.install-attach-us5.6gT3TY/bridget.previous`.
- Skill canonique active alignée exactement sur celle du worktree. Aucun commit,
  changement de configuration iTerm ou redémarrage de daemon/agent.
- Le module Markdown reste inchangé : SHA-256
  `3b737c7462197eff39b2da43547c356ab641a98ef1905c8b18646fdca4e9dab1`.

Une vue attach déjà ouverte conserve l'ancien exécutable ; Ctrl-C puis réouverture
charge cette version sans arrêter l'agent observé.
