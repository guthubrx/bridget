# Tâches 036 — Observer la prise d'un mandat

- [x] T001 Mesurer les quatre producteurs de `turn_start`, les traces natives
  Codex et Claude, le spécimen rc5 et les distributions de délai.
- [x] T002 Figer la portée locale, les quatre états, le seuil borné et la limite
  inter-machine dans `specs/036-observer-prise-mandat/spec.md`.
- [x] T003 [US1] Ajouter le lecteur local provider-aware et son diagnostic de
  source dans `scripts/bridget-idle.py`.
- [x] T004 [US1] Intégrer `MANDAT_NON_SOUMIS` à la partition texte et JSON dans
  `scripts/bridget-idle.py`.
- [x] T005 [US3] Intégrer `PRISE_INOBSERVABLE` avec cardinal explicite dans
  `scripts/bridget-idle.py`.
- [x] T006 [US1] Ajouter les deux formes visuelles du spécimen bloqué et le
  mutant absence→sain dans `scripts/test-bridget-idle.sh`.
- [x] T007 [US2] Ajouter le contrôle sain opposé et le mutant injection→bloqué
  dans `scripts/test-bridget-idle.sh`.
- [x] T008 [US4] Distinguer `enqueue` et `remove` Claude dans
  `scripts/test-bridget-idle.sh`.
- [x] T008b Prouver qu'une copie Maicie locale vide ne masque pas une
  injection non soumise vue par le daemon.
- [x] T008c [US5] Rejouer les spécimens réels `cartae0` et `rc7` pour séparer
  steering et injection au repos sans faux positif ni faux négatif.
- [x] T009 Exécuter les gates base/tête, les mutants et un tir réel local, puis
  documenter les preuves dans `specs/036-observer-prise-mandat/implementation.md`.
- [x] T010 Relire le diff, rendre le matériel et livrer une tête locale et
  distante identiques.
