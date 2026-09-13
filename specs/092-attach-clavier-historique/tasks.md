# Tâches 092 — Clavier et historique attach

## US5 — tableau classique et LF iTerm, installé

- [x] T016 [US5] Fixer le contrat complet, relever GlobalKeyMap ShiftReturn SendText LF et l'envoi humain ; FR-012/013/014, SC-010/011/012. Aucun réglage global modifié.
- [x] T017 [US5] Oracles rouges PTY ICRNL CR/LF et commandes classiques au vrai handler ; compilation pilote 4,16 s, 6 tests exécutés / 6 échecs attendus avant patch production. Log externe `/tmp/b92classic.YpV88G/red.log`.
- [x] T018 [US5] Implémenter garde CR/LF double-TTY, Ctrl-A/E/U/K/W/Y, Option-Backspace/D et registre de dernier fragment par les primitives communes ; matrice Unicode/historique/non-TTY. 7 tests US5 dans la passe attach finale : 106 PASS ; revue indépendante APPROVE.
- [x] T019 [US5] Vérifier grille réelle, restauration termios, non-régression attach, docs FR/EN/skill, revue ciblée ; final workspace/fmt/clippy puis release/install sans restart. Workspace : 1251 PASS / 0 FAIL / 47 ignorés ; 71 suites. Grille debug et release PASS. Installation atomique du release SHA-256 dc802c1e8f904317411483464a5804900804d03ec540889a92e83ff08331f7c2 ; aucun redémarrage.

## Réouverture autorisée — US4, édition au curseur

Le « go » utilisateur autorise le lot clavier décrit juste avant, pas une reprise
du resize 093. Les validations historiques ci-dessous ne valident pas US4.

- [x] T011 [US4] Fixer spec/plan/contrat et ownership ; conserver le WIP resize à part.
- [x] T012 [US4] Ajouter les oracles rouges de décodage/navigation/insertion/LF et suppression du raccourci Option+Entrée ; transmettre le filtre exact au pilote (FR-009/010, SC-007). Rouge pilote observé : 5 tests exécutés, 5 échecs attendus, compilation4,98 s.
- [x] T013 [US4] Étendre InputBuffer et le décodeur existants : curseur UTF-8, caractères/mots, insertion/backspace au curseur, brouillon historique et reconnexion (FR-009/010, SC-007/008). 10 tests US4 verts, contre-revue approuvée.
- [x] T014 [US4] Raccorder snapshot cohérent et projection visuelle du curseur, fenêtre multiligne, statut et double-TTY ; PTY/grille réelle (FR-011, SC-009). Binaire candidat attach réel, grilles 80×24 et 12×24, texte et position physique vérifiés sans Send.
- [x] T015 [US4] Aligner README FR/EN, skill et recette ; revue indépendante, tests ciblés, fmt/clippy, consolidation finale ; isoler le delta resize non livré avant release et installation vérifiée. Workspace séquentiel : 1244 PASS/0 FAIL/47 ignorés ; release installé par remplacement atomique, SHA-256 cbe1a886d3254ea9637b1abc0738b136ea0a940464e3a96f87066ffc72315f1b. Aucun redémarrage.

Préconditions : reuse-audit PASS, checklist de préparation complète, analyse sans
bloquant. Pilote = artefacts/contrôle ; équipier = implémentation. Aucun commit automatique.
Toutes les tâches sont séquentielles (même fichier). Base ea52cd04.

Clôture fonctionnelle le 2026-09-06 à 14:40 CEST, après convergence puis
validation indépendante ; preuves dans verification.md. Exception procédurale
explicite pour T001 : le rouge initial avant implémentation n'a pas pu être
observé (TMPDIR du développeur refusé). La case signifie tests implémentés et
fonction vérifiée ; elle ne revendique pas cette observation historique.
Recette physique, commit et installation restent hors des actions automatiques.

## Phase 1 — Contrat clavier (US1)

- [x] T001 [US1] Ajouter dans crates/bridget-daemon/src/attach.rs les tests `spec092` au point handle_input_byte : Shift+Entrée CSI u/modifyOtherKeys fragmenté, Alt+Entrée, Entrée et contrôles ; prouver zéro Send avant Entrée puis corps multiligne exact, observer le rouge initial (FR-001/003, SC-001).
- [x] T002 [US1] Remplacer alt_prefix par le décodeur privé incrémental borné, réutiliser InputBuffer/handle_input_byte ; tester séquence inconnue, surdimensionnée, Ctrl-C/Ctrl-D et UTF-8 sans suffixe injecté ni double envoi (FR-001/003, SC-001/004).

## Phase 2 — Durée de vie terminal (US1/US3)

- [x] T003 [US1] Étendre la garde locale de crates/bridget-daemon/src/attach.rs pour push/pop clavier TTY non dumb, sans attente ni préférence globale. Réutiliser PseudoTerminal : restauration succès/Ctrl-C/EOF/erreur, double restauration inerte, non-TTY/dumb sans protocole (FR-002, SC-004).

## Phase 3 — Historique local (US2)

- [x] T004 [US2] Ajouter les tests discriminants A/B/brouillon avec Haut/Bas CSI/SS3, limites, édition et UTF-8/multiligne ; étendre InputBuffer avec historique volatile et navigation via le renderer existant (FR-004, SC-002).
- [x] T005 [US2] Raccorder la mémorisation au seul succès de write_socket_message pour Send et SelectRuntime valide ; tester échec/invalidité/vide, 101 entrées, dépassement de 1 Mio, message géant, état conservé lors d'une reconnexion de la même invocation et nouvelle invocation vide ; historique TUI seulement, sortie redirigée sans rappel/ANSI (FR-005/006, SC-003).

## Phase 4 — Couture, documentation, validations

- [x] T006 [US3] Exécuter et compléter la couture PTY de crates/bridget-daemon/src/attach.rs : touches pendant le flux, statut sous saisie, couleurs/NO_COLOR, largeur/curseur et `/model` contrôle sans Send ; conserver les oracles 091 (FR-007, SC-005).
- [x] T007 [US3] Aligner README.md, README.en.md et skills/bridget/SKILL.md sur les touches, historique local, limites et modèle déjà disponible ; rédiger specs/092-attach-clavier-historique/implementation.md avec preuves/limites sans revendiquer une frappe physique non observée (FR-008, SC-006).
- [x] T008 Valider les tests attach ciblés, fmt, clippy workspace/all-targets puis une consolidation cargo test --workspace ; produire un candidat release. Le pilote peut lancer les validations interdites au shell de l'équipier sans modifier ses droits. Aucun déploiement/commit ; consigner sorties, décomptes et durées dans implementation.md (SC-004/005/006).

## Phase 5 — Contrôle final du pilote

- [x] T009 Revue du diff, analyse de convergence FR/SC et audit final (sécurité/minimalisme/complexité/responsabilité). Si lacune : ajouter uniquement de nouvelles tâches de correction et redéléguer. Vérifier checklist complète avant livraison ; ne pas prétendre à une revue cross-provider si aucun fournisseur distinct disponible (SC-001 à SC-006).

## Couverture et budget recalibré

FR-001 T001/T002 ; FR-002 T003 ; FR-003 T001/T002 ; FR-004 T004 ; FR-005/006 T005 ;
FR-007 T006 ; FR-008 T007. SC-001 T001/T002 ; SC-002 T004 ; SC-003 T005 ; SC-004
T002/T003/T008 ; SC-005 T006/T008 ; SC-006 T007/T008/T009.
Estimation restante à génération : développement 12–20 min, tests/revue 8–15 min,
hors durée de consolidation workspace et arbitrage humain. Aucun changement de modèle imposé.

## Convergence — cycle 1 (14:18)

- [x] T010 [US1/US2] Corriger dans attach.rs le test de navigation qui détruit son pair UnixStream (`let (write_stream, _)`) avant Send ; prouver les cas SC-004 TTY stdin en O_RDONLY avec stdout TTY (activation par sortie writable), stdin TTY + stdout redirigé sans activation/rappel/ANSI, restauration sur échec d'activation et sorties contrôlées. Ajouter un oracle de reconnexion via le chemin d'invocation/drive_interactive conservant le même historique et une nouvelle invocation vide (FR-002/006, SC-003/004). Refuser aussi les types d'événement CSI u inconnus plutôt que de les traiter comme pressions. Correction par l'équipier, aucune régénération du fichier ; puis validation pilote.
