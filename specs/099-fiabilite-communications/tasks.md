# Tâches 099 — Fiabilité et identité des communications

Statut : Implemented — 16/16 tâches vérifiées. Les chemins ci-dessous sont relatifs au dépôt
/Users/moi/Nextcloud/10.Scripts/64.bridget.
Tests avant correction. Pas de livraison partielle déclarée terminée.

## Fondations
- [x] T001 Consigner les reproductions initiales et les scénarios d'acceptation dans specs/099-fiabilite-communications/implementation.md et tests/features/099-fiabilite-communications.feature ; vérifier les refus et limites explicites.

## US1 — Isolation d'un destinataire lent
- [x] T002 [US1] Ajouter les tests spec099 de socket saturée, verrou de writer occupé, échange témoin et réponse immédiate dans crates/bridget-daemon/src/daemon.rs ; constater l'échec avant correction.
- [x] T003 [US1] Réutiliser l'écriture bornée hors verrou et installer le suivi avant remise dans crates/bridget-daemon/src/daemon.rs ; Nack explicite sur échec et tests T002 verts.

## US2 — Autorisation de l'identité
- [x] T004 [US2] Couvrir faux MCP, CLI from_declared=false, Client SendIdempotent, preuve invalide/valide et révocation d'un auxiliaire déjà admis dans crates/bridget-daemon/src/daemon.rs et tests de sécurité existants ; témoins positifs préservés.
- [x] T005 [US2] Étendre Registered/RegisterAuxiliary et l'identité privée dans crates/bridget-transport/src/protocol.rs, crates/bridget-daemon/src/mcp_identity.rs et wrapper.rs ; sérialisation compatible, secret masqué, fichier privé borné et sans symlink.
- [x] T006 [US2] Raccorder les clients et verrouiller toutes les admissions dans crates/bridget-daemon/src/communication/client.rs, mcp.rs, cli.rs et daemon.rs ; adapter les fixtures aux preuves réelles, aucun bypass test et aucun emprunt d'UUID par un humain externe.

## US3 — Contrôle t3code pendant l'attente
- [x] T007 [P] [US3] Ajouter les régressions spec099 attente/annulation/expiration/daemon absent dans crates/bridget-daemon/src/t3code.rs et audits/2026-09-16/session-2026-09-16-bridget-global-01/reproduce_t3.py ; faux HTTP existant, aucun tour post-annulation.
- [x] T008 [US3] Rendre l'attente LinkWorker réactive dans crates/bridget-daemon/src/t3code.rs sans doubler IdempotentDeliveryTracker ; contrôler les bornes avant dispatch et préserver les demandes suivantes.
- [x] T009 [US3] Étendre la notification CancelDelivery au transport t3code dans crates/bridget-daemon/src/daemon.rs ; aucun tour textuel supplémentaire pour annuler.

## US4 — Réponses récupérables
- [x] T010 [US4] Ajouter les régressions spec099 destinataire absent, reprise/redémarrage, ACK perdu et plusieurs attentes dans crates/bridget-daemon/src/t3code.rs et audits/2026-09-16/session-2026-09-16-bridget-global-01/reproduce_t3.py ; une absence dans ListRequests bornée ne signifie jamais clôture.
- [x] T011 [US4] Conserver réponse et attentes jusqu'à confirmation dans crates/bridget-daemon/src/t3code.rs ; réutiliser la persistance atomique, aucune sauvegarde d'un préfixe de pending et aucune réexécution fournisseur.

## US5 — Observation honnête
- [x] T012 [US5] Tester texte Unicode long et refus de journalisation dans crates/bridget-daemon/src/t3code.rs ; tenir compte du failure sink de JournalWriter et de la borne de ligne 4 Mio.
- [x] T013 [US5] Corriger coupe silencieuse et repère après échec dans crates/bridget-daemon/src/t3code.rs ; contenu complet dans les bornes, lacune explicite sinon.

## Vérifications transverses
- [x] T014 Documenter les garanties, refus anciens auxiliaires et limites de confiance dans README.md, README.en.md et specs/099-fiabilite-communications/implementation.md ; tenir à jour quickstart.md avec commandes exactes.
- [x] T015 Exécuter les régressions 099, les tests existants concernés, le workspace, fmt et clippy selon quickstart.md ; consigner pass/fail et toute limite réelle dans implementation.md, pas de succès supposé.
- [x] T016 Refaire la contre-revue adverse si accessible, vérifier la convergence FR/SC/code/tests et produire l'audit final du diff ; consigner le verdict dans implementation.md, sans commit.

## Dépendances et parallélisme

T001 → T002 → T003 → T004–T006 pour le daemon/protocole/clients.
T001 → T007–T008 → T010–T013 pour le pont ; T009 après libération de daemon.rs.
Deux propriétaires de fichiers distincts peuvent travailler en parallèle ; aucun
travail simultané dans daemon.rs. T014–T016 après intégration.
Chaque famille est vérifiée indépendamment ; le périmètre livré reste US1–US5,
pas un MVP US1 seulement.

## Couverture exigences

FR01–03 : T002–T003 ; FR04–06 : T004–T006/T014 ;
FR07 : T007–T009 ; FR08–09 : T010–T011 ; FR10 : T012–T013 ;
FR11 : gate de réutilisation et T005/T011/T013 ; FR12 : T001/T015.
SC01 : T002–T003 ; SC02 : T004–T006 ; SC03 : T007–T009 ;
SC04 : T010–T011 ; SC05 : T012–T013 ; SC06 : T015–T016.

## Convergence

Reprise du 2026-09-16 par l'agent bdget (Claude), après recette complète
`cargo test --workspace --release --no-fail-fast` : 1333 réussites, 11 échecs
laissés par l'implémentation initiale, qui n'avait exécuté que des modules ciblés.

- [x] T017 [US2] Migrer `presence_tests::tour_non_abouti_redevient_mandatable_et_le_mandat_parvient` et `spec_079_ack_puis_deux_redemarrages_rejouent_une_seule_continuation` vers un expéditeur attesté (propriétaire vivant + rattachement auxiliaire) dans crates/bridget-daemon/src/daemon.rs.
- [x] T018 [US2] Rendre au chemin idempotent l'admission de l'étiquette humaine d'un client négocié sans identité (daemon.rs `handle_idempotent_send`, cli.rs `send_idempotent_to_daemon`), test `spec099_humain_idempotent_depuis_client_nu_reste_admis` ; harnais tests/claude_interactive_097_test.rs 8/8.
- [x] T019 [US2] Lire l'annuaire sans preuve côté MCP (`execute_who` sur connexion nue, mcp.rs), test `who_lit_l_annuaire_public_sans_enregistrement` ; core_089_isolation_test 5/5 ; fixture core_089_concurrency_test alignée.
- [x] T020 [US2] Aligner le faux daemon de core_089_security_test sur l'ordre réel des trames (attestation avant négociation) : 8/8.
- [x] T021 [US1] Observer l'état busy par attente dans managed_parity_test (remise asynchrone) et inscrire un diagnostic des tours métier en cas d'écart ; 3 passages verts à vide, un tour en trop vu une fois sous charge (piste ouverte).
- [x] T022 Documenter l'amendement « humain et annuaire » dans docs/decisions/035.

