# Validation 242

Statut : In Progress. Défaut de texte confirmé : managed_resume_context utilisait « Worktree : indisponible » pour tout échec de managed_resume_worktree, qui ne mesure que Git. La tâche Revue142 rapporte une vue commit conforme et un résultat disponible. Son état et son contenu ne sont pas modifiés.

Correction : afficher le dossier courant local, conserver la cause Git échappée et distinguer ce contrôle de la disponibilité des fichiers. Aucune attestation supplémentaire fondée sur un fichier de notice, aucun changement de droits ni nouvelle garde. La poursuite concerne uniquement l’espace attesté et des fichiers accessibles.

Tests Mac : python3 scripts/build.py cargo test -p bridget-daemon --lib carte_ -- --test-threads=1 : 12 PASS, incluant le document lu sans Git, chemin absent, identité/Git, échappement et bornage existants. Avertissements de compilation existants hors changement. Nettoyage automatique exécuté, différé car observation globale de processus incomplète ; aucune suppression forcée.

Le défaut de transport de Revue142 reste distinct, cause historique inconnue. Le résultat guichet_request_status ne remplace pas la lecture de la tâche native. Aucun rejeu métier ni redémarrage T3.
