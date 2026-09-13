# Vérification complément US4 — livré

Mandat humain « go », 06/09/2026. Agent23 Sol high écrit ; pilote contrôle.
Entrée : Send ; Shift+Entrée : LF au curseur ; Option+Entrée : aucun effet.
Flèches horizontales et Option+flèches, insertion/backspace, historique et affichage.
Le resize093 reste ouvert et séparé, pas de promesse de correction dans ce lot.

## Rouge initial réellement observé

Commande depuis /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/091-communication-agent-ux :

```sh
PATH=/Users/moi/.cargo/bin:$PATH TMPDIR=/tmp/b92edit.7Lmqcg CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target cargo test -p bridget-daemon --lib attach::tests::spec092_us4_ -- --nocapture
```

Compilation4,98 s ; 5 tests exécutés en0,00 s, 5 échecs attendus :
abXY au lieu de aXbY ; finX au lieu de deux,troisX fin ; LF à la fin plutôt
qu'au curseur ; Option+Entrée insère LF ; restauration brouillon sans position.
Snapshot diagnostic : /tmp/b92edit.7Lmqcg/attach-us4-red.rs.

## Contre-validation du candidat

10 tests `attach::tests::spec092_us4_` exécutés et verts ; ensuite les 105 tests
`attach::` verts (aucun ignoré). `cargo fmt --all --check` et
`cargo clippy --workspace --all-targets -- -D warnings` verts.

La contre-revue indépendante a fait corriger le parcours mots quadratique,
l'insertion prématurée/invalide UTF-8, et la position du curseur au premier
caractère d'une ligne repliée. Verdict ciblé final : APPROVE.

Recette interprétée dans un émulateur tmux isolé, uniquement comme harnais :
binaire debug candidat, vraie connexion attach vers l'agent vivant
a4d12c75-5994-4c02-9acc-2db07ba817af ; aucune touche d'envoi, aucun message
de mission. Deux ouvertures à taille fixe (le resize reste hors lot).

- 80×24 : `abc`, Gauche, `X` donne `> abXc`, curseur (5,21).
- Droite, ` deux trois`, Option+Gauche, `X` donne `> abXc deux Xtrois`,
  curseur (13,21). Le premier calcul manuel attendu était erroné de 1 ;
  le comptage exact inclut les 2 cellules de l'invite et les 11 du préfixe.
- Option+Droite puis ESC-b, CSI 13;2u, `Y`, ESC-CR : grille
  `> abXc deux` puis `YXtrois`, curseur (1,21). Shift insère LF au curseur,
  Option+Entrée ne modifie ni texte ni position et n'envoie rien.
- Backspace, `é🙂`, Gauche, `Z` : `éZ🙂Xtrois`, curseur (2,21).
- 12×24 : `abcdefghijk`, Gauche : `> abcdefghij` puis `k`, curseur (0,16).
  `X` donne `Xk` et curseur (1,16), sans décalage sur la marge précédente.
- Statut sous la saisie présent dans chaque grille. Ctrl-C ferme seulement
  les deux observateurs ; aucun agent/daemon redémarré.

Ce sont des séquences injectées dans un vrai terminal interprété et un vrai
attach, pas une frappe physique dans l'application terminal de l'humain.

## État de livraison

`cargo test --workspace -- --test-threads=1` : sortie 0, 71 suites,
1244 PASS, 0 FAIL, 47 ignorés explicitement. Journal :
/tmp/b92edit.7Lmqcg/workspace-tests.log.

Release optimisé construit en 27,49 s dans le target du worktree, puis copié et
installé par remplacement atomique de
/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget.
Le raccourci /Users/moi/.local/bin/bridget pointe toujours sur ce fichier.
Comparaison octet à octet avec le candidat release : identique.
SHA-256 : cbe1a886d3254ea9637b1abc0738b136ea0a940464e3a96f87066ffc72315f1b.
Smoke du release installé, attach réel en 80×24 : `abc`, Gauche, `X`,
Shift+Entrée, `Z` donne `> abX` puis `Zc`, curseur physique (1,21).
Contrôle sur grille interprétée, puis Ctrl-C du seul observateur.

Ancien exécutable récupérable :
/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/.install-attach-us4.02qXFN/bridget.previous
(SHA-256 60a54a6bd7da1c72fdfd8df0eacac6d95248ad5c1c2ef41bb99cbd55de2df831).
Skill canonique active synchronisée avec le contrat US4, liens globaux inchangés.
Aucun stage/commit, modification de configuration ou redémarrage d'agent/daemon.

Le candidat rejeté resize est archivé dans
/tmp/b93resize.M5kVBR/attach-paused-resize-before-us4.rs ; ses deux hunks de
production ont été retirés avant US4. Le module attach_renderer.rs est inchangé
octet pour octet par ce complément. Les défauts resize093 restent ouverts,
sans revendication de correction par cette livraison clavier.
