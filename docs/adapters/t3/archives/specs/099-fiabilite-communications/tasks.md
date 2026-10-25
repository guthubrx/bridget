# Archives techniques de l’adaptateur T3

Document d’origine : [specs/099-fiabilite-communications/tasks.md](../../../../../../specs/099-fiabilite-communications/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## US3 — Contrôle t3code pendant l'attente
- [x] T007 [P] [US3] Ajouter les régressions spec099 attente/annulation/expiration/daemon absent dans crates/bridget-daemon/src/t3code.rs et audits/2026-09-16/session-2026-09-16-bridget-global-01/reproduce_t3.py ; faux HTTP existant, aucun tour post-annulation.
- [x] T008 [US3] Rendre l'attente LinkWorker réactive dans crates/bridget-daemon/src/t3code.rs sans doubler IdempotentDeliveryTracker ; contrôler les bornes avant dispatch et préserver les demandes suivantes.
- [x] T009 [US3] Étendre la notification CancelDelivery au transport t3code dans crates/bridget-daemon/src/daemon.rs ; aucun tour textuel supplémentaire pour annuler.

<a id="extrait-2"></a>

## Extrait 2

## US4 — Réponses récupérables
- [x] T010 [US4] Ajouter les régressions spec099 destinataire absent, reprise/redémarrage, ACK perdu et plusieurs attentes dans crates/bridget-daemon/src/t3code.rs et audits/2026-09-16/session-2026-09-16-bridget-global-01/reproduce_t3.py ; une absence dans ListRequests bornée ne signifie jamais clôture.
- [x] T011 [US4] Conserver réponse et attentes jusqu'à confirmation dans crates/bridget-daemon/src/t3code.rs ; réutiliser la persistance atomique, aucune sauvegarde d'un préfixe de pending et aucune réexécution fournisseur.

<a id="extrait-3"></a>

## Extrait 3

## US5 — Observation honnête
- [x] T012 [US5] Tester texte Unicode long et refus de journalisation dans crates/bridget-daemon/src/t3code.rs ; tenir compte du failure sink de JournalWriter et de la borne de ligne 4 Mio.
- [x] T013 [US5] Corriger coupe silencieuse et repère après échec dans crates/bridget-daemon/src/t3code.rs ; contenu complet dans les bornes, lacune explicite sinon.
