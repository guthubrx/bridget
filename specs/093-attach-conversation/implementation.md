# Implémentation 093 — installé, correction resize ouverte

## Contre-recette resize — 06/09, 19:38 CEST

Le candidat de correction n'est PAS installé. Binaire actif inchangé :
`60a54a6bd7da1c72fdfd8df0eacac6d95248ad5c1c2ef41bb99cbd55de2df831`.
Ne pas confondre les étapes historiques plus bas avec l'état actuel.

- Premier correctif : fond gris par EL plutôt que padding imprimable et
  recalcul des rangées du footer après resize. Recette simple 100→45→160 verte.
- Contre-recette répétée : 20 cycles avec statut long produisent 16 headers ;
  ce résultat ROUGE prime sur les validations unitaires. Aucune installation.
- Recette lente, quatre cycles sans course de géométrie : curseur à largeur160
  y=29,26,23,20 ; history_size=18,21,24,27 ; blancs sous footer=4,7,10,13.
  L'expiration du statut et ses styles ne sont donc pas la cause suffisante.
- Barrière réelle avant le premier clear : initial100 cursor22,32 history15 ;
  resize45, AVANT toute écriture du renderer, cursor22,30 history18 ; après
  clear/draw cursor22,30 history18. Le terminal réorganise déjà la grille et
  l'historique avant le repaint. Voir resize-barrier-before-clear.txt et
  resize-barrier-after-clear.txt dans ce dossier. Les 40 marqueurs historiques
  restent présents : déplacement hors écran, pas effacement démontré.
- Course distincte attestée : géométrie utilisée45 mais observée160 après draw
  à1218/2503 ms ; inverse à3015 ms. Traces locales conservées dans
  /tmp/b93resize.M5kVBR/geometry-trace.txt et slow-trace.txt.

Validations du PREMIER candidat (insuffisantes pour livrer) : attach 96 PASS,
1 ignored ; clippy workspace/all-targets -D warnings vert ; fmt/diffcheck verts ;
workspace série vert (dont daemon764 PASS/10 ignored, transport265 PASS/1 ignored) ;
release construit en29,19 s, non installé. Oracle de conservation exacte du
statut coloré ajouté ensuite :1/1 PASS. Une grille terminal réelle reste le gate.

Conclusion de diagnostic : le modèle de lignes relatif ne suit pas une zone
physique attestée ; les blancs abandonnés sous la zone font dériver son ancrage.
Une simple marge droite ou un debounce ne sont pas une garantie de correction.
Pas de second patch de production sans choix d'ancrage vérifiable. T011/T012
restent ouverts. Aucun daemon, fournisseur ou agent utilisateur redémarré.

## Réouverture ciblée — recette humaine du 06/09 à18:59

Markdown/couleurs validés visuellement par l'humain. Capture après élargissement :
répétitions du header humain et du label agent. Le paragraphe antérieur à une
nouvelle question peut légitimement être déjà engagé dans le scrollback ; cela
ne justifie pas les duplications. T011 est rouvert pour ce bug d'affichage.
Hypothèses à vérifier avant correction : clear relatif au curseur après reflow
réel, footer_rows conservé à l'ancienne largeur, padding gris absent du modèle
de rendered_lines. Les tests PTY existants observent les octets mais ne modélisent
pas la réorganisation effective des cellules du terminal. Recette indépendante
dans émulateur tmux isolé /tmp/b93resize.M5kVBR/tmux.sock, uniquement une vue attach,
aucun retour au mode agents tmux ni changement de daemon/fournisseur.
Mandat de diagnostic lié : mcp-67952-6a9d9c53-18, agent23 propriétaire du correctif.


## État et mandat

2026-09-06, début 17:47 CEST ; estimation 32–65 minutes, fin ~18:20–18:52.
Le renderer antérieur ne rendait pas Markdown : aucune livraison093 n'est encore
revendiquée. L'ajout utilisateur resize est inclus au contrat actif.

T001/T002 vérifiés : spécification/plan, reuse-audit PASS, analyse PASS ; scripts
et templates officiels absents constatés, workflow appliqué directement.
Sync utilisateur exécutée sans changement. Aucun autre fournisseur inscrit à
l'annuaire ; Agent23 Sol high écrit, reviewer Codex Sol high indépendant lit.

Mandat d'implémentation envoyé par Bridget : mcp-67952-6a9d8c9a-5,
recipient a4d12c75-5994-4c02-9acc-2db07ba817af. Review interne review_attach_093,
lecture seule. Aucune modification de modèle, daemon ou permissions.

## Base de comparaison

Branche session-093-attach-conversation, même worktree physique091 autorisé.
WIP090/092 conservé. Snapshot pré-093 : /tmp/b93.lzawyK/attach-before.rs,
SHA256 acbc3e690457a27a933e3ab174d6a33dd278efca6894722fb9a881bec22e3f31.
Manifestes pré-093 sauvegardés dans le même répertoire temporaire.

Le binaire actuellement installé reste la version pré-093 :
5e8383ea666a0043ddefedc13085427f0ecf749dd8f407b777b5279e14ee0ebc.
Aucune mise à jour installée n'est déduite de la présence des artefacts.

## Vérifications à exécuter

```sh
PATH=/Users/moi/.cargo/bin:$PATH TMPDIR=/tmp/b93.lzawyK CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target cargo test -p bridget-daemon spec093 -- --nocapture
PATH=/Users/moi/.cargo/bin:$PATH TMPDIR=/tmp/b93.lzawyK CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target cargo test -p bridget-daemon attach::tests -- --nocapture
PATH=/Users/moi/.cargo/bin:$PATH cargo fmt --all --check
PATH=/Users/moi/.cargo/bin:$PATH TMPDIR=/tmp/b93.lzawyK CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target cargo clippy --workspace --all-targets -- -D warnings
PATH=/Users/moi/.cargo/bin:$PATH TMPDIR=/tmp/b93.lzawyK CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target cargo test --workspace --quiet
```

Résultats : en attente du diff compilable. Pas de test revendiqué sur093 à ce stade.
Build release, audit et installation après convergence seulement. Aucune nouvelle
recette nécessitant un compte fournisseur payant. Observation humaine non encore faite.

Premier essai pilote vers 18:08 : commande ciblée spec093 ci-dessus, sortie101
avant les tests. Erreur E0382 dans journal_render_record : event déplacé dans la
structure puis consulté pour reasoning/ordinary_turn_end. Sortie transmise à
l'auteur par mcp-67952-6a9d9010-9. Ne pas confondre ce diagnostic de WIP avec une
validation. Cargo a résolu pulldown-cmark0.13.4 et unicase2.9.0 ; lockfile vérifié.

Second essai vers18:13 : même commande spec093, compilation réussie en15,08s.
6tests passent,1échoue en0,01s : spec093_unicode_et_ansi_hostile_ne_faussent_pas_la_largeur
compare une chaîne ininterrompue à des lignes repliées ; sortie transmise à l'auteur.
Les oracles court-Markdown/label/filtrage/resize passent, mais la revue préventive
reste ouverte et aucun gate de livraison n'est encore franchi.

Cycle2 vers18:23 : candidat annoncé suspendu,9tests093,7PASS/2FAIL.
Suite attach complète après ajout des dumps diagnostiques :89PASS/2FAIL,0,87s.
Donnée ANSI réelle : `界界🙂\n␛[31mrou\nge ␛[2J\n␛]8;;htt\nps://x␇l\nien`.
Le contrôle est bien neutralisé ; l'assertion ignorait encore les séparateurs de
wrapping. Donnée PTY réelle : uniquement80octets d'effacement lors du premier read ;
l'oracle s'arrête avant réception du repaint. Correction seulement après cette
observation, conformément à la règle anti-boucle, sans nouvelle hypothèse de prod.
Les3points de contre-revue (préfixe mutable, terminal non corrélé, continuation
de liste) restent à fermer ; aucun binaire installé sur ces résultats partiels.

## Résultat final — 18:48 CEST

Les états ci-dessus sont le journal chronologique, pas des réserves actuelles.
Les trois points ont été corrigés et contre-relus : APPROVE indépendant
review_attach_093. Label figé après engagement, unités logiques indépendantes
du wrapping, terminal non corrélé visible, indentation de continuation de liste
préservée. Entités de contrôles terminal neutralisées après parsing aussi.

- `cargo test -p bridget-daemon --lib attach:: --quiet` : 95 PASS, 0 FAIL, 0,88 s.
- `cargo fmt --all --check` et `git diff --check` : PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` : PASS, 9,09 s.
- `cargo test --workspace --quiet -- --test-threads=1` : PASS, code 0 ; les tests
  ignorés restent explicitement ignorés (aucune recette fournisseur inventée).
- Build release final séparé du binaire actif : en cours de clôture ci-dessous.

### Exception de validation parallèle, préexistante et reproduite

La première suite workspace en parallèle a échoué sur deux tests CLI :
`statusline_attribue_ses_faits_sans_se_declarer` et
`projection_cli_et_reference_partagent_le_canon_du_daemon_reel` (daemon fermé).
Chacun passe isolément. La collision avec le drapeau global SHUTDOWN_REQUESTED
est reproduite avant093 : binaire de tests
`/Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug/deps/bridget_daemon-ecd4371015c9baea`,
daté du 06/09 à 15:39:31, SHA256
`23ff7a5f35fe5f0605e21666e1290716ca36190ce3e65576a7b7e04c350c1aa9`.
Commande : ce binaire, les deux filtres de noms ci-dessus et
`l_adaptateur_des_fils_de_fond_observe_le_drapeau_d_arret --test-threads 3 --quiet`.
Trois exécutions : 3/3 PASS, 3/3 PASS, puis 1 PASS/2 FAIL reproduisant la panne.
Les fichiers CLI/daemon concernés ne sont pas modifiés par093. Cette dette n'est
pas corrigée dans ce lot ; la suite séquentielle ne la fait pas disparaître.

### Portée honnête

Le resize est exercé par pseudo-TTY, sans frappe, sur la partie encore gérée du
bloc ; aucun reflow intégral du scrollback n'est promis. Pas de coloration lexicale
universelle : Markdown, styles de blocs code et hiérarchie visuelle seulement.
Aucun redémarrage daemon, agent ou changement de permissions/modèle requis.

## Installation effective

Build release final : PASS en26,41s sans warning. Installation atomique exécutée
à18:50CEST environ, SHA256 candidat = binaire actif :
`60a54a6bd7da1c72fdfd8df0eacac6d95248ad5c1c2ef41bb99cbd55de2df831`.
Binaire : `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget`.
Sauvegarde pré093 :
`/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/.install-attach093.Z74aCU/bridget.previous`.
Le raccourci `/Users/moi/.local/bin/bridget` pointe bien vers ce binaire.

Recette réelle : binaire installé + pseudo-TTY100×35 + agent23 vivant, reprise
du journal à11750. Markdown effectivement transformé (titre sans ##, puces•,
blocRust sans fences), corps sans indentationUUID, noms actualisés sur annuaire.
Les raisonnements ne paraissent pas ; les fins ordinaires corrélées ne paraissent
pas. Une fin isolée au début de cette fenêtre reste visible car non corrélée,
conformément au garde-fou. Le premier essai `--seq` a été refusé proprement :
l'option réelle est `--from-seq` ; aucun effet sur l'agent.
Cette recette n'est pas une capture de l'écran physique de l'humain. Les tests
PTY de resize restent la preuve automatique ; pas de redessin universel du scrollback.

Ouvrir : `bridget attach a4d12c75-5994-4c02-9acc-2db07ba817af` après Ctrl-C dans
l'ancien attach. Ctrl-C ne stoppe pas l'agent. Aucun stage, commit ou merge.

Recette couleur réelle complétée : le shell de test hérite NO_COLOR=1, donc
première observation volontairement sans SGR. Nouvelle invocation limitée au
processus de recette : `env -u NO_COLOR TERM=xterm-256color` + binaire installé,
`attach a4d12c75-5994-4c02-9acc-2db07ba817af --from-seq 17330`.
Capture PTY : titre cyan gras (`1;38;5;45`), emphase gras, code clair sur fond236,
saisie pleine largeur fond236 et statut cyan. Aucun `[raisonnement]` ni fin
ordinaire sur ce tour complet. Agent connected, gpt-5.6-sol/high attestés.
Les trois vues de recette ont été fermées par Ctrl-C, sorties0 ; agent préservé.
Audit v14 validate_session : 0 erreur, 0 warning. Skill canonique active publiée
par ajout du seul paragraphe093, sans écraser ses changements antérieurs.
