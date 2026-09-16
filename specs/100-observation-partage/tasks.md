# Tâches 100 — Observation et partage

Statut : Livré — session100 fusionnée dans main et release c680c6ce36c5 installée le 2026-09-16 sur la 099 finale. 17/17 tâches closes, limites de validation dans implementation.md. Racine des chemins :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/100-observation-partage.

## Fondations

- [x] T001 Écrire les scénarios métier dans tests/features/100-observation-partage.feature et la trace des limites/revues dans specs/100-observation-partage/implementation.md ; vérification des trois US et des témoins négatifs.

## US1 — Extrait et partage (P1)

- [x] T002 [US1] Tester lecture fragmentée, Unicode, limites, lacunes et daemon absent dans crates/bridget-daemon/src/attach.rs ; fixtures socket isolées, aucune lecture arbitraire.
- [x] T003 [US1] Étendre crates/bridget-daemon/src/attach.rs et communication/client.rs pour produire l'extrait borné via assemblage existant ; tests T002 verts.
- [x] T004 [US1] Ajouter lecture/partage CLI et MCP dans crates/bridget-daemon/src/cli.rs et mcp.rs ; tester paramètres exclusifs, journal absent, partage réellement reçu et reply conservé.

## US2 — Abonnement (P1)

- [x] T005 [US2] Définir et tester contrat fermé, filtres, propriétaire, TTL et once dans crates/bridget-transport/src/protocol.rs et crates/bridget-daemon/src/observation.rs ; aucun owner arbitraire.
- [x] T006 [US2] Raccorder gestion d'abonnements, fins de tour/permissions et notifications bornées dans crates/bridget-daemon/src/daemon.rs ; vérifier aucun blocage source ni répétition sans transition.
- [x] T007 [US2] Exposer types/sub/list/unsub en CLI/MCP via client partagé dans crates/bridget-daemon/src/cli.rs, mcp.rs et communication/client.rs ; erreurs et reçus mêmes sémantiques.

## US3 — Écritures concurrentes (P2)

- [x] T008 [US3] Tester puis conserver les chemins structurés avant troncature dans les producteurs de crates/bridget-transport/src/ ; lectures et événements échoués exclus, couverture exacte documentée.
- [x] T009 [US3] Détecter les collisions dans crates/bridget-daemon/src/observation.rs et raccorder le flux source dans daemon.rs ; tests hôte/auteur/chemin/fenêtre/doublons/capacité, aucune écriture disque ni verrou métier.

## Vérifications transverses

- [x] T010 Tester parcours isolés CLI/MCP/daemon dans crates/bridget-daemon/tests/spec100_observation_test.rs ; permissions propriétaire et témoin de communication pendant notifications vérifiés.
- [x] T011 Mettre à jour README.md, README.en.md, specs/100-observation-partage/quickstart.md et docs/decisions/036-observation-non-bloquante.md ; garanties et limites observables, exemples utilisables, aucun succès métier déduit.
- [x] T012 Exécuter tests ciblés, régressions099, workspace, fmt et clippy ; consigner commandes/résultats dans specs/100-observation-partage/implementation.md et corriger dans le périmètre.
- [x] T013 Exécuter convergence et audit final ; actualiser specs/100-observation-partage/implementation.md, contre-revue et statut seulement après preuves et zéro tâche ouverte.

## Dépendances, périmètre, vérification

- [x] T014 Aligner sur la 099 finale, conserver ses correctifs et adapter les fixtures du protocole.
- [x] T015 Rejouer la validation de l'ensemble intégré et la revue de livraison ; consigner les limites.
- [x] T016 Committer et fusionner la session 100 dans main, sans toucher aux autres branches.
- [x] T017 Installer la release après sauvegarde, vérifier les services et les nouveaux outils ; documenter le retour arrière.

T001 → T002 → T003 → T004 ; puis T005 → T006 → T007 ; T008 → T009 ; T010–T013 après intégration.
US1 vérifiable seule mais livraison complète = US1+US2+US3. Aucun MVP ne clôt la demande.
Parallélisme possible sur recherche/producteurs hors fichiers daemon partagés,
pas d'édition concurrente du même fichier. Chaque création suit reuse-audit.md.

Couverture : FR01–03=T002–T004/T010 ; FR04–08=T005–T007/T010 ;
FR09–10=T008–T009 ; FR11=T003/T005–T006/T009–T010 ; FR12=T011 ; FR13=T001/T012–T013.
SC01=T002–T004 ; SC02–04=T005–T007/T010 ; SC05=T009–T010 ; SC06=T012–T013.

Bilan initial avant demande de livraison : 13/13 vérifiées. Converge CONVERGED en un passage. Audit diff100 local
sans finding résiduel confirmé, artefacts validés (0 erreur/0 warning).
T012 : commande globale explicitement bloquée et preuves consignées, pas un PASS.
Contre-revue externe non remise (autorité d'identité indisponible), non substituée
par un verdict inventé. Ce bilan ne clôt pas les tâches de livraison T014–T017 ajoutées ensuite.
