# Plan technique - SPEC-081

## Décision

Le projet possède déjà agent_identities.agent_id et agent_profiles.display_name dans crates/bridget-daemon/src/agent_profile.rs. Cette base est réutilisée. La migration rend cet agent_id autoritaire partout où name était une clé de routage. Aucun second registre n'est créé.

agent_id est un UUID v4 minuscule. Il est stable pendant la vie d'un agent, transmis au wrapper lors du lancement, et une recréation produit une nouvelle identité. Les valeurs de routage historiques ne servent qu'à la migration puis sont supprimées.

## Flux cible

1. Un spawn crée ou reçoit agent_id et un profil.
2. Le wrapper s'enregistre avec agent_id.
3. Le routeur indexe les connexions par agent_id.
4. Messages, présences, flotte et exécution conservent le même agent_id.
5. Le daemon joint display_name pour les rendus et les prompts.
6. Maicie sélectionne agent_id. Un ID absent passe en requires_retarget.

## Migration

La commande bridget identity migrate propose :

- --dry-run : lecture seule, compteurs des lignes convertibles, introuvables et ambiguës.
- --apply : sauvegarde les SQLite, exécute la transaction Bridget puis Maicie, écrit une version de migration et relance les agents qui n'étaient pas stopped.

Ordre :

1. Construire la table historique -> agent_id depuis agent_identities et le ledger.
2. Convertir ledger, requêtes suivies, exécution, flotte, présence et cycle de vie Bridget.
3. Convertir configuration et données Maicie avec la même table.
4. Marquer toute référence sans identité requires_retarget, sans livraison automatique.
5. Retirer current_routing_name et agent_routing_aliases, puis écrire le marqueur de version.

Le preflight échoue avant toute écriture si une valeur historique mène à plusieurs agent_id ou si une base n'est pas sauvegardable.

## Composants

| Composant | Réemploi | Évolution |
|---|---|---|
| bridget-core Router | routeur existant | index et validation agent_id, plus d'auto-nom |
| BridgetMessage | type existant | from/to ont la sémantique principal agent_id |
| protocol AgentInfo | contrat existant | agent_id et projection display_name, sans name |
| AgentProfileStore | tables existantes | agent_id seule clé vivante, migration one-shot |
| daemon, flotte, exécution | stores existants | clés agent_id pour relance et reprise |
| wrapper | handshake et prompts | agent_id au transport, display_name à l'injection |
| UI et Desktop | projection profil | id technique pour routes, display_name pour rendu |
| Maicie | profils, store, outbox | agent_id, requires_retarget |

## Compatibilité et sécurité

- Pas de double lecture durable ni alias de compatibilité.
- Les wrappers historiques sont refusés avec erreur de version.
- agent_id ne devient pas une preuve d'autorisation.
- Les sauvegardes ne sont jamais exposées par l'UI.

## Vérification

- Tests unitaires routeur et messages.
- Tests SQLite de migration Bridget, rollback et idempotence.
- Tests Maicie : délégation convertie, agent supprimé retargeté, outbox bloquée.
- Intégration : renommage sans rupture de liens, recréation sans réattribution.
- Vérification manuelle : dry-run puis rendu UI sans fuite de nom historique.
