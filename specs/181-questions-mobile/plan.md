# Plan181 — Questions natives Bridget

Statut : livré avec conservations justifiées.25/25tâches ; ConvergeP1/P2/P3 ; auditpass14/14 ; source0.6.2et release publiées. Contre-revue indépendante sans verdict (api_error/délai), réserves et reçus de livraison conservés.

## Architecture et réutilisation

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/plan.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/plan.md#extrait-2).

Créer un module natif user_input.rs et une table additive de demandes structurées, car human_inbox est limité aux décisions offertes et native_child_requests est strictement lié aux délégations155 et à leurs permissions héritées. Ne pas affaiblir ces contrats pour transporter une réponse libre d’un agent principal. Aucun nouveau daemon, service, file réseau, dépendance ou modèle vocal.

## Contrat et état

Identifiant UUID de demande durable, agent/instance/projet issus de la connexion authentifiée, questions bornées non secrètes, empreinte canonique, origine et callback natif exact. Réponse structurée : sélections et texte pour chaque question ; un seul choix si multiSelect=false. Le complément n’est pas interprété. Canon, commande et réponse sont immuables après consignation.

Transitions : pending -> submitted -> applied ; pending -> resolved_elsewhere/expired/cancelled ; submitted -> unknown/closed si la capacité native disparaît. submitted atteste l’enregistrement, applied nécessite le reçu du producteur. Pas de réponse réinjectée en prompt ordinaire. Rejeu exact rendu depuis reçu ; même clé/autre enveloppe refusée. Compétition de deux réponses traitée en transaction SQLite. Instance/projet/capacité revalidés avant remise.

Conservation7jours ;32demandes ouvertes par agent,2048au total ; corps total64Kio maximum ;64questions/options cumulées bornées ; texte libre10Kio par réponse,32Kio pour l’ensemble. Afficher les refus sans tronquer. Index par agent/état et clé de commande. Coût O(octets+questions/options) pour validation, O(log n) pour idempotence et mutation, lecture bornée.

## Interfaces

Outil MCP bridget_ask_user : création idempotente, état/attente/reprise/annulation propres au producteur. La commande humaine bridget questions permet liste, lecture et réponse sur le même contrat, par le parcours natif Bridget. L’agent n’obtient jamais le droit de répondre à sa propre question comme humain.

Les callbacks codex_app_server existants sont réemployés pour les questions d’agents principaux ; les demandes de délégation155 conservent leur routage et leurs permissions. Le transport Claude stream-json reconnaît uniquement AskUserQuestion comme question libre ; ses autres permissions restent refusées selon le contrat actuel. Les TUI opaques sans callback utilisent l’outil MCP Bridget, sans simulation de touches. Le guidage de lancement doit nommer ce chemin compatible.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/plan.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/plan.md#extrait-4).

Notification : conserver la poussée FCM sans contenu ; relire les questions via HTTPS authentifié avant rendu privé. Ajouter choix courts et RemoteInput pour une question simple ; ouvrir le bon formulaire pour les demandes complexes. Receiver non exporté, PendingIntent explicite et identité de liaison, question et révision figées ; vérifier l’état au traitement et conserver la commande en cas d’incertitude. Les réponses de notification et de l’écran utilisent la même règle métier. Les heures silencieuses restent appliquées.

## Vérification

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/plan.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/plan.md#extrait-6).

## Constitution, livraison et maintenance

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/plan.md#extrait-7).

## Fallback des primitives

Skills Speckit disponibles et lues sur Mac. Les scripts setup-plan.sh et check-prerequisites.sh ont été tentés dans le checkout181 : No such file or directory. Les templates projet manquent aussi. Protocole des skills exécuté directement, artefacts complets conservés ; pas de fausse invocation de commande slash. Analyze et Converge seront exécutés selon leur protocole manuel, avec résultats tracés.

## Précisions après Analyze

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/181-questions-mobile/plan.md#extrait-8).
