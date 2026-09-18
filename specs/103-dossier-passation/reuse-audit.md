# Audit de réutilisation — 103-dossier-passation

## Décision

Statut: PASS
Date: 2026-09-16. Lecture code main1738a072 + specs102, revue indépendante Explore.
PASS porte sur le plan documentaire et la réutilisation, pas sur du code exécuté.

## Synthèse

Un module de validation/rendu handoff.rs, un outil métier bridget_handoff et une sous-commande handoff. Aucun nouveau stockage/service/dépendance.
Le plan a été écrit après l'exploration, pas refactoré silencieusement après découverte
d'un doublon. Aucun arbitrage produit bloquant ni duplication évidente laissé ouvert.

## Réutilisations correctement identifiées

Racine des preuves relatives du tableau :
/Users/moi/Nextcloud/10.Scripts/64.bridget/

| Élément proposé | Existant et preuve | Décision |
|---|---|---|
| Transmission du dossier | crates/bridget-daemon/src/mcp.rs:588 execute_send ; communication.rs:64 | RÉUTILISER sans champ protocolaire nouveau |
| Validation/rendu métier | aucun handoff/bundle de communication trouvé ; attach.rs:128 rend seulement un extrait | CRÉER handoff.rs, règle de structure/bornes/déterminisme, pas transport |
| Persistance | store/ledger_requests.rs:794 et store.rs:59 | RÉUTILISER le corps ledger ; aucune table |
| Dossier artefact partagé | artifact_store.rs:863 share_with_agent, artifact_service.rs:281 scope de lecture | NE PAS DÉTOURNER ; ne permet pas le partage inter-agent promis |
| Rejeu | idempotency/send_delivery.rs:17, mcp.rs:588 | RÉUTILISER099 |
| Contexte source | attach.rs:35 JournalRequest et :67 JournalExcerpt | Référencer/sélectionner explicitement ; aucune capture automatique |
| MCP/CLI/skill | mcp.rs catalogue, cli.rs envoi, wrapper.rs BRIDGET_SAFE_MCP_TOOLS | ÉTENDRE les surfaces existantes |


## Existant potentiellement pertinent non mentionné

Aucun après intégration de l'exploration. Anciennes fonctions Maicie et docs de handoff
ne sont pas des services de passation/recherche à réactiver. Les primitives d'artefact
ont été évaluées explicitement, pas ignorées.

## Duplications évidentes

Aucune dans le plan retenu. Interdit à l'implémenteur de recréer Send, une base de recherche,
un générateur de résumé, une file de notifications ou un contrôle d'accès parallèle aux fils.
Rechercher encore par nom ET responsabilité avant toute nouvelle création non prévue.

## Mémoires et règles applicables

Constitution globale1.8, AGENTS.md, .specify/memory/constitution.md et standards.md lus.
Articles VII/XVI/XVIII/XIX/XX : décision tracée, isolation, coût borné, réutilisation et
preuves de comportement. mem absent ; mémoire projet Bridget absente. Research/ADR
servent de repli documentaire, pas d'équivalent DevKMS prétendu.

## Specs livrées applicables

089 cœur de communication,094parité des accès,099envoi fiable,100journal/observations.
082/083artefacts évalués comme existant mais non réutilisés pour élargir les droits.
101observationsT3 et102fils restent dans leurs worktrees ; ne pas déduire leur présence
dans main des seuls documents. Relation exacte102 définie dans spec.md/plan.md.

## Journal de recherche

Explore lecture seule par nom handoff/bundle/search_messages et responsabilité search/ledger/
artifact publish/read/share/journal ; inspection des appels, des tests et de la rétention.
Commandes : rg -n, rg --files, git status/worktree ; aucun test/runtime lancé.
Sources web primaires et limites capturées dans research.md ; données envoyées synthétiques.

## Arbitrages

Décisions minimales arrêtées dans research.md. Pas de nouvelle dépendance.
La politique existante du ledger n'est pas présentée comme un nouvel espace confidentiel.
Si l'implémenteur rencontre un équivalent nouveau après intégration d'autres branches,
réauditer avant création ; ne pas invoquer ce PASS pour forcer un doublon.

## Gate avant tasks

- [x] Tous les services/composants/tables/endpoints/dépendances proposés sont recensés.
- [x] Recherche par nom et responsabilité exécutée, preuves localisées.
- [x] Réutilisations/limites des specs livrées et prévues distinguées.
- [x] Aucune duplication évidente ni arbitrage utilisateur non résolu.
- [x] Plan fondé sur l'existant ; génération des tâches autorisée.

