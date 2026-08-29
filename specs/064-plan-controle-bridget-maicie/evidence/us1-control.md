# Preuve US1 - contrôle d'exécution

Date de validation : 2026-08-29.

## Périmètre et chronologie

La preuve a été exécutée dans des répertoires temporaires issus du worktree isolé. La bascule `execution_projection.dual_write` est activée seulement par les fixtures qui en ont besoin. Aucun daemon de production, relais UI de production ou agent existant n'a été redémarré.

1. `QueueOnly` persiste l'enveloppe exacte et répond `Ack` sans remettre de prompt.
2. `TriggerTurn` persiste l'enveloppe, crée `execution-<message_id>` en `starting`, puis remet `DeliverExecution` avec génération et révision.
3. `SteerCurrent` conserve son intention explicite. Il reste un contrôle de tour et n'est jamais converti en travail FIFO. Une capacité absente produit un refus structuré.
4. `InterruptAndStart` conserve sa propre intention, crée une exécution durable et laisse le transport annuler seulement le tour actif capturé avant de reprendre sa FIFO.
5. `ControlOnly` est refusé comme message métier : la commande `ControlExecution` versionnée est la seule voie de contrôle corrélée.
6. Un silence fournisseur atteint une issue `failed`; une saturation atteint `failed` avec la raison machine `provider_queue_full`.
7. Un terminal d'un message absent des bindings n'émet ni état de tour, ni réponse. Une transition envoyée par un wrapper non propriétaire est refusée sans modifier l'exécution.
8. Quatre demandes d'autorisation distinctes pour le même tour donnent au plus trois attentes, puis `failed` avec `approval_loop_detected`.

## Oracles exécutés

Les commandes suivantes ont réussi, pour 22 tests ciblés :

```text
/home/moi/.cargo/bin/cargo test -p bridget-daemon --test work_submission_test --quiet
/home/moi/.cargo/bin/cargo test -p bridget-daemon --test execution_store_test --quiet
/home/moi/.cargo/bin/cargo test -p bridget-daemon --test execution_lifecycle_test --quiet
/home/moi/.cargo/bin/cargo test -p bridget-daemon --test managed_wrapper_test --quiet
/home/moi/.cargo/bin/cargo test -p bridget-daemon --test ui_relay_test relais_ui_expose_separement_connexion_vitalite_tour_attente_et_file -- --exact
/home/moi/.cargo/bin/cargo test -p bridget-daemon --lib daemon::intentions_de_soumission_restent_distinctes_et_persistantes -- --exact
/home/moi/.cargo/bin/cargo test -p bridget-daemon --lib daemon::transition_execution_refusee_si_wrapper_non_proprietaire -- --exact
/home/moi/.cargo/bin/cargo test -p bridget-daemon --lib wrapper::reconnect_tests::evenement_terminal_tardif_n_emet_ni_etat_ni_reponse -- --exact
/home/moi/.cargo/bin/cargo test -p bridget-daemon --lib wrapper::reconnect_tests::boucles_d_autorisation_sont_bornees_par_une_issue_machine -- --exact
```

La durée totale mesurée de cette séquence est de 5,02 secondes. La suite transport commune passe aussi : 214 succès, 1 ignoré.

## Couverture adversariale

`managed_wrapper_test` couvre le silence et la saturation sur un vrai processus de wrapper avec adaptateur de fixture. Les mauvais identifiants et terminaux tardifs sont vérifiés à la frontière du wrapper, puis la nouvelle garde daemon vérifie qu'un wrapper d'un autre agent ne peut pas modifier une exécution qu'il ne possède pas. Les trois niveaux sont nécessaires : fournisseur, corrélation locale et autorité durable.

## Limites nommées

- Aucun fournisseur réel ni route de production n'a été sollicitée. La preuve porte sur les contrats et adaptateurs de fixture.
- Le compteur de boucles d'autorisation vit dans le binding de tour actif. Après la fin ou le redémarrage du wrapper, une nouvelle observation doit être recorrélée avant toute transition.
- La suite complète `bridget-daemon` reste non verte à cause de deux tests antérieurs hors US1 : inscription MCP auxiliaire et reprise après arrêt brutal. Les rejouements isolés échouent également.
- La suite UI complète comporte quatre tests antérieurs d'inscription humaine qui échouent avant de publier leur URL. Le test de projection US1 passe isolément.
