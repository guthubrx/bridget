# Projet momentanément indisponible

Une session T3 valide ne suffit pas si le projet du fil n’a pas été attesté par Bridget. Une lecture peut alors rendre permission_attestation_unavailable, vérification origin_session_reattest, tandis que l’annuaire montre une présence active avec projet inconnu.

Après un résultat de projet vide, le connecteur corrigé réannonce les mêmes chemins et le rattachement au prochain cycle suivant un délai de 30 secondes. Les contrôles restent identiques. Un résultat positif arrête ces reprises. Une modification des chemins est annoncée immédiatement.

Relire le projet dans l’annuaire, puis faire une lecture native depuis le même fil. Conserver les références de toute demande d’issue inconnue ; une nouvelle clé peut dupliquer le travail. Si le projet reste inconnu après une reprise, faire vérifier l’accès du worker source au dossier annoncé et son éventuel worktree. Ne changer ni droits ni chemins pour masquer le refus. Les autres travaux autorisés suivent leur propre état.

Ce mécanisme est actif seulement après chargement du connecteur qui porte le correctif 291. Il ne nécessite pas une nouvelle session fournisseur.
