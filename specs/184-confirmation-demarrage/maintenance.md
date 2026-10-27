# Maintenance préservant les travaux — état de conception au 27 octobre 2026

Statut : NON QUALIFIÉE, NON AUTORISÉE pour l’amorçage exceptionnel. Aucun correctif de maintenance chargé. Ce document ne constitue pas une procédure exécutable.

## Faits métier reçus
Politique sol, message mcp-60861-6ae0bdf7-12 : réattestation native unique, puis même tâche KS a5b2b14b passée de mission_pending à working, mission_deadline_at1793116941, résultat absent. Aucune causalité attribuée à la seule réattestation.
Message mcp-60861-6ae0c3a8-13 : result_available, erreur nulle, résultat lu/ACK, commitc5a1d88fd0929957417f72815f52d86cb23354df publié sur branche166-sol-code-local-ksstore-20261027-02, tests rendus, worktree propre selon le propriétaire. La remise métier de cette tâche est donc confirmée par le propriétaire. Aucun remplacement. Ce fait ne prouve pas l’activation184.
Demande propriétaire mcp-86062-6ae0c48a-3 : clôturer la garde uniquement si le travail est fini, conserver les commits et fichiers ; sinon conserver la garde. Demande expirée sans réponse pendant cette investigation. Aucune clôture présumée ni annulation par le mainteneur.

## Première piste rejetée : assouplir Prepare pour le seul maître
Revue indépendante56abca6d-bb94-4990-9355-31ae39eae39f, demande bridget-maintenance-scope-review-20261027-01, base4d89cfab52f031bf19865c10ef8cff72411bfa49. Résultat lu/ACK, garde clôturée nativement, continuable=false, error=null.
- Gate::enter bloque les trames non exemptées : heartbeat, réponses worker et supervision ne sont pas séparés des nouvelles admissions.
- Un wrapper natif peut s’arrêter sur EOF ou à l’expiration de sa reconnexion ; autoriser le restart du seul maître ne prouve pas la conservation du fournisseur.
- Le démarrage appelle native_child_requests::prepare_restart avant l’exclusion des tâches distantes. Une reprise ordinaire n’est pas un transfert sans effets.
- Retirer seulement le test pending(), ou changer le nom du bail en core-only, n’est pas une correction sûre. Aucun code de ce type écrit.

## Contrat nécessaire pour un futur transfert du daemon
Cette conception n’est pas implémentée : bail durable lié à l’identité/génération du maître et à une clé ; suspension des nouvelles mutations utilisateur ; continuité explicite et testée des heartbeats, événements, contrôle worker, supervision et reconnexion ; reprise sans réémission de mission, remplacement de fournisseur ou clôture des demandes enfants. Les anciens pairs doivent refuser sans effet si leur capacité manque. Aucun résultat Terminal ne peut être déduit de l’absence d’un reçu.
Tests requis : sauvegarde/relecture cohérente ; même tâche/instance/génération/fil ; interruptions à chaque phase ; concurrence annulation et admission ; ACK perdu ; expiration ; refus de double maître ; aucune perte de demande ou livraison ; aucun arrêt de fournisseur ; rollback de binaire distingué d’une restauration de données.
Ce parcours futur ne résout pas à lui seul l’amorçage du processus183 déjà vivant.

## Deuxième piste examinée : intervention exceptionnelle limitée au maître
Revue indépendante1c8d388d-079a-4ec5-a77a-9aeeb61f0c0f, demande bridget-maintenance-bootstrap-review-20261027-01, même base. Verdict défavorable en l’état, lu/ACK et garde clôturée nativement, continuable=false/error=null.
La proposition n’a pas été exécutée. Elle exigerait une décision humaine distincte du mandat qui interdit le forçage, après qualification des points suivants :
1. Fermeture native par les propriétaires de toutes les autres gardes finies ; aucune autre tâche, commande, demande, admission ou reprise incertaine. Une présence unreachable ne prouve pas l’arrêt ; notamment fe3c88b9 sur SYS n’a pas été réattestée ici. e5959986 resterait explicitement en état inconnu/cancelling, sans écriture directe de son store.
2. Gel natif des nouvelles délégations par un humain réel, sans cancel-active, avec reçu halted=true. Ce gel ne suspend ni les tâches ni les événements déjà présents.
3. Aucun groupe local vivant ou reprise désirée susceptible de lancer un fournisseur ; aucune demande enfant ouverte, livraison pending/sent, clôture ou inbox non réglée. Contrôler le chemin complet de prepare_restart, pas seulement le compteur des tâches.
4. Exclusion effective de l’auto-updater et de toute activation/récupération concurrente. Le statut réel observé indique automatic=true, install_target0.6.3 ; son service est actif. Aucune préférence ni aucun service modifié pendant l’examen.
5. Sauvegarde cohérente de SQLite, flotte et configuration ; traitement documenté des événements entre sauvegarde et arrêt. Revenir au binaire183 ne restaure pas ces données ; restaurer les données serait une autre opération avec risque de perte d’événements.
6. Arrêt propre borné sans escalade SIGKILL ; KillMode=process et TimeoutStop30s ne suffisent pas à le garantir. Aucun restart ne doit être engagé avant qualification de ce point.
7. Vérification du build chargé, maintien des PID tiers, absence de rejeu métier, retour au seul binaire183 en cas d’échec compatible. Les unités workerCartae et mobile n’ont pas PartOf/BindsTo vers core ; cela ne constitue pas une preuve complète de continuité.

Empreintes relues des exécutables présents :
- /home/moi/.local/lib/bridget-communication/183-admission-7478e9e7/bin/bridget : d6ba590c987b67a206d6ea87e5d9e832d3b3f47abc54a19c30df93deb93770ef.
- /home/moi/.local/lib/bridget-communication/184-guidance-7899d0bdab01/bin/bridget : 621101901c8fc6c3df278cc82f94133a02132d501bd1b4a57f55a07785753483.
Ces empreintes établissent les fichiers présents, pas leur activation. Le diff183→184 examiné ne montre pas de migration de stockage ; la procédure entière n’est pas qualifiée pour autant.

## Décision actuelle
Conserver les tâches, fichiers, candidats, exécutables chargés et retours. Aucun nouveau Prepare identique, aucune suppression de garde, aucun arrêt de tâche d’autrui, aucun store modifié à la main et aucun service Opus touché. Les deux pistes ont produit des objections concrètes ; ne pas les présenter comme disponibles ou approuvées. Le résultat métier KS est acquis selon son propriétaire ; la correction de maintenance et le chargement184 restent non réalisés.
