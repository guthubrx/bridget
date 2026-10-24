# Contrat mobile V1

Ce contrat porte une identité humaine d'appareil. Il n'expose pas le protocole interne complet du daemon.

## Transport

HTTPS public ; écoute de façade sur loopback. Version explicite ; JSON strict sans champs inconnus pour les mutations. Autorisation hors URL. Réponses non mises en cache. Pagination et abonnements bornés.
Une origine et un chemin inattendus sont refusés. Aucun endpoint d'exécution de shell.
Les limites exactes de taille, fréquence, durée et connexions sont fixées avec les tests du serveur HTTP avant activation.

## Association et administration

- L'installation locale de confiance crée une association temporaire et son QR.
- POST /v1/pair : consommer une association pour le même appareil et rendre son autorisation.
- GET /v1/devices : appareils autorisés, selon le droit de gestion.
- DELETE /v1/devices/{device_id} : révoquer et fermer les abonnements correspondants.
- Le droit d'émettre une nouvelle association ne peut pas être obtenu en changeant un champ d'une requête distante.

## Lecture

- GET /v1/federation : machines et état de disponibilité.
- GET /v1/projects : projets autorisés.
- GET /v1/agents : page d'agents autorisés, filtre machine/projet.
- GET /v1/agents/{agent_id}/events : page du journal avec séquence et éventuelle lacune.
- GET /v1/tasks et GET /v1/tasks/{task_id} : projections natives autorisées.
- GET /v1/commands/{command_id} : reçu exact dans la portée de l'appareil.
- GET /v1/events : suivi borné, reprise par repère opaque ; révocation recontrôlée.
- GET /v1/artifacts/{artifact_id} : référence d'artefact autorisée, jamais un chemin arbitraire.

## Mutations

- POST /v1/commands : commande à destinataire explicite ; device_id, command_id, issued_at, type fermé et enveloppe figée.
- Types utiles : message, création de tâche à partir d'un profil autorisé, réponse à une question, continuation, pause et annulation quand disponibles.
- La route valide la portée avant d'atteindre le puits natif. Elle ne synthétise aucune permission fournisseur.
- Les clés et reçus natifs restent la base de la réconciliation.
- Une enveloppe différente sous une même clé donne envelope_mismatch.
- Une issue ambiguë donne outcome_unknown et une action de lecture, jamais une consigne de nouveau lancement.
- POST /v1/artifacts : uniquement dans les limites et types admis par l'artefact natif.

## Refus

Objet stable : version, status, code, message, recovery, request_id éventuel.
Distinctions nécessaires : authorization_required, device_revoked, forbidden_scope, capability_unavailable, endpoint_unavailable, invalid_request, payload_too_large, envelope_mismatch, outcome_unknown, idempotency_expired, resync_required et refus natif conservé.
Un timeout du transport ne confirme ni échec métier ni annulation.

## Compatibilité

GET /v1/status expose seulement version de protocole et compatibilité ; aucune donnée d'agent publique.
Une version incompatible est refusée avant toute mutation.
L'application garde la version précédente des données locales jusqu'à validation d'une migration.

## Précisions avant implémentation

- Association : secret valable cinq minutes. L'appareil fige sa clé et un identifiant de tentative avant échange. Une réponse perdue autorise seulement la réconciliation de cette même tentative et de cette même clé, jamais la création d'un autre appareil.
- Portée : projets, machines et opérations explicites. Tout-projets et toute-machine sont des choix administratifs explicites ; une liste vide n'accorde rien.
- Administration de l'association : uniquement depuis l'installation locale de confiance. Un simple nom de principal reçu dans une requête n'autorise aucune mutation.
- Limites initiales : 64 Kio par commande JSON, 50 agents ou événements par page, 16 appareils actifs, 4 abonnements par appareil, 32 requêtes concurrentes, 20 secondes par requête hors suivi et 25 secondes par cycle de suivi. Les fichiers suivent en plus la borne native et un plafond mobile de 8 Mio.
- Les limites mémoire et connexions s'appliquent avant lecture complète du corps. Une saturation rend busy avec un délai borné, sans créer de commande.
- L'authentification est recontrôlée sur chaque cycle de suivi ; la révocation ne dépend pas d'une connexion Internet nouvelle.
- L'admission humaine mobile exige un profil et un projet déjà autorisés. Elle constitue un accès humain nouveau et explicite au puits natif, jamais un parent synthétique ou un droit de fournisseur fabriqué.

## Décision utilisateur — 24 octobre 2026

Première livraison : piloter les agents existants et leurs tâches. La création directe de nouveaux agents est reportée comme extension. L'envoi à un agent existant reste possible ; cet agent applique ses droits de délégation. Le téléphone ne reçoit aucun droit de lancement direct par anticipation.

## Disponibilité de fédération

L'inventaire des machines reprend le registre et le contact worker natifs : connected, stale, observed_agent ou unknown. Une machine sans agent courant n'est pas supprimée. Aucune adresse, clé, correspondance de disque ou catalogue fournisseur n'est projetée. La portée machine est explicite ; les objets de projet restent filtrés séparément.

## Historique et portée partielle

Les journaux natifs ne prouvent pas encore une partition historique par projet et machine. La lecture mobile de cet historique exige donc une autorisation explicite tous-projets et toutes-machines, en plus de read. Une portée partielle reçoit history_scope_unavailable sans contenu. Un filtre d'affichage ne change jamais cette règle. Cette limite reste à lever ou à qualifier avant clôture de T011.

## Origine des données conservées sur Android

Une commande conserve son origine HTTPS et son appareil dans le magasin protégé avant tout envoi. Le daemon compare device_id au principal attesté par la clé. Envoi et lecture de reçu sont refusés avant réseau si une réassociation a changé cette origine ; le code est command_origin_changed. Un renouvellement conserve aussi endpoint et device_id ; une origine différente donne authorization_origin_changed et ne transmet pas l’ancienne clé. Les brouillons sont séparés par origine, appareil et agent ; un renouvellement de clé du même appareil conserve l’accès. Aucune donnée ancienne n’est supprimée pour faire réussir une nouvelle association.

## Suivi borné et contrôle natif

GET /v1/events accepte agent_id, after (repère opaque) ou after_seq pour une première lecture, et limit 1..50. Le repère lie version, appareil, agent et séquence ; il ne confère aucun droit. Le cycle utilise le flux natif, dure au plus 25 s et contrôle la révocation chaque seconde ainsi qu’avant la réponse. Quatre cycles par appareil, six au total, laissent deux travailleurs aux autres actions. Les appareils sont paginés sur GET /v1/devices après et limit 1..50. Une erreur ne réinitialise aucune commande.

L’identité d’une commande est le triplet appareil, opération native et command_id. L’enveloppe reste figée. Le message garde issued_at selon son contrat natif. L’annulation reprend son contrat natif request_id/task_id, sans horodatage client ajouté artificiellement ; sa création locale reste datée sur le téléphone. La lecture du reçu d’annulation ne déclenche jamais cette annulation.

## Fichiers natifs

POST /v1/artifacts reçoit l’enveloppe device_id, command_id, agent_id, file_name, media_type, expected_digest, bytes en base64. Le fichier décodé est limité à 8 Mio, deux transferts entrants simultanés ; une clé sans droit message est refusée avant lecture de ce corps. Nom sans chemin, PDF/texte UTF-8/PNG/JPEG et signature d’entrée contrôlée. Le reçu de publication natif est relu par GET /v1/artifacts/uploads/{command_id}. GET /v1/agents/{agent_id}/artifacts accepte after/limit ; le curseur date/référence conserve les publications du même instant. POST /v1/artifacts/read reçoit action artifact_read, agent_id, artifact_ref, version_ref, offset, limit (16 Kio maximum). La version fixe projet, type, taille et digest ; aucun chemin de disque. La lecture historique exige read et une autorisation explicite de fédération complète ; elle reste possible agent hors ligne. Une publication exige l’agent courant et sa portée message. Les fichiers Android sont conservés chiffrés sous leur clé de commande, avec l’origine en données authentifiées, un budget local de 32 Mio et un effacement explicite des seules commandes soldées ou non envoyées.

## Tâches et décisions

GET /v1/tasks exige agent_id et accepte after/limit ; GET /v1/tasks/{task_id} accepte offset et livre le résultat natif par fragments de 16 Kio. Les deux lectures exigent task_read sur la fédération complète. La projection distingue actions natives et instructions au parent. POST /v1/commands type task_cancel reprend device_id, command_id et task_id ; GET /v1/tasks/{task_id}/cancel/{command_id} ne fait que relire le reçu. La clé reste liée à une seule tâche, y compris entre deux racines. Un résultat disponible n’est jamais effacé par une annulation tardive.

GET /v1/questions et /v1/questions/{item_id} reprennent la boîte humaine native, ses options et faits bornés. POST /v1/commands type question_respond fige device_id, command_id, item_id et choice. GET /v1/questions/{item_id}/responses/{command_id}?choice=... relit la même enveloppe sans décider. Le choix doit être offert. Un rejeu exact relit la décision native ; un autre item ou choix sous la même clé est refusé. Le téléphone ne forge aucun ACK. Le reçu distingue producer_acknowledged et application.confirmed : un ACK peut aussi solder une demande devenue invalide. Une application fournisseur n’est confirmée que par le reçu natif delivered ; elle reste inconnue pour un producteur qui ne fournit pas cette preuve. task_respond exige une autorisation complète explicite.

## Protection du bootstrap derrière HTTPS

La façade écoute uniquement en loopback. Le proxy installé remplace toujours X-Bridget-Client-IP par l’adresse de la connexion distante ; il ne reprend pas la valeur fournie par le client. L’association est limitée à 30 demandes par minute et par adresse, dans un index éphémère de 1024 adresses. Le renouvellement est limité séparément par appareil connu, après résolution de la clé courante ou de son reçu de rotation encore valable. Une clé inconnue n’alloue aucun compteur. Les limites de connexion et de travail restent communes et bornées.

Le suivi d’historique exige une portée complète avant toute lecture native. L’absence d’une ressource et une ressource hors portée peuvent être masquées par not_found pour ne pas divulguer son existence. L’autorité native réauthentifie l’appareil indépendamment de la frontière HTTP.
