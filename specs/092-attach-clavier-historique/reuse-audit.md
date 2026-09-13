# Audit de reutilisation de l'existant — 092

## Decision

Statut: PASS
Date: 2026-09-06
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/091-communication-agent-ux/specs/092-attach-clavier-historique

Conclusion courte: le plan étend les quatre points locaux déjà présents. Aucun
canal, renderer, stockage ou contrôle modèle n'est recréé. Exploration préparatoire
déléguée au même équipier en lecture seule (réponse f0ca36c008684), complétée par
vérification directe. Son constat durée de vie et écho redirigé est intégré au plan.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 7 |
| Items audites | 7 |
| Reutilisations deja prevues | 5 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 4 |
| Specs existantes applicables | 1 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve à ea52cd04 | Commentaire |
|---|---|---|---|
| Tampon de saisie | InputBuffer | crates/bridget-daemon/src/attach.rs:709 | REUSE, remplacer alt_prefix |
| Entrée/envoi | handle_input_byte | crates/bridget-daemon/src/attach.rs:1945 | REUSE, un seul Send |
| Garde terminal | RawTerminal/with_raw_terminal | crates/bridget-daemon/src/attach.rs:50 | REUSE, durée de vie clavier |
| Affichage | BlockRenderer/RendererSender | crates/bridget-daemon/src/attach.rs:1132 | REUSE, input_changed |
| Couture de test | PseudoTerminal + UnixStream::pair | crates/bridget-daemon/src/attach.rs:3315 | REUSE, aucune lib ajoutée |
| Décodeur enrichi | absent ; bool alt_prefix seulement | crates/bridget-daemon/src/attach.rs:714 | CREATE état privé borné, remplace le bool |
| Historique de saisie | absent ; VecDeque déjà importé | crates/bridget-daemon/src/attach.rs:10 | CREATE état privé, pas historique journal |

## Existant potentiellement pertinent non mentionne

Aucun. Le contrôle `/model` à attach.rs:2049 est explicitement conservé ; il ne
constitue pas une tâche de développement nouvelle.

## Duplications evidentes

Aucune duplication non arbitrée ; aucune nouvelle dépendance.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| AGENTS.md | communication/observation ; isolation | pas de daemon de production pour les tests |
| .specify/memory/constitution.md | référence utilisateur | constitution globale lue |
| /Users/moi/.speckit/constitution.md | XIX/XX | petits états avec invariant, pas framework |
| my-specify-all fourni + demande humaine | délégation et pas de commit automatique | pilote docs, équipier code |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 091-communication-agent-ux | rendu TTY, Alt+Entrée, SelectRuntime | conserver les points d'entrée et tests |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| rg InputBuffer/RawTerminal/handle_input_byte/PseudoTerminal | attach.rs | points ci-dessus |
| rg InputHistory/history_prev/history_next/readline/rustyline | crates/ | aucun historique d'entrée, seulement readline de fixtures |
| rg historique/Alt/model | spec/plan/tasks 091 et README FR/EN | journal distinct ; contrôle modèle déjà livré |
| rg crossterm/rustyline/reedline/termion | manifestes workspace | aucune dépendance correspondante |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Éditeur complet | réutiliser/étendre | seulement touches demandées et contrôles induits | 2026-09-06 |
| Historique disque | ne pas créer | historique local demandé, confidentialité | 2026-09-06 |
| Répertoire 091 | réutiliser sous branche 092 | cwd autorisé du processus humain existant | 2026-09-06 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
