# Tâches251 — In Progress
## Cadrage et contrat
- [x] T001 Relever la mesure privée, les sources et les décisions dans specs/251-communications-attentives/research.md ; aucun contenu SNCF versionné.
- [x] T002 Produire spec/plan/contrats et reuse-audit.md PASS ; absence des primitives constatée, protocole manuel.
- [x] T003 Demander la contre-revue plan autre fournisseur, conserver son reçu dans specs/251-communications-attentives/adversarial-review-plan.md ; traiter ses objections avant activation.
## US1 — Stocker sans réveil
- [x] T004 [US1] Étendre MessageIntent et ClientCapability dans crates/bridget-core/src/message.rs et crates/bridget-transport/src/protocol.rs ; canon251 distinct dans communication.rs, formes historiques inchangées.
- [x] T005 [US1] Ajouter le dépôt atomique au store existant crates/bridget-daemon/src/idempotency/send_delivery.rs ; ledger+reçu terminal, aucun suivi/remise et collision protégée.
- [x] T006 [US1] Raccorder le dépôt dans crates/bridget-daemon/src/daemon.rs avant tout dispatcher ; accepter dormant/DND sans le réveiller, garder identité/projet/routage, refuser legacy non négocié.
## US2 — Conserver actions et résultats
- [x] T007 [US2] Dans daemon.rs refuser reply/corrélation/autorité pour information avant capture ; tester demande classique témoin et non-capture des résultats.
- [x] T008 [US2] Tester dans daemon.rs et idempotency/send_delivery.rs dix dépôts, rejeux/mismatch, réouverture et panne SQL : zéro contrôle fournisseur et aucune ligne de remise/exécution.
## US3 — Prévenir les bons agents
- [x] T009 [US3] Vérifier puis corriger les observations des dormants dans daemon.rs ; fait durable conservé et aucun Deliver, one-shot actif préservé.
- [x] T010 [US3] Mettre la matrice information/action/urgence et les bulletins regroupés dans docs/reference-communication.fr.md et AGENTS.md ; pas d'avis général aux seules présences connectées.
## US4 — Utiliser et vérifier
- [x] T011 [US4] Ajouter intent=information à crates/bridget-daemon/src/mcp.rs et cli.rs, négociation avant envoi, statut stored seulement sur acceptation ; tests de capacité absente et reçu.
- [x] T012 [US4] Vérifier les lectures du ledger existant et la compatibilité avec le lecteur de retour ; docs/decisions/251-communications-attentives.md et preuves sans appel modèle.
## Convergence, revue et livraison
- [ ] T013 Exécuter les contrôles ciblés via scripts/build.py ; preuves et commandes dans specs/251-communications-attentives/validation.md.
- [ ] T014 Contre-revue code autre fournisseur si joignable, Converge et audit après dernier changement ; aucun P1/P2 non traité dans specs/251-communications-attentives/validation.md.
- [ ] T015 Publier/activer les sept composants et retours qualifiés sur quatre hôtes par procédure existante, T3 reste ouvert ; contrôler images/processus/lien et vrai dépôt silencieux.
- [ ] T016 Mettre à jour bilan, intégrer sur main, pousser/vérifier alignement quatre hôtes, nettoyer ressources propres inutiles ; preuves dans specs/251-communications-attentives/validation.md.
## Dépendances et exécution
T001→T003→T004→T005→T006→T007→T008→T011→T009→T010→T012→T013→T014→T015→T016. Même responsable pour le code ; revue distincte seulement, aucun partage des mêmes fichiers. Pas de tranche MVP : les quatre stories et livraison sont requises.

## Convergence
- [x] T017 [US1] Dans crates/bridget-daemon/src/daemon.rs, ne pas appliquer les gardes de remise à StoreOnly ; tester disjoncteur saturé, contenu déjà remis, quarantaine et zéro saut, sans assouplir le mode normal.
- [x] T018 [US2] Dans crates/bridget-daemon/src/idempotency/send_delivery.rs et daemon.rs, distinguer la collision ledger constatée d'une panne SQL ; refus durable information_id_conflict, corps préexistant conservé, guidage précis et rejeu stable.
