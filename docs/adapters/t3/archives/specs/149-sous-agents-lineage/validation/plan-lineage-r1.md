# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/plan-lineage-r1.md](../../../../../../../specs/149-sous-agents-lineage/validation/plan-lineage-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Sources lues : AGENTS.md du worktree, constitution globale 1.9.0, skill bridget
(`~/.codex/skills/bridget/SKILL.md`, exigence d'écriture récente prioritaire sur
le contrat 148), spec.md, plan.md, tasks.md, data-model.md,
contracts/lineage.md, contracts/permissions.md, test-strategy.md r2,
checklists/requirements.md, ADR 149. Code lu : worktree Bridget base 148
`6807c22b` (code inchangé) et worktree T3 base 148 `33f6d04e11` (code inchangé).

<a id="extrait-2"></a>

## Extrait 2

- Fichiers : `tasks.md:65` (T040), `plan.md:135` (« La recette desktop ouvre
  Lineage et le journal »).
- Reproducteur logique : T040 demande « une application desktop de validation
  isolée ». Toute seconde instance T3 partage `~/.t3/userdata`. La skill bridget
  l'interdit sans exception : elle marque en échec les tours en cours de tous
  les fils (« Provider session did not survive a server restart ») et efface
  `server-runtime.json` en se fermant. Les PID de production 58468/57109/58394/
  58396 restent chargés et hors de portée (AGENTS.md). T040 contredit donc sa
  propre clause « sans toucher l'application chargée de production ».
- Correction :
  1. Remplacer T040 par une recette UI sur serveur de recette isolé : daemon
     Bridget fixture + serveur T3 fixture (port éphémère, base privée, même
     gabarit que l'interop R4), interface web ouverte dans un navigateur ou la
     preview T3. Vérifier : Lineage, journal réel enfant actif puis terminal,
     nested/statuts/modèle, absence de fil de premier niveau, arrêt, reconnexion.
  2. Ajouter dans T040 : « Cette preuve est une recette UI web. Elle ne valide
     pas la coque desktop native. Aucune seconde application T3 n'est lancée.
     Aucune relance de l'application de production. »
  3. Adapter `plan.md:135` : recette UI web au lieu de recette desktop.
  4. Ajouter le scénario correspondant dans test-strategy.md (S149-28 proposé)
     et sa ligne dans la section 10. Le fichier de preuve peut rester
     `desktop149.md` ou être renommé `ui-recipe149.md` ; le contenu doit
     distinguer recette UI web et recette native desktop.

<a id="extrait-3"></a>

## Extrait 3

- Fichiers : `tasks.md:51` (T029) ; code T3 `Orchestrator.ts:474`
  (`case "thread.stop"`), `ThreadLifecycleService.ts:15-24`.
- Reproducteur : ThreadLifecycleService possède archive, unarchive, delete,
  update-metadata, set-runtime-mode, set-interaction-mode, set-model-selection.
  L'arrêt d'un fil est une commande `thread.stop` traitée dans Orchestrator.ts.
- Correction : viser `Orchestrator.ts` (geste `thread.stop`) et le service
  BridgetLineage pour l'arrêt enfant. ThreadLifecycleService reste pertinent
  pour FR019 : ses set-*-mode ne doivent jamais toucher un enfant en cours.

<a id="extrait-4"></a>

## Extrait 4

| Point du contrat | Preuve dans le code | Conclusion |
|---|---|---|
| Garde humaine 147 et révocation ferme les flux | `daemon.rs:690,813-864` ; `revoke_identity_authorizations` termine chaque watch avec `BindingUnavailable` | Réutilisable tel quel pour lineage watch |
| Ready seq0 non coalescible | `protocol.rs:2227-2230` (deserialiseur dédié), `:2311-2314` (capacité `human_thread_watch_v1`) | Le contrat 149 réutilise un mécanisme testé ; capacités `human_lineage_*_v1` suivent le même chemin |
| Saga 148 : annulation et descendants | `native_delegation.rs:951` (`descendants_busy`, visite bornée 4096), `:1011` (`cancel_tree`), `:432-433` (cancelling) | lineage cancel humain (T021) est un habillage sous garde 147 |
| Résultat borné 256 KiB | `native_delegation.rs:625` (`262144`) | Fenêtrage UTF8 16 KiB possible ; helpers de frontière déjà présents (`attach.rs:1598-1606`) |
| Schéma tâches et index | `delegation.rs:51-58` : `native_delegations`, index uniques owner+request_id et mission | `parent_task_id`, index `child_agent_id`, `projection_meta` (generation/seq) s'ajoutent sans conflit ; aucun seq/generation existant, donc aucun état dupliqué |
| CLI humaine existante | `cli.rs:156-160` (`thread inspect`, `thread watch`) | Le namespace `lineage inspect/watch/cancel` suit le gabarit existant |
| Journal durable et follow | `attach.rs:205,424,491-624` (`SnapshotCaughtUp{through_seq}`, `caught_up`, relai) | Follow par AttachRelay réalisable ; réattestation avant ouverture et avant chaque publication reste à écrire (T020, couvert par le contrat lineage.md:143-146) |
| Filtre du pont avant montage | `t3code.rs:1180-1214` : boucle unique sur `snapshot.threads`, montage par `Link::open` | Le décodage `bridgetTaskRef` dans `t3code_contract*.rs` puis l'exclusion avant `Link::open` tombent au bon endroit ; fichiers T022 corrects |
| Preuve T3 revalidée hors verrou | `t3code_mcp.rs:63-96` (`validate_endpoint`, `attest_http` timeout 3 s), test `session_is_revalidated_and_refusals_are_closed:270` | T009 étend un mécanisme existant et testé |
| Registre privé T3 | `mcpSession.ts` (62 lignes, registre par threadId avec set/read/clear) | Extension pour les faits de politique réalisable ; le fichier est petit et dédié |
| Sections T3 réservées | `OrchestratorMcpService.ts:124,1777` (`sessionIdentity`) ; `tools.ts:65` (`BridgetSessionTool`) | Les seules sections ciblées par T008 existent |
| Origine T3 | `orchestrationV2.ts:668,1559,2334` (unions origin) | Ajout `bridget_native` réalisable ; cf. F6 |
| Colonne de gauche masque déjà subagent | `Sidebar.logic.ts:845` ; composeur lecture seule `ChatView.tsx:2159,4260` | L'affirmation du plan (`plan.md:102`) est vraie sur le code 148 |
| Fils virtuels/sous-agents T3 | `ProjectionStore.ts:267,759` (`subagents`, `subagent.updated`) ; `WireProjection.ts:106,187` ; `ThreadRelationshipsControl.tsx:158` ; `threadRelationships.ts:8,93-102` ; `SubagentProjection.test.ts` | Les primitives existent ; la commande interne via EventSink est réalisable sans effet fournisseur |
| Fichiers T3 ciblés existent | `ThreadLaunchService.ts`, `ThreadLifecycleService.ts`, `threadActivity.ts` (mobile), `threadWorkflows.ts`, `subagentRuntime.ts`, `contracts/index.ts` (export `bridget.ts`), adaptateurs Codex/Claude V2 | Aucun chemin de tâche ne pointe un fichier absent, sauf les nouveautés voulues (`bridgetPermissions.ts`, `BridgetLineage.ts`) |

<a id="extrait-5"></a>

## Extrait 5

Conforme. Pagination visée O(log N + P) par index root/parent (lineage.md:228).
Réconciliation T3 en O(n), une seule indexation (data-model.md, lineage.md:229).
`descendants_busy` est borné par un ensemble de visite (4096). Aucune double
boucle nouvelle prévue. Les helpers de frontière UTF8 existent et sont
réutilisés. Les annotations de complexité demandées au §18.3 seront à poser
dans le code (T016-T018, T025).
