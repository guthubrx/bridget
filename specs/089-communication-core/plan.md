# Plan 089 — Extraire sans réécrire les garanties

**Branche :** session-089-communication-core | **Date :** 2026-09-05 | **Spécification :** spec.md
**Base immuable :** dfa2134dcfe2a2522e3ae77d93561e6ae72556b3.
**État :** préparation réalisée ; extraction et gates fonctionnels non exécutés.

## Résumé et approche

Créer un dépôt indépendant avec historique, puis extraire par coutures testées. Le premier jalon utile est un échange réel Codex↔Claude sans GUI ni Maicie. La fédération SSH, la reprise et la sécurité ne sont pas des options de finition : leur contrat est verrouillé avant de couper les dépendances.

Le clone contient encore tout le produit historique. Il ne devient « communication-only » qu'après suppression des dépendances et passage SC-08906. Ni un renommage de dossier, ni des features Cargo cachant tout le code, ni une réécriture intégrale ne constituent l'extraction.

## Contexte technique

- Rust 1.92.0, édition 2024, toolchain et lockfile existants conservés.
- Trois crates cibles : bridget-core, bridget-transport, bridget-daemon. SQLite/rusqlite, JSONL/serde, sockets Unix et SSH ; pilotes natifs existants.
- Tests comportementaux Rust existants conservés. Scénarios métier en Gherkin lisible associés aux tests Rust ; aucun runtime Python de tests ajouté au produit.
- macOS local, Linux distant ; processus et données de tests entièrement isolés.
- MCP actuellement 2025-06-18 ; pas de montée de version implicite.
- Budgets : RPC MCP existant 10 s global, bornes de trames/files/cursors épinglées à la caractérisation ; aucune augmentation pour faire passer un test.
- Mesures attach historiques à reconduire : 10 événements/s pendant 60 s, p95 local <1 s et max <3 s ; p95 distant <3 s avec biais d'horloge documenté. Les budgets relatifs d'append déjà corrigés restent ceux des tests épinglés, sans retour au seuil naïf.

## Vérification constitutionnelle

| Gate | Décision |
|---|---|
| Périmètre utilisateur | Session 089 et nouveau répertoire approuvés. Aucun chantier GUI, Routr, A2A serveur ou nouvelle orchestration. |
| Isolation | Clone indépendant et worktree dédié ; aucun déploiement, aucun registre/live DB importé, aucun push vers l'ancien dépôt. |
| Minimalisme | Réutiliser traits, canons, ledger et transactions. Aucun framework externe ni nouvelle crate sans démonstration d'une dépendance circulaire impossible à couper par module. |
| Tests avant suppression | Épingler corpus/commandes/baseline ; tester des mutants comportementaux, pas une re-sérialisation contre elle-même. |
| Sécurité | Conservation des protections IPC, processus et facturation ; tests adverses à chaque tranche, pas un audit final uniquement. |
| Revue indépendante | À proposer avant coupe de production et avant bascule ; aucun sous-agent ou déploiement déclenché automatiquement. |
| Données | Migration uniquement de copies de fixtures ; adoption des données utilisateur sous autorisation distincte. |
| Qualité | Erreurs typées, autorité transactionnelle unique, API petites, graphe sans cycle. fmt/clippy/tests comme gates de chaque tranche. |

Aucune dérogation de production accordée. Un échec externe bloquant doit rester non validé, pas devenir un skip silencieux.

## Frontières cibles

1. **bridget-core** : identifiants, enveloppes et invariants de communication purs.
2. **bridget-transport** : protocole public, codecs et contrat ManagedSession ; adaptateurs ACP/Codex/Claude ; journal et provenance.
3. **bridget-daemon** : transactions, annuaire, requests, delivery, ledger, lifecycle de processus et reprise. CLI/MCP/attach sont des façades de ces mêmes services.
4. **Consommateurs extérieurs** : Maicie, future UI/T3 et Routr n'importent pas le store. Ils utilisent les interfaces publiques autorisées.

Les modules de stockage et de transport ne doivent jamais importer cli.rs/mcp.rs pour construire un canon, une identité ou une issue. Les helpers réellement partagés sont déplacés vers un module neutre, non recopiés. Pas de trait générique de repository pour masquer SQLite : la transaction constitue ici l'autorité.

## Carte initiale de découplage

| Zone existante | Traitement prévu et vérification |
|---|---|
| Cargo.toml ; crates/bridget-daemon/Cargo.toml | Retirer le membre métier obligatoire et la dépendance maicie après extraction des lectures neutres. cargo metadata/cargo tree et installation sans plugin doivent le prouver. |
| crates/bridget-daemon/src/cli.rs, mcp.rs | Garder communication/processus/observation, contenus référencés et services négociés ; retirer commandes UI/projets et prévisualisation, refus de commande explicite. |
| crates/bridget-daemon/src/daemon.rs | Séparer dispatch communication, autorisation et orchestration de projet ; ne pas déplacer des branches aveuglément. |
| crates/bridget-daemon/src/store.rs, idempotency.rs | Préserver ACK+ledger+answered+événement dans une transaction, migrations testées ; table inutilisée ne signifie pas table supprimable sans inventaire. |
| crates/bridget-daemon/src/ui.rs, mission_projection.rs, project_runtime.rs et modules artifact_* | Retirer présentation/runtime ; classer artifact_* avant coupe : préserver la publication/référence de contenu utile aux messages, retirer le renderer/sandbox HTML. L'histoire reste disponible dans Git. |
| crates/bridget-daemon/src/wrapper.rs, registry.rs, managed_process.rs | Préserver cycle de vie, permission, capacités, chemins absolus et définitions figées ; retirer la dépendance au runtime de projet, pas les garde-fous fournisseur. |
| crates/bridget-transport/src/managed_session.rs et trois pilotes | Réutiliser le contrat corrigé (raw/source/origine/terminal typé) ; journal effectif autorise attach ; aucune simulation ACP par un pilote natif. |
| crates/bridget-transport/src/protocol.rs | Isoler les familles, préserver corpus filaire retenu et négociation ; ne pas remplacer par un DTO local « équivalent ». |
| scripts/federate-ssh.sh, deploy-remote.sh | Paramétrer les chemins/labels, préflight sans écrasement, installation séparée ; pas de lancement du script historique contre la production. |
| skills/bridget/SKILL.md (nouvelle source livrable) | Courte, tirée du contrat vérifié ; ne pas écraser les skills globales durant le développement. |

Les clients Maicie restent externes, mais la communication durable avec un service, son contrôle de capacité et ses faits de lifecycle restent inclus. Déplacer une politique d'approbation/réassignation hors du daemon n'autorise pas à supprimer les messages publics qu'elle consomme.

## Phasage et points d'arrêt

### P0 — Inventaire et contrats (T001–T006)

Inventaire exact des couplages/commandes, fixtures gelées avec empreintes et mutation d'un octet, inventaire des tests retenus/retirés, menaces et espaces isolés. Produire une baseline exécutable dans un worktree de la référence, jamais dans la production. Le premier jalon de revue vérifie surtout ce qui serait perdu, avant suppression.

### P1 — Couper les dépendances (T007–T013)

Installer la configuration indépendante avant tout démarrage. Extraire façade et lectures communes, découpler Maicie, puis UI/runtime de projet ; conserver schéma utile et compatibilité explicitement testée. Chaque commit compile et sa tranche de contrats passe. Mesurer le graphe final plutôt que compter des fichiers renommés.

### P2 — Communication et observation locales (T014–T023)

Équivalence CLI/MCP/skill, identité et réponses liées, reprise crash, journal et pilotes. Tests de vrais processus et watchdogs ; SIGKILL uniquement dans un harnais autorisé de processus de test, jamais la flotte de production. Si les règles locales interdisent un tel signal, obtenir un arbitrage explicite avant le test, ne pas remplacer le crash par SIGTERM.

### P3 — SSH comme gate obligatoire (T024–T028)

Réutiliser le transfert de socket, pas un protocole réseau additionnel. Préflight sûr, journal et ledger maître, coupure/reconnexion, deux machines réelles ; une sonde locale seule n'est pas la recette interserveur. Documenter commande exacte et nettoyage des seuls objets de test. Ne pas désactiver la vérification de clé d'hôte.

### P4 — Durcir et préparer l'adoption (T029–T036)

Refus hostiles, concurrence, compétences déclarées, métriques avant/après, corpus complet, documentation d'installation isolée/retour arrière et revue. Installation/bascule de la flotte sur décision utilisateur séparée. T3 reste une suite éventuelle, pas une dépendance à la clôture de 089.

## Stratégie de preuve

Chaque critère SC a une commande, un résultat, une durée et un oracle dans implementation.md. Au moins un mutant pertinent pour corrélation, idempotence, source brute, Gap/fraîcheur, refus avant processus, deadline globale et namespace isolé doit échouer. Aucune fermeture de critère par la seule inspection d'un test non exécuté.

Les tests unitaires utilisent uniquement des dépendances contrôlées ; les gates de couture traversent les binaires réels, sockets, SQLite et processus. Les gates fournisseur utilisent un compte autorisé, sans migration silencieuse vers une facturation API. Des octets bruts de sortie réels ne sont versionnés qu'après vérification d'absence de secrets.

Fichiers de validation nouveaux : crates/bridget-daemon/tests/core_089_*_test.rs ; scénarios lisibles : tests/features/089-communication-core.feature. Les tests historiques sont gardés sous leurs noms ; déplacer un test ne doit pas effacer son historique de défaut.

## Risques et réduction

- **Store monolithique :** schema/version/FK mêlés ; inventorier tables et appels avant toute suppression, fixtures anciennes et ouverture concurrente.
- **Coutures masquées :** CLI et MCP peuvent diverger sans conflit textuel ; comparer leur passage dans le même daemon, pas uniquement leurs faux serveurs.
- **Session fournisseur :** capacité annoncée n'implique pas livraison ; vérifier mission, réponse et journal réels, limites de garantie explicites.
- **Régression SSH :** scripts liés au home et launchd historiques ; chemins obligatoires de test et refus de cible occupée.
- **Réécriture à rallonge :** pas de nouveau framework, une tranche utile à la fois, arrêt de toute abstraction sans lecteur réel.
- **Compatibilité :** négocier/rejeter les anciennes opérations hors périmètre, ne pas les ignorer ni annoncer une capacité non implémentée.

## Complexité suivie

Aucune dépendance A2A/LangChain/LangGraph retenue. Conservation temporaire du produit complet dans le clone pour comparaison, pas dans le livrable final. Temps d'extraction encore non mesuré ; premier recalage après P0, avec coût des coutures restantes plutôt qu'un pourcentage de lignes supprimées.
