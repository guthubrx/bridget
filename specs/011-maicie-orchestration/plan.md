# Plan d'implémentation : Maicie v3, coordinatrice d'orchestration légère

**Branch**: `session-11-maicie-orchestration` (à créer avant implémentation) | **Date**: 2026-08-22 | **Spec**: `specs/011-maicie-orchestration/spec.md`

## Summary

Créer Maicie comme compagnon Rust toujours joignable sous `plugins/maicie/`.
Elle utilise le contrat public de Bridget pour l'annuaire et les demandes
suivies, et une SQLite privée pour les objectifs et délégations. Le MVP livre
une coordination volontaire par CLI/JSON et une agrégation factuelle des
réponses ; les faits ACP sont affichés sans inférence métier. GUI, TUI,
scheduler, interpréteur LLM et auto-réveil sont exclus.

## Technical Context

**Language/Version**: Rust 2024 (workspace existant)  
**Primary Dependencies**: `serde`, `serde_json`, `uuid`, `rusqlite`, `log` et
`env_logger` du workspace ; aucune dépendance nouvelle sans ADR  
**Storage**: SQLite Maicie privée, fichier distinct de `bridget.db`  
**Testing**: `cargo test --workspace` ; tests unitaires, intégration et contrat
à fixtures sous `plugins/maicie/tests/`  
**Target Platform**: macOS et Linux locaux, même modèle de socket Unix que Bridget  
**Project Type**: binaire compagnon local dans un workspace Cargo  
**Performance Goals**: une consultation d'état locale p95 < 250 ms sur une
fixture de 100 objectifs ; aucune boucle de polling non bornée  
**Constraints**: disponibilité Maicie indépendante ; zéro base partagée ;
zéro déduction de blocage depuis texte ; outbox idempotente ; session 008 pour
les événements ACP ; session 009 pour les lancements ; activation de profil
approuvée par l'humain ; support multi-fournisseur au niveau de Bridget/ACP  
**Scale/Scope**: équipe locale de quelques dizaines de profils ; un objectif
peut déléguer à plusieurs agents mais le MVP n'ordonnance pas un graphe

## Constitution Check

| Gate | État | Justification |
|---|---|---|
| Minimalisme (XIX) | PASS | compagnon unique, CLI/JSON avant interfaces ; aucun scheduler/DAG |
| Responsabilité future (XX) | PASS | états, sources, corrélations et limites documentés ; pas d'heuristique cachée |
| État durable | PASS | SQLite privée par propriétaire ; réconciliation par contrat |
| Sécurité | PASS | permission ACP historique auto-décidée ; approbation mono-usage SpawnOrder |
| Observabilité | PASS | transitions et décisions corrélées et journalisées |
| Dépendances | PASS | réutilisation du workspace, aucune dépendance prévue |
| Complexité | PASS | index SQLite sur objectifs ouverts/message_id ; aucune horloge Maicie |

Réévaluation post-design : PASS. Le seul couplage conservé est le contrat
public Bridget, justifié par le besoin réel de déléguer et de connaître la
disponibilité.

## Project Structure

### Documentation

```text
specs/011-maicie-orchestration/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── contracts/maicie-cli-et-frontiere-bridget.md
├── quickstart.md
├── reuse-audit.md
├── tasks.md
├── analyze.md
└── checklists/requirements.md
```

### Source cible

```text
plugins/maicie/
├── Cargo.toml                 # futur membre du workspace
├── src/
│   ├── main.rs                # daemon Maicie + CLI
│   ├── app.rs                 # cas d'usage d'orchestration
│   ├── domain.rs              # entités et invariants
│   ├── config.rs              # configuration validée
│   ├── store.rs               # SQLite Maicie, aucune table Bridget
│   ├── outbox.rs               # reprise idempotente avant/après I/O Bridget
│   ├── bridget_client.rs      # seul adaptateur vers le contrat public Bridget
│   ├── reconcile.rs            # lookup/replay d'outboxes et issues Bridget
│   ├── runtime.rs             # normalisation factuelle ACP
│   ├── profiles.rs            # profils et demandes SpawnOrder, jamais un spawn OS
│   └── telemetry.rs           # journal corrélé sans contenu par défaut
├── tests/
│   ├── contract/
│   ├── integration/
│   └── fixtures/
└── README.md                  # frontière de propriété déjà créée
```

**Structure Decision** : le sous-répertoire `plugins/maicie/` est obligatoire.
Le membre Cargo reste dans le workspace, mais aucun crate Bridget ne dépend de
Maicie. Le répertoire racine `plugins/` est une convention de propriété, pas
un mécanisme dynamique de chargement.

## Design par phase

### Phase 0 — Contrat et exécutable minimum

1. Ajouter `plugins/maicie` au workspace, avec un binaire/daemon séparé qui
   s'enregistre sous l'identité stable `maicie` auprès de Bridget.
2. Définir `BridgetClient` comme unique frontière I/O : annuaire, envoi suivi
   idempotent à identifiant client, demandes, annulation, négociation de
   version/capacités et abonnement session 008 lorsque disponible.
3. Ajouter configuration validée : socket Bridget, chemin SQLite Maicie,
   classes de durée et profils. Ne pas introduire de config globale opaque.
4. Publier et tester le contrat JSON de la section `contracts/`, y compris Ack
   perdu, déduplication et incompatibilité producteur/client.

### Phase 1 — MVP P1 : objectif volontaire et réconciliation

1. Implémenter `ObjectifCoordonné`/`Délégation`, les deux outboxes et snapshot
   de transport avec migrations idempotentes et index sur état ouvert/message_id.
2. Créer `maicie delegate`, `status`, `add-participant`, `remove-participant`,
   `summarize` et `close`, avec sortie textuelle et JSON unique.
3. Résoudre seulement une cible explicitement demandée ou l'égalité de tags
   déclarés : un seul compatible disponible, sinon proposition sans choix.
4. Écrire l'enveloppe filaire complète de l'outbox et le `message_id` avant
   l'envoi ; prepared/outcome_unknown font lookup puis replay exact. Le
   tombstone Bridget couvre l'horizon de retry ou l'issue est idempotency_expired.
5. Au redémarrage, relire seulement SQLite Maicie puis appeler Bridget pour
   réconcilier l'outbox et les délégations ; ne jamais créer un nouvel id.
6. Vérifier explicitement que le trafic Bridget direct ne déclenche aucune
   mutation Maicie, sauf confirmation explicite d'une intention adressée à
   Maicie elle-même.

### Phase 2 — P2 : observations ACP et délais affichés

1. Consommer les faits par le contrat public `Subscribe` de session 008,
   avec `subscription_id`, séquence, `Gap`, `End` et reprise, sans lire journal
   interne ni parser une sortie terminal.
2. Afficher séparément disponibilité, snapshot transport, activité runtime,
   permission auto-décidée, fraîcheur et flux incomplet ; aucune attente humaine
   fictive ni rapport sémantique n'est ajouté.
3. Traduire `courte`/`normale`/`longue` vers les timeouts Bridget configurés.
4. Bridget reste l'unique horloge active : Maicie ne déclenche rien à une
   échéance locale ; elle réagit seulement à un événement Bridget ou une
   consultation explicite.

### Phase 3 — P3 : profils et consentement

1. Charger les profils nommés avec capacités déclarées.
2. Résoudre les candidats présents d'abord, puis proposer un profil inactif.
3. Persister une approbation locale mono-usage (hash profil/contexte,
   paramètres, expiration, acteur) et `ActivationOutbox(command_id)` dans la
   même transaction ; lookup/replay le SpawnOrder public 009 après revalidation.
4. Ne jamais appeler une API de processus, sélectionner implicitement ou lancer
   automatiquement dans cette feature.

### Phase 4 — Finition

1. Tests de régression de Bridget et du plugin, contrats et scénarios quickstart.
2. Logs structurés corrélés par objectif/délégation, sans corps de messages par
   défaut ; métriques locales minimales si l'infrastructure projet les supporte.
3. Documentation de démarrage, limites, arrêt propre et suppression de données.

## Sécurité et opérations

- Le socket Bridget est une frontière locale de confiance existante, pas une
  frontière d'autorisation ; Maicie doit donc borner et valider toute entrée.
- Les permissions ACP sont déjà auto-décidées par le registre 007 ; Maicie les
  montre comme historique, sans intercepter le cycle de tour.
- Les profils ne stockent pas de secrets. Maicie émet un `SpawnOrder` public
  après revalidation ; elle ne possède aucun processus enfant.
- Les écritures SQLite Maicie sont transactionnelles : création de décision et
  transition associée sont atomiques. La coordination avec Bridget reste une
  saga réconciliable, pas une transaction distribuée fictive.

## Complexité Tracking

| Choix | Coût | Garde-fou |
|---|---|---|
| Deux processus locaux | contrat supplémentaire | limite de responsabilité claire, aucun état partagé |
| Réconciliation au redémarrage | lecture des objectifs ouverts O(n) | n borné par l'équipe locale, index sur état/corrélation |
| Profils séparés des instances | une entité de plus | besoin réel : une personnalité peut dormir ou être remplacée |
| Outbox durable | une table de plus | évite le doublon après Ack perdu, besoin prouvé par crash |

## Hors périmètre explicite

- GUI, TUI et synchronisation visuelle multi-canal ;
- DSH, T3 Code et toute dépendance d'interface externe ;
- DAG, planificateur, cron/ticks, worktrees, gestion de projet ou exécution de
  code dans les agents ;
- apprentissage automatique de durée, auto-réveil, délégation autonome,
  compréhension de texte libre, synthèse par LLM ou rapport sémantique MCP/ACP ;
- gestion multi-utilisateur, authentification distante et sécurité inter-tenant.
