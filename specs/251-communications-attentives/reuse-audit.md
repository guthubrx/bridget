# Audit de réutilisation251
Décision : PASS.
## Synthèse
Réutiliser MessageIntent, négociation ClientHello, prepare_dispatch, transaction IdempotencyStore, ledger/search/read, fils notify:[], politique de maintenance existante. Aucun service/table/dépendance.
## Existant vérifié
- MessageIntent : crates/bridget-core/src/message.rs:54. Étendre, pas nouveau type parallèle.
- Canon : crates/bridget-daemon/src/communication.rs:147. Ajouter discriminant sans changer les formes anciennes.
- Store : store/ledger_requests.rs:729 et idempotency/send_delivery.rs:698. Réutiliser ledger/transaction ; nouvelle méthode métier terminale nécessaire car begin_send_delivery crée une remise.
- Fils : threads.rs:953-1006, store/threads.rs:1098 ; history+notify[] silencieux, autres entrées supersedes_seq. Aucun nouveau bulletin store.
- Journal : mcp.rs:1805-1910 ; recherche/lecture existantes. Aucune boîte supplémentaire.
- Observations : daemon.rs:3060-3115 / observation_missed.rs. Faits typés, pas prose libre ; ajouter contrôle dormant avant écriture.
- Files : daemon.rs:12195 / execution_store.rs:432 ; ne garantissent pas une simple conservation. Non réutilisées comme information.
## Recherche par nom et responsabilité
rg StoreOnly/information/silenc/journal ; prepare_dispatch/capture_reply/send_delivery ; record_message/INSERT ledger ; QueueOnly/ControlOnly ; notify/supersedes ; client capabilities. Sources et mission exploration c4225813 lues. Réponse Haiku lecture seule, résultat reçu/ACK, pas un test runtime.
## Similaires et décisions
Thread history convient aux bulletins partagés mais pas au dépôt direct simple ; aucune création de fil par destinataire. Observation_missed appartient aux abonnements : ne pas détourner. La nouvelle méthode store_information porte atomicité et non-réveil, pas un wrapper vide. Pas de nouvelle abstraction.
## Règles / mémoires / sessions
AGENTS, constitution/standards, mémoire et sessions102/104/135/156/159/239/241/244/247/248/250 consultés selon pertinence. Aucun effacement d'historique, ni droit supplémentaire.
## Arbitrages
CREER la variante StoreOnly et une opération sur IdempotencyStore : seule contrainte nouvelle non couverte par QueueOnly ; pas de nouvel état exécutable. Tout autre item sera recherché avant écriture.
## Gate avant tasks
- [x] Chaque ajout a une recherche par nom et responsabilité.
- [x] Briques similaires identifiées et exclusions justifiées.
- [x] Pas de duplication évidente ni refactor non arbitré.
- [x] Aucun arbitrage humain requis.
- [x] Plan et spec cohérents, statut PASS.

Recherche de guide à l'implémentation : docs/reference-communication.md et .fr.md existent. REUTILISER ces deux fichiers, aucun nouveau guide parallèle. Skills versionnées skills/bridget/SKILL.md et references/commandes.md étendues ; pas de nouvelle skill. Garde anti-fuite aux puits begin_send_delivery et defer_idempotent_delivery existants, aucun wrapper de transport nouveau.

R2 : recherche InformationCollision/id_conflict/collision dans idempotency.rs et send_delivery.rs. REUTILISER IdempotencyError (une variante métier fermée) et reject_idempotent_send, pas de nouvelle classe/endpoint. SELECT indexé sous Immediate distingue le fait de collision, sans parser de texte SQLite. Gardes de transport regroupées dans prepare_dispatch existant, sans helper parallèle.
