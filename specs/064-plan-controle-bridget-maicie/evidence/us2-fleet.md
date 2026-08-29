# Preuve US2 - Flotte et propriété

Date : 2026-08-29

## Parcours indépendant

1. Une réservation avec parent, mandat, rôle et identifiant d'exécution crée un lien `reserved` avant toute création de processus.
2. Les limites d'enfants, de profondeur et de cycle sont refusées avant la génération. L'échec de préparation clôt le lien sans le réattribuer.
3. La disparition du parent conserve le lien comme `orphaned`. Un transfert explicite remplace le parent et produit `transferred` sans perdre le résultat tardif.
4. Chaque changement de lien est ajouté au journal SQLite cursé. Une attente relit d'abord le delta, se termine vide à sa borne, puis se réveille sur un fait durable. Après réouverture du superviseur, le lien et son événement restent lisibles.
5. Le wrapper demande `WaitAgentLinks` sans choisir de parent : le daemon le dérive de `conn_instances`. Une connexion inconnue reçoit un Nack, sans accès à un autre parent.

## Commandes exécutées

- `/home/moi/.cargo/bin/cargo test -p bridget-daemon --test agent_graph_test` : 4 passés, 0 échec.
- `/home/moi/.cargo/bin/cargo test -p bridget-daemon attente_de_descendance_derive_le_parent_de_la_connexion_et_reprend_un_curseur --lib` : 1 passé, 0 échec.
- `/home/moi/.cargo/bin/cargo test -p bridget-daemon lien_agent_optionnel_survit_au_roundtrip_fleet --lib` : 1 passé, 0 échec.
- `/home/moi/.cargo/bin/cargo test -p bridget-daemon projection_ui_distingue_connexion_vitalite_tour_attente_file_et_propriete --lib` : 1 passé, 0 échec.
- `/home/moi/.cargo/bin/cargo test -p bridget-transport lifecycle_messages_roundtrip_and_stay_outside_attach` : 1 passé, 0 échec.
- `/home/moi/.cargo/bin/cargo test -p bridget-daemon --no-run` : compilation de toutes les cibles de test réussie.
- `git diff --check` : aucun espace blanc invalide.

## Non vérifié

Aucun daemon ni agent existant n'a été redémarré. Le parcours est isolé sur SQLite temporaire et ne prouve donc pas encore l'adoption par les wrappers déjà déployés.
