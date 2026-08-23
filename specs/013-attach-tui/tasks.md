# Tasks 013 — Vue attach lisible

**Base** : branche `session-13-attach-tui` depuis `main` (7f91f67).
**Règles** : celles de `docs/regles-chantier.md` + validations avant commit,
commit en review immuable, auteur ≠ relecteur, zéro trace IA, jamais de push.

- [x] T1301 Renderer de blocs (D-1301, D-1303, D-1304, D-1306) : tour rendu
  en place (ANSI), clé de bloc session_id+message_id, tampon borné 64Kio/400
  lignes à troncature visible, un seul thread écrivain stdout (zéro I/O sous
  le verrou screen — corrige attach.rs:1017) ; matrice stdin×stdout 4 cas,
  repli non-TTY intégral.
  **Observable** : golden stdout non-TTY byte-à-byte (4 cas), test bascule
  replay→live sans doublon/trou, démo TTY.

- [x] T1302 Capture ET étiquetage des outils (D-1302, D-1307, D-1308) : le
  mapping title→name→kind est capturé AU JOURNAL (acp.rs — champ additif
  compatible v1), le renderer l'affiche sanitizé (Cc/Cf/ANSI), kinds
  inconnus inclus ; exception golden versionnée pour ce délta transport.
  **Observable** : fixture hostile (Read/Bash/kind-inconnu/titre ESC-OSC-bidi)
  → lignes étiquetées et assainies ; constat n° 11 clos À LA SOURCE.

- [x] T1303 Rattrapage compact : historique rendu en blocs clos (zéro
  re-streaming), bascule rattrapage→live signalée, Gap/End inchangés.
  **Observable** : fixture de rattrapage → blocs compacts ; bancs 008
  intacts (SC-003).

- [ ] T1304 Cohabitation saisie/flux : ligne de saisie ancrée en bas, le
  flux se rend au-dessus sans corrompre la frappe ; redimensionnement
  toléré.
  **Observable** : test pseudo-TTY réel (saisie pendant un tour entrant).

- [ ] T1305 Mesure et finition : SC-001 mesuré en 80×24 sur corpus
  déterministe 3 tours (lignes comptées avant/après, timeout global),
  README section vue attach mise à jour, DEPRECATIONS relu.
  **Observable** : chiffres consignés dans implementation.md.

## Ajout en cours de session

- [ ] T1305b Constat no 17 (observation utilisateur) : message d'erreur
  attach incohérent pour un équipier ARRÊTÉ — dit « inconnu » en le listant
  comme attachable. Correctif : distinguer inconnu/arrêté (« équipier arrêté,
  historique consultable via … » le cas échéant) et ne lister comme
  attachables que les vivants (ou afficher leur état). À absorber avec T1305.
