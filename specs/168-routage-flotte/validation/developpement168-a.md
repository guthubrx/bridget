# Développement168 — lot A

Date : 2026-10-15.
Owner : DEV168_A.
Portée : T005, T006 et préparation T025, exclusivement registre et fixture.

## Livré au raccord

- `routing_policy` v2 optionnelle dans `AgentRegistryFile` ; absente, elle conserve le parcours163.
- Connexions, catalogue par connexion, provenance, efforts compatibles et défaut natif explicite.
- Projection pure `AgentRegistry::routing_candidates()` : connexion×modèle déclarés, y compris désactivés, sans lancement, découverte ou authentification.
- Projection publique `AgentRegistry::routing_catalogue(observed_at)` et sélection pure
  `AgentRegistry::select_routing_candidates(coordinator_machine_id, filters)` pour les raccords C.
- Validation stricte des connexions, modèles, efforts, axes fixes, TTL, budgets, services et décideur Claude local.
- Borne complète 1..128 avant filtres : dépassement = `candidate_limit`, sans catalogue partiel.
- Les décideurs déclarés inaccessibles restent valides : C peut les ignorer sans tentative ; la référence et la localité Claude restent fermées.
- `compilation_capacity: Option<u8>` est déclarative et bornée 0..10 ; l'absence reste inconnue.
- Fixture déclarative Codex/Claude avec défaut natif et connexion Cartae non provisionnée désactivée.

## Oracles source ajoutés, non exécutés

- Catalogue complet de 12 options, deux connexions Codex distinctes, Claude déclaré, efforts réels et défaut natif nul.
- Refus `candidate_limit` avec 129 options, y compris une connexion désactivée.
- Axes indépendants, valeur fixe et contradiction comme filtres sans bascule.
- États décideur inaccessibles admis et refus d'une connexion Claude non locale au coordinateur.
- Capacité compilation absente/inconnue puis borne 10 admise et 11 refusée.

## Contrôles réalisés

- `rustfmt --edition 2024 crates/bridget-daemon/src/registry.rs` : succès.
- `git diff --check` : succès.
- Aucun Cargo, compilation ni test exécuté : aucun résultat PASS n’est revendiqué.

## Interfaces publiées à ROOT

Message Bridget `mcp-3299110-6ad08992-1` (remise en vol) :
`RoutingPolicy`, `RoutingConnection`, `RoutingModel`, `RoutingModelProvenance`,
`RoutingMachinePolicy`, `RoutingCandidate`, `AgentRegistry::routing_policy()` et
`AgentRegistry::routing_candidates()`, puis `RoutingCandidateFilters`,
`RoutingCandidateSelection`, `AgentRegistry::routing_catalogue()` et
`AgentRegistry::select_routing_candidates()`.

## Reste à ROOT et aux consommateurs

- Raccorder les candidats déclarés à la façade catalogue, aux filtres et à l’admission v2.
- Exécuter les oracles 12/128/129 et les scénarios de deux environnements Codex avec le permis Cargo ROOT.
- La fixture ne constitue pas une attestation d’entitlement ou d’activation fournisseur.
