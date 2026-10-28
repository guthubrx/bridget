# Session201 — Annulation avant création du reçu

Correction autorisée : une annulation déjà demandée reste cancelling si Start a été refusé avant création du reçu worker. Incident6231b14c-acfe-4b85-80f4-f019a6ecfb81, clé opus167-bilan-integration-complet-20261028-01, commande6231b14c/instance471786a5-7b58-40fd-8759-d87890692cc6/génération1270, échéance1793212464. Aucun reçu commande, préparation conservée ; aucune mission émise. L’absence seule ne prouve pas un arrêt.

Réutiliser FenceUnstarted198 uniquement après UnknownExecution du Cancel, pour la commande durable exacte d’une Task cancelling sans mission émise. Worker compatible requis. La clôture repose sur StartFenced durable ; aucun Start, Prepare, Release ou relancement. Un pair ancien, réponse perdue, preuve divergente ou incarnation inconnue conserve l’annulation en attente. Fichiers et historique conservés.

La correction doit être générique. Le diagnostic précise la capacité ou preuve manquante sans inventer une cause de ResourceLimit. La maintenance doit préserver les tâches actives et inconnues ; aucune modification manuelle de DB, aucun redémarrage forcé.

Défaut de maintenance confirmé par revue : prepare_restart des demandes enfant traitait les tâches distantes avant leur exclusion de la reprise locale. Le maître n’a pas arrêté leurs groupes ; il ne peut donc pas invalider ou régler ces callbacks depuis cette seule reprise. Exclure les tâches distantes de ce coldrestart, conserver leurs demandes ouvertes/en attente/envoyées jusqu’à leur propre contrôle natif, et garder le comportement local existant.
