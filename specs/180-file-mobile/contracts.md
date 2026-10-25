# API mobile 180 — extensions v1

GET /v1/message-queue/<agent_id> : items en phase held du téléphone courant, body complet et revision SHA256 minuscule ; receipts expose les phases des remises du même téléphone et destinataire. Aucune portée d’un autre téléphone n’est retournée.

POST même chemin : command_id, request_id UUIDv4, action edit/cancel/promote, expected_body SHA256 minuscule du texte affiché ; body obligatoire pour edit et interdit ailleurs. Le téléphone conserve request_id et enveloppe en cas de réponse perdue. Une révision différente est refusée. Un même request_id utilisé pour une autre action est refusé.

Résultat appliqué : status ok, request_id, state applied. L’issue inconnue ne retire pas l’entrée locale. Cancel et promote retournent state refused et un code natif si le message n’est plus retenu ou si le moteur ne peut pas guider le tour. Les erreurs HTTP portent le même code fermé. Aucun code ne promet un arrêt ou un redémarrage du fournisseur.

L’édition ne change ni l’identifiant, ni la position, ni le canon de création. Les octets remis et le ledger reçoivent le texte corrigé sous la même transaction. La projection d’une promotion relit le reçu durable même après reprise. Les enveloppes de création ne sont jamais réécrites pour simuler un autre envoi.
