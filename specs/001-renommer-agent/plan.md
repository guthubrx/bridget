# Plan d’implémentation : Renommer un agent Bridget

**Branche** : `session-01-renommage-agent-bridget` | **Date** : 2026-08-14 | **Spec** : [spec.md](spec.md)

## Résumé

Permettre à un agent Bridget connecté de remplacer son nom sans se reconnecter. Le routeur remplace atomiquement sa clé d’annuaire ; le wrapper conserve le nom confirmé afin que la reprise reprenne cette identité. Les sous-commandes consultent un fichier d’état local partagé, afin de ne jamais réutiliser le nom figé lors du lancement.

## Contexte technique

**Langage/version** : Rust, édition 2024  
**Dépendances principales** : `serde`, `serde_json`, `uuid`, `log`  
**Stockage** : fichiers locaux de noms persistés pour les wrappers  
**Tests** : `cargo test` (tests unitaires et d’intégration Rust)  
**Plateformes cibles** : macOS et environnements Unix disposant de sockets Unix  
**Type de projet** : CLI et démon local  
**Objectif de performance** : renommage local perceptible comme immédiat ; aucune reconnexion de l’agent  
**Contraintes** : unicité des noms, cohérence de l’annuaire, compatibilité des messages existants et fichier d’état local accessible au processus de l’agent  
**Périmètre** : protocole de transport, routeur, démon, CLI, wrapper et tests associés

## Vérification de la constitution

- Spécification, plan et tâches rédigés en français : conforme.
- Changement fonctionnel documenté avant code : conforme.
- Solution minimale : une opération de protocole dédiée et un fichier d’état local du wrapper, indispensable pour que les commandes enfant utilisent le nom courant ; aucun service, base de données ou abstraction supplémentaire : conforme.
- Responsabilité mainteneur : contrat CLI explicite et tests couvrant succès, collision, ancien nom et persistance : conforme.
- Changement structurant : documenter l’interface de commande comme contrat, sans ADR séparé car l’architecture existante reste inchangée.

Vérification après conception : conforme ; aucun point non résolu.

## Structure du projet

```text
crates/
├── bridget-core/src/router.rs                 # Annuaire et routage des agents
├── bridget-transport/src/protocol.rs          # Messages wrapper ↔ démon
├── bridget-daemon/src/cli.rs                  # Commande de renommage et lecture du nom courant
├── bridget-daemon/src/daemon.rs               # Traitement côté démon
├── bridget-daemon/src/wrapper.rs              # Persistance, connexion durable et état local
└── bridget-daemon/tests/integration_test.rs   # Parcours de bout en bout

specs/001-renommer-agent/
├── research.md
├── data-model.md
├── contracts/rename-agent-cli.md
└── quickstart.md
```

**Décision de structure** : modifier les composants déjà responsables de l’identité, sans nouveau module. Les tests d’intégration vérifient le parcours complet et les tests du routeur protègent l’invariant d’unicité.

## Complexité

Aucune violation ou complexité additionnelle à justifier.
