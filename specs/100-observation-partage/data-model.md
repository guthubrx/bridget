# Modèle 100

Extrait : agent, entries[objet JSON original avec seq], first_seq/last_seq, through_seq, next_seq,
complete, notices. Entrée conservée comme JSON valide, sans synthèse.
Abonnement : id opaque, owner attesté, event (enum), agent?, file?, once,
expires_at, suppressed_total ; portée daemon_instance. Compteurs globaux :
notifications_lost, evicted_writes. Sortie : file64, borne1s, expiration5s.
Fait : event, agent, host, file?, other_agent?, seq? ; aucune identité fournie
par l'abonné n'est utilisée comme identité émettrice.
Écriture récente : clé(hôte, chemin normalisé), agent, repère monotone,
dernière paire alertée ; cache borné et purgé après 30 secondes.
Séquence source dédupliquée par connexion autorisée, nettoyée à sa fermeture.
JournalLiveFeed : file256 de métadonnées post-flush indépendante de la vue attach.

Transition abonnement : créé → actif → supprimé / expiré / ponctuel déclenché.
Pas de cycle de validation de mission. Perte au redémarrage documentée.
