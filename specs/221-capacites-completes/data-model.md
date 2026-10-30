# Modèle de données221

Réutiliser d’abord les entités existantes.

| Entité | Réutilisation | État/validation à préserver |
| --- | --- | --- |
| Connexion/compte/modèle | registry et observations220 | nonce local, TTL, époque, disabled≠échec |
| Tâche/admission/tentative | delegation/execution_store/fleet | canonimmutable, owner, résultat≠arrêt |
| Projet/base/espace | workspace records/provisioning | base exact, ancreGit, rapport complet |
| Fil/tour/passation | threads/handoff/journal | provenance, permissions, pagination |
| Point de reprise/import | nouvelle extension de manifest si pas d’équivalent | source/export/version, preview, CAS, traces sans secrets |
| Secret request/reference | extension du contrat humain après audit158B | pending→submitted→consumed/expired/revoked ; valeur jamais enDB |
| Sessionoutil | navigateur/appareil/PTY liés au worker | owner/instance/epoch, contrôle humain, arrêt inconnu explicite |
| Réveil/abonnement | wake/store/reçus | nonce, canon, signature, échéance, cancel |
| Usage/prix | provider_limits/événements | source,date,unité,devise,mesure≠estimation |
| Preuve decapacité | validation221 | catégorie, versions, commande, sortie assainie/SHA, verdict, limites |

Les nouvelles tables/champs seront proposés uniquement après audit du store existant. Aucun schéma supplémentaire n’est créé à ce stade.
