# Implémentation 092 — clavier et historique attach

## Complément US5 — installé le 06/09/2026

CR envoie et LF ajoute une ligne en double TTY, avec conservation locale des
octets CR/LF par la garde termios. Ce chemin couvre le raccourci SendText LF
configuré dans iTerm sans modifier ses préférences. Ctrl-A/E/U/K/W/Y,
Option-Backspace/D et le registre de dernière suppression sont disponibles.
Option+Entrée reste inerte. Détails, preuves PTY/grille et empreinte installée dans
`verification-us5.md` : 106 tests attach, 1251 tests workspace réussis, zéro échec,
fmt/clippy verts, release installé sans redémarrage. Les résultats ci-dessous
restent les preuves historiques US4/US1–3, non une validation implicite US5.

## Complément US4 — 06/09/2026

Le mandat complémentaire remplace le raccourci historique Alt+Entrée par
**Shift+Entrée seulement**. Option+Entrée et Échap puis Entrée sont désormais
ignorés, sans envoi. Gauche/Droite et Option+Gauche/Droite déplacent le curseur ;
insertion, retour arrière et LF opèrent à cette position. L'historique restaure
le brouillon avec son curseur. Le texte UTF-8 n'est publié au renderer qu'après
validation du caractère complet, avec un snapshot texte/position cohérent.

La validation pilote actuelle est consignée dans `verification-us4.md` :
10 tests US4 et 105 attach verts, fmt/clippy verts, revue indépendante APPROVE,
grilles d'un attach réel vérifiées. La consolidation et l'installation y sont
suivies séparément. Les paragraphes ci-dessous sont le rapport **historique**
T001–T010 : leurs mentions d'Alt+Entrée et de blocage des tests ne décrivent pas
le nouveau contrat US4. Le resize093 reste explicitement hors de ce complément.

## Rapport historique T001–T010

Date : 2026-09-06. Auteur du diff d'implémentation : équipier désigné par le pilote.
Périmètre : T001 à T007 et convergence T010 ; validation ciblée T008 tentée sans
élargissement de droits.

## Réalisation

- Décodeur clavier incrémental privé, limité à 32 octets stockés par séquence.
- Shift+Entrée via CSI-u et `modifyOtherKeys`, avec maintien d'Alt+Entrée et
  d'Échap enrichi suivi d'Entrée.
- Entrée, retour arrière, Ctrl-C et Ctrl-D bruts ou CSI-u normalisés ; relâchements,
  séquences inconnues et séquences surdimensionnées ignorés sans injection de texte.
- Push Kitty `ESC[>1u` et pop `ESC[<u` possédés par `RawTerminal`, uniquement avec
  stdin et stdout TTY sur terminal non `dumb`, sans requête de capacité ni
  préférence globale. L'émission passe par stdout afin que stdin puisse rester
  ouvert en lecture seule ; un échec de push tente le pop puis restaure termios.
- Navigation Haut/Bas CSI ou SS3 uniquement avec stdin et stdout TTY, brouillon
  restauré, édition détachée des entrées rappelées et historique maintenu pendant
  les reconnexions d'une même ouverture. Hors double TTY, les flèches sont
  consommées sans modifier le tampon ni créer d'événement renderer.
- Historique volatile de 100 entrées et 1 048 576 octets, sans déduplication.
  Un corps dépassant seul la borne reste envoyable mais n'est pas mémorisé.
- Mémorisation après écriture réussie uniquement, pour `Send` et `/model` valide.
- Renderer 091, statut, Unicode, multiligne, redimensionnement et contrôle `/model`
  conservés ; aucune trame, dépendance ou persistance ajoutée.

## Preuves ajoutées

Les dix-huit tests internes `spec092_*` couvrent le vrai `handle_input_byte` avec
`UnixStream::pair`, les séquences injectées octet par octet, CSI/SS3, brouillon,
édition, UTF-8, bornes 100/1 Mio, écriture échouée, message géant et nouvelle
instance vide. Les coutures pseudo-TTY couvrent push/pop, stdout redirigé, stdin
ouvert en lecture seule, restauration après échec d'activation, Ctrl-C, EOF, erreur
de boucle et Shift+Entrée lu depuis le terminal raw ; une couture renderer vérifie
tampon multiligne, statut, couleur et position finale du curseur. Une couture
`drive_interactive` sur deux
connexions prouve que la même invocation conserve l'historique alors qu'un nouveau
tampon d'invocation repart vide.

Commande de forme exécutée avec succès :

```text
PATH=/Users/moi/.cargo/bin:$PATH cargo fmt --all --check
```

Commande ciblée tentée :

```text
PATH=/Users/moi/.cargo/bin:$PATH cargo test -p bridget-daemon attach::tests::spec092_shift_entree_csi_u_et_modify_other_keys_inserent_des_lf_sans_envoyer -- --exact
PATH=/Users/moi/.cargo/bin:$PATH cargo test -p bridget-daemon attach::tests::spec092 -- --nocapture
```

La compilation n'a pas commencé : le sandbox refuse la création du répertoire
temporaire Rust sous `/var/folders/.../T/rustc...` avec `Operation not permitted`.
Les tests ciblés, clippy et la consolidation workspace restent donc à exécuter par
le pilote dans son environnement autorisé. `git diff --check` est vert.

## Limites et recette humaine

Une injection pseudo-TTY atteste les octets et la couture logicielle, pas une
frappe physique. Si le terminal transmet les mêmes octets pour Entrée et
Shift+Entrée, Bridget ne peut pas les distinguer ; Alt+Entrée reste le repli.
`TERM=dumb` ou une sortie redirigée n'active pas CSI-u. L'historique disparaît à
la fermeture d'attach et n'est jamais écrit sur disque, au ledger ou chez le
fournisseur.

### Self-review Article XIX/XX

- Nécessité : les touches enrichies exigent un état de décodage entre lectures ;
  le rappel exige un index, un brouillon et deux bornes de rétention.
- Minimalisme : extension des quatre points 091, aucune dépendance, couche publique,
  voie réseau, option ou renderer supplémentaire.
- Hypothèses : les terminaux compatibles suivent les formes du contrat clavier ;
  les terminaux anciens ignorent le push Kitty et gardent Alt+Entrée.
- Vérifié : lecture complète des artefacts, rustfmt, contrôle whitespace et revue
  ciblée des mutations, de la restauration et des limites mémoire.
- Non vérifié ici : compilation/tests/clippy, bloqués par le TMPDIR du sandbox ;
  frappe physique, réservée à la recette humaine.
- Code évité : éditeur de ligne complet, persistance, négociation bloquante,
  activation globale `modifyOtherKeys` et nouveau contrôle modèle.
- Complexité : O(1) par octet avec 32 octets maximum de séquence ; rappel O(taille
  du corps) ; éviction O(nombre et taille des seules entrées retirées).
