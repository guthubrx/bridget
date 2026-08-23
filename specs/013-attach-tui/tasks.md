# Tasks 013 — Vue attach lisible

**Base** : branche `session-13-attach-tui` depuis `main` (7f91f67).
**Règles** : celles de `docs/regles-chantier.md` + validations avant commit,
commit en review immuable, auteur ≠ relecteur, zéro trace IA, jamais de push.

- [ ] T1301 Renderer de blocs : état « tour courant » rendu en place (ANSI),
  en-têtes de tour, clôture sur end_turn/erreur ; détection TTY et REPLI
  intégral sur le mode ligne actuel en non-TTY.
  **Observable** : golden non-TTY octet pour octet (SC-002) ; démo TTY.

- [ ] T1302 Étiquetage des appels d'outils : nom/titre réel extrait de
  l'événement ACP, kinds inconnus inclus, ligne compacte + statut.
  **Observable** : fixture avec Read/Bash/kind-inconnu → trois lignes
  étiquetées ; le constat n° 11 est clos.

- [ ] T1303 Rattrapage compact : historique rendu en blocs clos (zéro
  re-streaming), bascule rattrapage→live signalée, Gap/End inchangés.
  **Observable** : fixture de rattrapage → blocs compacts ; bancs 008
  intacts (SC-003).

- [ ] T1304 Cohabitation saisie/flux : ligne de saisie ancrée en bas, le
  flux se rend au-dessus sans corrompre la frappe ; redimensionnement
  toléré.
  **Observable** : test pseudo-TTY réel (saisie pendant un tour entrant).

- [ ] T1305 Mesure et finition : SC-001 mesuré (échange réel ≤ 1 écran),
  README section vue attach mise à jour, DEPRECATIONS relu.
  **Observable** : chiffres consignés dans implementation.md.
