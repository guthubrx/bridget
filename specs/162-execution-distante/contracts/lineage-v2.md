# Contrat de lecture Lineage 162

## Compatibilité et autorisation

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/contracts/lineage-v2.md#extrait-1).

Le nouveau reader tente v2. Une seule lecture v1 sans ce flag est permise sur refus structuré unsupported_version ou invalid_request attribué à cette tentative. Le CLI 160 retourne invalid_request JSON, RC2, pour un flag inconnu. Timeout, refus d’autorisation, données malformées ou autre code : conserver l’erreur, sans repli général. Aucun nouvel enum fermé envoyé à un ancien serveur avant compatibilité établie.

## Snapshot

V2 conserve génération, séquence, curseurs, propriété et limite de 4 096 tâches. Une tâche reçoit placement nullable : machine_id, nom configuré facultatif, worker_id, last_contact_at et contact_state. Les champs fournisseur/modèle existants restent la référence. last_contact est observé au maître depuis le transport. L’interface affiche cette date comme observation lors de la lecture de la page, jamais comme âge vivant non rafraîchi. Il reste volatile ; seules les transitions de contact_state provoquent notification. Les transitions de contact_state incrémentent seq et invalident le curseur. last_contact_at reste une observation lors de la lecture de chaque page, sans gel global. Le snapshot métier conserve ses garanties existantes.

Si une page v2 exige le repli autorisé, abandonner tout le staging v2 et reprendre v1 depuis le curseur nul. Ne jamais mélanger versions, pages ou générations. En v1 de repli, afficher « Métadonnées distantes indisponibles », jamais une machine locale supposée.

## Réemploi et présentation

Conserver les quatre RPC read/watch/journal/cancel et leurs gardes orchestration:read/operate. Watch v1 reste inchangé. Aucun repli de version pour journal ou cancel. Garder la projection thread:bridget-task:<uuid> sans session fournisseur, absente de la colonne des conversations. Afficher machine et contact dans le Lineage, l’infobulle et le détail existants.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/contracts/lineage-v2.md#extrait-2).

## Oracles

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/contracts/lineage-v2.md#extrait-3).
