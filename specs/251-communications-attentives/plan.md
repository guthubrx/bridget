# Plan251
## Architecture et périmètre
Rust/serde/rusqlite existants, protocole ClientHello, MessageIntent, ledger et reçu idempotent. Aucun service, table, dépendance, endpoint ou mécanisme d'inférence nouveau. Fichiers changés : message.rs, protocol.rs, communication.rs, execution_store.rs (exhaustivité), idempotency/send_delivery.rs, daemon.rs, mcp.rs, cli.rs ; documentation/consignes existantes.
## Ordre
1. Contrat StoreOnly et capacité SendInformationV1 ; canon distinct, défaut queue inchangé.
2. Transaction IdempotencyStore::store_information : de Prepared à terminal/accepted, ledger dans le même COMMIT. Ne créer ni livraison, ni suivi, ni soumission. Relecture exacte et reprise par le reçu existant. Refuser une collision de clé ledger étrangère plutôt que l'écraser.
3. Après réservation et avant capture, refuser toute combinaison réponse/corrélation/autorité. prepare_dispatch conserve droits/projet/routage. Après préparation, StoreOnly va au dépôt et retourne ; jamais au dispatcher. Legacy Send StoreOnly refusé : parcours idempotent obligatoire.
4. CLI --intent information et MCP intent=information : négociation avant SendIdempotent ; reçu stocké seulement après Accepted, inconnu reste inconnu. CLI force idempotence pour ce mode. Lire par ledger existant.
5. Politique écrite de maintenance, exemples fils notify:[] et actions ciblées. Corriger la garde dormance des observations confirmée, avec test de fait conservé sans écriture au fournisseur.
6. Tests ciblés Store/daemon/façades, contrôles positifs demande/résultat/guidage, retour compatible. Aucun appel modèle pour un test de silence. Contre-revue plan puis code, convergence réelle.
7. Livraison0.8.17 (si exécutable) sur quatre hôtes par maintenance native existante, image retour qualifiée, release/APK selon parcours existant. T3 ouvert, anciennes façades signalées ; une façade nouvelle installée permet le mode, pas d'activation globale inventée.
## Complexité / constitution
Le dépôt est O(taille corps + log N), sans scan de flotte ni chargement d'historique. Une opération nouvelle pour une règle produit nouvelle, dans le store existant. Pas de framework d'attention ni seuil d'inactivité. Tous les autres intents gardent leurs contrats. Le retour ne rejoue aucun dépôt terminal ; vérifier avec les lecteurs réels. Effet d'interface limité au JSON/CLI, pas de nouvelle page.
## Revue / vérification
La revue est statique sur paquet et diff, lecture seule, fournisseur distinct si joignable. Les tests comparent les effets réellement créés : ledger, remises, suivi et contrôle fournisseur. Une sortie vide n'est pas preuve de silence.

Dispositions revue : capacité serveur obligatoire au puits, StoreOnly jamais native_task_exchange, refus structurel dans begin_send_delivery/defer_idempotent_delivery ; voir adversarial-review-plan.md.
