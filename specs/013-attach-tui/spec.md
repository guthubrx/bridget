# Spec 013 — Vue attach lisible (TUI léger)

**Status**: Clôturée — implémentée, revue (11 constats fermés) et mergée le 2026-08-23

**Origine** : retour utilisateur du 2026-08-23 — le streaming ligne-par-token
est illisible ; attendu : la grammaire visuelle des assistants CLI (bloc qui
se construit en place, tours séparés, outils étiquetés), sans devenir une
application lourde.

## Exigences

- **FR-001 Rendu en place** : le tour en cours s'affiche comme UN bloc qui
  grandit (contrôle curseur ANSI), pas une ligne par delta. Repli automatique
  sur le mode ligne actuel quand stdout n'est pas un TTY (bancs, pipes,
  tests : comportement octet pour octet inchangé — golden test exigé).
- **FR-002 Blocs de tours** : chaque tour a un en-tête (émetteur, heure) et
  une séparation visuelle claire ; le bloc ne se ferme QUE sur `turn_end`
  ou sur un champ terminal explicite de l'événement — jamais sur un simple
  diagnostic `error` non terminal (méthode inconnue en cours de tour) ; test
  exigé : diagnostic puis update dans le MÊME bloc.
- **FR-003 Étiquetage des outils** : afficher le nom/titre réel porté par
  l'événement ACP (`Read x.rs`, `Bash cargo test…`), y compris pour les
  kinds inconnus (clôt le constat recette n° 11) ; une ligne compacte par
  appel, statut accordé/refusé.
- **FR-004 Saisie ancrée** : la ligne de saisie (raw mode 008) reste en bas ;
  le flux se rend au-dessus sans se mélanger à la frappe.
- **FR-005 Rattrapage compact** : l'historique rejoué se rend en blocs déjà
  clos (jamais de re-streaming token par token) ; la bascule
  rattrapage→live reste signalée.
- **FR-006 Sobriété** : largeur du terminal respectée (repli propre en
  redimensionnement), pas de dépendance TUI lourde nouvelle sans arbitrage
  (préférence : ANSI direct ; une dépendance minimale type largeur-terminal
  se justifie par écrit).

## Hors périmètre

Couleurs thématisables, souris, panneaux multiples, historique scrollable
interne (le scrollback du terminal suffit).

## Critères mesurables

- SC-001 : sur un échange réel (équipier géré, 3 tours dont un appel
  d'outil), la sortie TTY tient en ≤ 1 écran lisible là où l'actuelle
  produit > 60 lignes.
- SC-002 : chemin de rendu non-TTY inchangé octet pour octet sur la fixture
  existante, à l'exception du seul delta payload FR-003, versionné.
- SC-003 : les bancs 008 (SC-001/002/005) restent verts sans modification.
