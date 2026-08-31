# Reuse audit - SPEC-076

**Date**: 2026-08-30
**Tête analysée**: aba60f0
**Statut**: PASS
**Portée**: documentation SPEC-076 uniquement, aucun code modifié.

## Composants et décisions

| Besoin proposé | Existant | Preuve | Décision |
|---|---|---|---|
| Snapshot et flux UI | UiSnapshotV1, watch et routes relay | crates/bridget-daemon/src/ui.rs:526, :798-903, :1442-1524 | ÉTENDRE. Une projection projets rejoint le snapshot existant. |
| Application UI | app.js et thème existants | crates/bridget-daemon/assets/ui/app.js | ÉTENDRE. Barre projets et conversation réutilisent le rendu actuel. |
| Identité projet | ProjectIdentity Maicie | plugins/maicie/src/app.rs:108-235, plugins/maicie/src/store.rs:552-754 | ÉTENDRE. Maicie reste propriétaire; une transition typée disabled vers active est nécessaire, car elle n'existe pas encore. |
| Liaison projet | ProjectBinding et projection | crates/bridget-daemon/src/store.rs:267-307, :847-868 | RÉUTILISER. Aucun registre UI. |
| Audit projet | ProjectAuditEvent | crates/bridget-daemon/src/store.rs:315-389, :1051-1085 | RÉUTILISER. Historique lisible sans second journal. |
| Mutations projet | Saga CLI Maicie et client ProjectRegistry | plugins/maicie/src/main.rs:659-825; plugins/maicie/src/bridget_client.rs:1174-1277 | ÉTENDRE par entrée Maicie UI typée. Pas d'accès direct store, pas de shell libre. |
| Racines autorisées | ProjectRootPolicy | crates/bridget-daemon/src/project_policy.rs:16-123 | ÉTENDRE. Validation et refus actuels restent obligatoires. |
| Registre fournisseur | AgentDefinition, définition résolue, digest | crates/bridget-daemon/src/registry.rs:44-86, :158-215 | RÉUTILISER. La configuration UI ne déduit pas le fournisseur. |
| Permissions et profils | ProjectProfile | SPEC-067, documents de programme | CONSOMMER après livraison. Aucun clone ni approbation UI. |
| Cycle coordinateur | stop, relaunch, decommission | SPEC-075, worktree actif | CONSOMMER après stabilisation. |
| Connexion Desktop | profil SSH et panneaux Desktop | SPEC-074, worktree actif | NE PAS TOUCHER. SPEC-076 reste serveur UI et contrat consommable. |
| Relais UI local | UiRelayConfig::loopback et token UI | crates/bridget-daemon/src/ui.rs:213-220, :402-426, :511-515 | RÉUTILISER. T003 doit attester l'UID, le tunnel Desktop et l'absence d'écoute réseau hors loopback. |

## Surfaces nouvelles légitimes

| Surface | Recherche effectuée | Verdict |
|---|---|---|
| ProjectOnboarding | Recherche sur workspace : absent hors SPEC-076 | CRÉER seulement comme projection durable, jamais registre métier. |
| CoordinatorConfiguration | Recherche sur workspace : absent hors SPEC-076 | CRÉER comme instantané sans secret, lié aux autorités 066, 067 et 072. |
| DiscoveryRun | Recherche sur workspace : absent hors SPEC-076 | CRÉER car aucun contrat existant ne porte un créneau de découverte lecture seule. |
| Routes UI projets | Recherche sur slash v1 projects : absente | CRÉER sous le relais UI existant, avec version, schéma fermé et command_id. |
| Reload de politique | Politique actuelle chargée une fois | ÉTENDRE ProjectRootPolicy; aucune préférence navigateur indépendante. |
| Réactivation projet | Aucune transition disabled vers active actuelle | plugins/maicie/src/domain.rs:109-114; crates/bridget-daemon/src/store.rs:659-676 | ÉTENDRE les autorités existantes par opération typée et auditée; ne jamais recréer l'identité. |

## Arbitrages déjà tranchés

| Sujet | Conflit | Arbitrage | Preuve |
|---|---|---|---|
| Mutation UI projet | SPEC-065 limitait sa tranche à la CLI Maicie | Interface locale authentifiée admise seulement par opérations typées, Maicie restant autorité | décision utilisateur et ADR-019 |
| Nom fournisseur | Outil agent peut différer de upstream et modèle | Config complète distincte : outil, upstream, modèle, effort, permissions | décision utilisateur et SPEC-072 |
| Mémoire initiale | Fichier dédié versus conversation | Aucun fichier Bridget créé; rapport dans la conversation durable | décision utilisateur et FR-024 à FR-028 |
| Suppression | Retrait UI versus suppression fichier | Disable et archivage seulement, jamais suppression dossier ou histoire | décision utilisateur et SPEC-065 |

## Duplications évidentes

Aucune duplication évidente non arbitrée.

- Un nouveau store projet UI serait un doublon de ProjectIdentity ou
  ProjectBinding : interdit.
- Un nouveau canal de messages serait un doublon du relais UI : interdit.
- Une exécution shell UI serait un doublon dangereux de la CLI : interdite.
- Une approbation de profil UI serait un doublon interdit de SPEC-067.

## Concurrence observée

| Worktree | Fichiers actuellement modifiés | Mesure |
|---|---|---|
| session-074-bridget-desktop | cli.rs, tests/ui_relay_test.rs et application Desktop | SPEC-076 ne touche pas ce worktree. Rebase et audit frais avant code. |
| session-075-cycle-vie-agents | sources cycle de vie, UI, transport et documents en cours | SPEC-076 ne touche pas ce worktree. Consommer seulement contrat intégré. |
| session-076-interface-projets-coordinateur | documents SPEC-076 et ADR-019 | Aucun fichier source modifié. |

## Gate avant tasks

- [x] Toutes les autorités métier et techniques existantes sont identifiées.
- [x] Les projections UI, saga Maicie, politique et registre fournisseur sont
      affectés à une stratégie de réutilisation explicite.
- [x] Les surfaces nouvelles ont une justification et aucun équivalent proche.
- [x] Les arbitrages utilisateur nécessaires sont consignés.
- [x] Aucun développement concurrent n'est modifié par cette SPEC documentaire.
- [x] Les risques de conflit futur 074 et 075 sont consignés avec exigence de
      worktree frais et rejeu d'audit avant code.

## Gate avant implémentation

Le présent audit documente la tête aba60f0. Il doit être rejoué sur une main
propre qui contient SPEC-066, SPEC-067 et SPEC-075. Il ne dispense pas des
gates listés dans plan.md.


## Rejeu sur main 20e50ac - 2026-08-31

**Statut**: PASS - aucune duplication non arbitrée.

| Besoin 076 | Existant confirmé | Décision |
|---|---|---|
| Prévisualisation dossier | aucune surface équivalente ; validation canonique dans ProjectRootPolicy | CRÉER project_workspace.rs, sans store ni mutation |
| Diagnostic Git | commandes Git fixes déjà employées par wrapper.rs et lifecycle.rs | RÉUTILISER le même principe, sans shell et après validation de racine |
| Politique de racines | ProjectRootPolicy et DaemonState | ÉTENDRE : génération, écriture atomique et reload de la dernière version valide |
| Vue interface | UiSnapshotV1, relais loopback et token | ÉTENDRE ; ne pas créer de second store navigateur |
| Liaisons existantes | ProjectAdminRequest et ProjectBindingProjection | RÉUTILISER pour lectures, rebind et retrait |
| Identité métier | saga ProjectRegistration Maicie | ÉTENDRE par intention UI typée ; aucun accès direct à la base Maicie |
| Cycle coordinateur | lifecycle.rs intégré par SPEC-075 | CONSOMMER, sans second superviseur |

Recherche exécutée : UiSnapshotV1, ProjectRootPolicy, ProjectAdminRequest,
ProjectRegistrationRequest, lifecycle.rs, wrapper.rs et app.js. Aucun équivalent
de project_workspace.rs n’existait. Les créations 076 restent limitées à une
prévisualisation pure et aux extensions identifiées ci-dessus.
