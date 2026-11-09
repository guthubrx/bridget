# Modèle251
MessageIntent reçoit StoreOnly (wire store_only). ClientCapability reçoit SendInformationV1.
Pas de nouvelle entité persistante : ledger conserve id, ts, sender, target, body ; idempotency_records conserve canon et résultat terminal accepted. Aucun send_delivery/execution/request pour ce dépôt. Le reçu public de façade nomme stored seulement pour l'intention information et un Accepted constaté. Aucun lu/ACK fournisseur déduit.
Le journal reste soumis à sa rétention existante ; reçu expire selon horizon existant. Les fils restent la source collective pour bulletins et supersedes_seq. Ne pas promettre un archivage infini.
