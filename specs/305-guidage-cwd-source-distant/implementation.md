# Guidage du dossier source pour un appel distant

Signalement e4a298cb-e22b-4ec7-9a32-bdd3f11a5502. Demande
arbres-silhouettes-r2-revue-20261120-0902, refus cwd_unavailable à 08:59:57
Europe/Paris, aucun task_id. Le parent natif est la tâche 2fec6bc0 et son
placement durable est monordinateur. Son dossier exact existe sur le Mac
au commit 5433970d507bf3ac695a4aeb527a82af43cbafc6.

La demande directe sans machine_id emprunte l'admission du coordinateur.
Elle résout le chemin localement sur Cartae. Le diagnostic ne nommait pas
cette machine et laissait croire que le dossier du Mac était absent. La
correction conserve le code cwd_unavailable et enrichit seulement son
résultat pour une demande directe sans machine_id dont l'origine est
attestée sur une autre machine. Elle conserve les refus et l'autorité de
routage, et n'inclut pas le chemin dans le message.

Test incident305 avec un appelant local et un appelant distant. Les deux
demandes invalides gardent le code. Seule la seconde précise la cause et
les paramètres attendus. Aucun dossier ni tâche n'est créé.

Les délais de lecture JSONL et les refus origin_session_reattest forment
un incident de transport distinct. La reconnexion à 09:05:59 et la
remise à 09:09:01 sont des preuves de passage ponctuelles. Les échecs
d'attestation à 09:07:59 et 09:08:15 empêchent de qualifier la stabilité.
Aucune cause précise d'attestation ou de panne générale déduite.
Les recherches en lecture seule ne trouvent pas d'admission ni de tâche
pour integration-page-14-20261120-0905 au moment du relevé. L'enveloppe
d'origine reste la seule admissible au rejeu strict quand le propriétaire
confirme son lien et le sort de l'appel.

Revue native demandée en lecture seule, clé
bridget-305-cwd-source-review-20261120-r1, tâche
f8f718cb-77a7-4db7-948c-0cddeb06d924. Aucun changement de configuration.

Contrôles terminés : test incident305, 51 tests native150 et Clippy strict
réussis. Le relecteur n'a constaté aucun défaut par lecture seule.
Résultat consulté par le port natif. Le libellé final vise toute tâche
et a été harmonisé. Le refus générique initial cwd_unavailable
n'incluait aucun detail à conserver.

Contrôle d'activation sur Cartae refusé avec
update_missions_active_or_unconfirmed. Aucun bail acquis ni service
arrêté. Le Mac et T3 restent en fonctionnement. Cache Cargo sous le
budget, aucun worktree créé.
