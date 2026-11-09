# Qualification
1. Deux identités de test connues, récepteur au repos puis busy/DND/dormant : dix informations ; dix corps en ledger, aucun Deliver/contrôle/file/suivi.
2. Même clé : reçu identique, un corps ; même clé avec autre intention/corps : envelope_mismatch. Erreur SQL provoquée : aucun reçu terminal ni livraison ; réouverture puis replay sûr.
3. Même fixture, demande reply=true classique : une remise et un suivi. Information+reply ou in_reply_to : refus précis, résultat de mission préservé.
4. CLI/MCP fausse ancienne négociation : aucun SendIdempotent ; façade nouvelle renvoie stored seulement sur Accepted.
5. Fil bulletin notify:[] : aucun wake ; observations dormant : fait durable, pas de message fournisseur. Ne pas tester avec les agents métier ni recopier les données SNCF.
6. Après activation, propre destinataire de maintenance seulement, dépôt information et lecture exacte ; comparer les compteurs durable/dispatch, pas lancer une mission modèle. Ne pas redémarrer T3.
