# Taches d'implementation - SPEC-073 Piloter un agent depuis sa fiche

## Phase 1 - Preparation et preuve de base

- [x] T001 Executer les suites de base `node --test crates/bridget-daemon/assets/ui/app.js`, `cargo test -p bridget-daemon ui::` et `cargo test -p bridget-daemon --test ui_relay_test` depuis `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui`, puis consigner les resultats et le commit de depart dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/specs/073-actions-agent-ui/evidence/baseline.md`

## Phase 2 - Fondations partagees

- [x] T002 Ajouter dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/src/ui.rs` les tests SPEC-073 qui prouvent la projection distincte de `persistent=Some(true)`, `persistent=Some(false)` et `persistent=None` dans `UiAgentRowV1`
- [x] T003 Etendre `UiAgentRowV1` et `compose_agent_rows` dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/src/ui.rs` pour projeter `persistent: Option<bool>` sans inference, avec passage des tests T002
- [x] T004 Ajouter dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/src/ui.rs` les tests SPEC-073 du contrat versionne, des champs inconnus, des validations `name` et `command_id`, des refus absent/arrete/non gere et du mapping de chaque `StopOutcome`
- [x] T005 Implementer dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/src/ui.rs` la route locale `POST /v1/agents/stop`, la relecture defensive de l'annuaire et l'adaptation stricte vers `WrapperToDaemon::StopOrder`, sans signal systeme ni logique de cycle de vie parallele

## Phase 3 - User Story 1 : ouvrir volontairement la fiche

Objectif : les trois points ouvrent une seule fiche sans selectionner l'agent ni modifier le fil.

Test independant : survol, focus et selection de ligne n'ouvrent rien ; souris et clavier sur les trois points ouvrent et ferment la fiche avec restitution du focus.

- [x] T006 [US1] Ajouter dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/assets/ui/app.js` les tests Node SPEC-073 couvrant absence d'ouverture au survol/focus, controle frere, non-selection, `aria-expanded`, Echap, clic exterieur, restitution du focus et 100 ouvertures locales sous 150 millisecondes sans requete reseau
- [x] T007 [US1] Refactorer `renderAgentButton` dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/assets/ui/app.js` en enveloppe neutre avec bouton de selection et bouton a trois points freres, y compris quand la date d'activite est absente
- [x] T008 [US1] Remplacer dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/assets/ui/app.js` le controle au survol de la fiche globale par un controle explicite non modal `role="dialog"`, avec fermeture unique, clic exterieur, Echap et focus coherent
- [x] T009 [US1] Adapter dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/assets/ui/theme.css` la ligne, l'alignement des trois points, les etats focus/hover et le panneau afin que le nom, l'etat et la date restent lisibles

## Phase 4 - User Story 2 : conserver l'identite complete

Objectif : le panneau volontaire conserve exactement les faits, logos et modes de SPEC-071.

Test independant : ouvrir les cas Codex, Claude Code, Cursor, Gemini CLI, inconnu, TMUX et FLUX et verifier les valeurs attestees sans inference par nom.

- [x] T010 [P] [US2] Etendre dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/assets/ui/app.js` les tests SPEC-071/073 de `identityCardData`, des quatre logos locaux, de l'inconnu, des modes et de l'absence d'inference depuis le nom
- [x] T011 [US2] Faire evoluer le rendu de la fiche dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/assets/ui/app.js` pour conserver tous les faits SPEC-071, ajouter un titre nomme et un bouton de fermeture sans dupliquer le catalogue runtime
- [x] T012 [US2] Finaliser dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/assets/ui/theme.css` le panneau integre, sans fond ou bordure parasite autour des logos, utilisable en 1280x720 et a 200 pour cent de zoom

## Phase 5 - User Story 3 : decommissionner un agent gere

Objectif : seule une gestion attestee autorise l'action, toujours apres confirmation explicite.

Test independant : annuler produit zero ordre, confirmer un agent gere produit un seul ordre, un agent externe, TMUX non gere ou arrete reste ineligible.

- [x] T013 [P] [US3] Ajouter dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/assets/ui/app.js` les tests SPEC-073 de l'eligibilite fondee uniquement sur `persistent`, avec explication pour agent non gere ou arrete
- [x] T014 [US3] Ajouter dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/assets/ui/app.js` les tests de confirmation : nom exact, historique conserve, avertissement de tour actif, focus initial sur Annuler, annulation sans requete et confirmation unique
- [x] T015 [US3] Implementer dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/assets/ui/app.js` l'action `Decommissionner`, son indisponibilite expliquee et la confirmation `role="alertdialog"` sans traitement special par nom d'agent
- [x] T016 [US3] Implementer dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/assets/ui/app.js` le confinement du focus de la confirmation, Echap vers Annuler, restitution du focus et resistance au rerendu qui retire le declencheur du DOM
- [x] T017 [P] [US3] Ajouter dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/assets/ui/theme.css` les styles de l'action, de l'explication et de la confirmation modale avec indication independante de la couleur

## Phase 6 - User Story 4 : afficher le verdict reel

Objectif : l'interface attend le daemon, bloque les doublons et distingue chaque issue sans transition optimiste.

Test independant : simuler arret propre, arret force, non gere, absent, timeout et daemon indisponible ; verifier le message exact et l'absence de faux etat `stopped`.

- [x] T018 [US4] Ajouter dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/assets/ui/app.js` les tests SPEC-073 du corps HTTP, du blocage d'une demande en vol, des six issues et de l'absence de mutation optimiste de `state.agents`
- [x] T019 [US4] Implementer dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/assets/ui/app.js` l'appel versionne avec identifiant de correlation stable par confirmation, l'etat `submitting` et le rendu exhaustif des verdicts et erreurs
- [x] T020 [US4] Raccorder dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/assets/ui/app.js` le succes au prochain snapshot atteste, sans deplacer localement l'agent, et conserver l'historique consultable dans la section des agents arretes
- [x] T021 [US4] Etendre `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/crates/bridget-daemon/tests/ui_relay_test.rs` avec les tests HTTP SPEC-073 obligatoires de methode, token, version, succes et erreur de protocole, en utilisant une socket daemon isolee et sans toucher au daemon de production

## Phase 7 - Validation, maintenance et preuves

- [x] T022 Executer `git diff --check`, `node --test crates/bridget-daemon/assets/ui/app.js`, `cargo test -p bridget-daemon ui::`, `cargo test -p bridget-daemon --test ui_relay_test` et les tests `StopOrder` existants depuis `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui`, puis corriger jusqu'a convergence complete
- [x] T023 Clore le parcours navigateur manuel à la demande explicite de l'utilisateur et consigner son remplacement par les tests automatisés DOM, clavier, HTTP et socket daemon isolée dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/specs/073-actions-agent-ui/evidence/validation.md`
- [x] T024 Verifier l'absence de nouvelle dependance, de second moteur d'arret, d'inference par nom et de suppression d'historique, puis mettre a jour les compteurs, preuves et statut dans `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/specs/073-actions-agent-ui/spec.md` et `/home/moi/bridget-referent/.worktrees/session-073-actions-agent-ui/specs/073-actions-agent-ui/tasks.md`

## Dependances

```text
T001
  |
  +--> T002 --> T003
  |
  +--> T004 --> T005
              |
              +--> US1 T006-T009
              |       |
              |       +--> US2 T010-T012
              |       |
              |       +--> US3 T013-T017
              |               |
              |               +--> US4 T018-T021
              |
              +--------------------> T022 --> T023 --> T024
```

- US1 depend seulement de la projection de base et reste testable sans arret reel.
- US2 depend du panneau de US1 mais pas du contrat d'arret.
- US3 depend du panneau et du fait `persistent`.
- US4 depend de la confirmation US3 et de l'adaptateur `StopOrder`.

## Opportunites paralleles

- T002 et T004 portent des bancs Rust distincts mais touchent le meme fichier : les executer sequentiellement pour eviter une collision d'edition.
- T010 peut etre preparee en parallele des travaux backend T004-T005.
- T017 porte uniquement CSS et peut avancer en parallele de T015-T016 apres stabilisation des classes DOM.
- T021 peut avancer apres figement du contrat T005, en parallele de T018-T020.

## Strategie d'implementation

1. Prouver la base avant modification.
2. Figer d'abord projection et contrat backend.
3. Livrer chaque user story comme increment testable, sans s'arreter au seul panneau US1.
4. N'ajouter aucune abstraction generique tant qu'elle n'a pas trois usages reels ou une responsabilite de securite prouvee.
5. Terminer seulement quand les 24 taches sont cochees et que toutes les preuves sont enregistrees.

## Validation Article XX

- T003 evite un second registre de gestion.
- T005 garantit une seule implementation du cycle de vie.
- T008 et T011 font evoluer la fiche globale existante au lieu d'en creer une seconde.
- T022 et T024 prouvent l'absence de dette cachee, de dependance ou de mecanisme parallele.
- Complexite ajoutee justifiee : une route locale et une confirmation modale sont necessaires pour exposer une operation destructive existante avec un verdict fiable et une accessibilite correcte.
