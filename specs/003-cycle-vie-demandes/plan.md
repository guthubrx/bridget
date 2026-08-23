# Plan : cycle de vie des demandes Bridget

**Branche** : `session-03-cycle-vie-demandes` | **Date** : 2026-08-14 | **Spec** : [spec.md](spec.md)

## Résumé

Faire passer les envois `--reply` du statut de simple rappel en mémoire à celui de demande suivie, persistée et annulable. L'objectif est de supprimer les relances inutiles sans transformer les notifications ordinaires en tâches.

## Contexte technique

**Langage** : Rust (édition définie par le workspace)  
**Dépendances principales** : `serde`, `rusqlite`, crates Bridget existantes  
**Stockage** : SQLite locale Bridget existante  
**Tests** : tests unitaires Rust et intégration par socket Unix  
**Plateformes** : macOS et Linux, daemon et wrappers via socket Unix  
**Type** : CLI et daemon local mono-repository  
**Objectif de performance** : transitions et consultation en temps constant ou logarithmique pour le nombre de demandes suivies ; aucune boucle de rappel sur les demandes terminales  
**Contraintes** : compatibilité avec les messages existants, pas de dépendance réseau ni de nouvelle base, reprise fiable après redémarrage  
**Périmètre** : cycle de vie des seules demandes `--reply`; interruption du raisonnement interne d'un LLM explicitement hors périmètre

## Contrôle constitutionnel

- **Minimalisme (XIX)** : une table SQLite et quelques transitions explicites remplacent l'état éphémère existant ; aucune abstraction de workflow généraliste ni dépendance additionnelle.
- **Responsabilité future (XX)** : les états terminaux et l'identifiant visible éliminent le raisonnement implicite par couple expéditeur/destinataire.
- **Complexité (XVIII)** : les accès par identifiant sont indexés ; la boucle de surveillance ne charge que les demandes `open` et ne conserve pas de scan des états terminaux.
- **ADR (VII)** : la décision est consignée dans `docs/decisions/001-cycle-vie-demandes.md`.

Le contrôle est **passé** avant et après conception : le périmètre couvre le besoin prouvé et réutilise le stockage et le transport existants.

## Conception

1. À la livraison d'un message avec `reply=true`, le daemon crée une demande suivie durablement avant de programmer ses rappels.
2. Une réponse porte l'identifiant de la demande à laquelle elle répond ; seul cet identifiant peut conduire à l'état `answered`.
3. `bridget cancel <id> [raison]` demande au daemon de faire la transition `open → cancelled`. Le daemon vérifie l'émetteur, supprime la demande de sa surveillance et notifie le destinataire.
4. Au démarrage, le daemon recharge les seules demandes `open` ; les délais sont calculés à partir d'horodatages persistés. Les demandes terminales ne reviennent jamais dans la boucle de rappel.
5. `bridget requests` expose à l'émetteur ses demandes suivies ; il n'expose pas les demandes des autres agents.

## Structure projet

```text
specs/003-cycle-vie-demandes/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── reuse-audit.md
├── contracts/
│   └── cli.md
└── tasks.md

crates/bridget-core/src/message.rs
crates/bridget-core/src/envelope.rs
crates/bridget-daemon/src/store.rs
crates/bridget-daemon/src/daemon.rs
crates/bridget-daemon/src/cli.rs
crates/bridget-daemon/tests/integration_test.rs
crates/bridget-transport/src/protocol.rs
```

## Choix de structure

Les modules existants restent responsables : le noyau définit le lien de réponse, le transport sérialise les opérations, le store persiste les transitions, le daemon applique les autorisations et la CLI les expose. Aucun nouveau service ni dossier de production n'est nécessaire.

## Complexité

| Opération | Complexité | Justification |
|---|---:|---|
| Créer, consulter ou annuler une demande par identifiant | O(log n) | index SQLite sur l'identifiant |
| Lister les demandes d'un émetteur | O(k) | k demandes de cet émetteur, indexées |
| Recharger les demandes ouvertes | O(o) | o demandes ouvertes uniquement au démarrage |
