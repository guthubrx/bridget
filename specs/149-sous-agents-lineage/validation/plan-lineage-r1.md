# Revue G-L r1 — plan lineage session149

Revue : GLM 5.3 Flash. Date : 2026-10-10. Périmètre : G-L (Lineage, lectures
natives, journal, filtre du pont). G-P (permissions) reste hors de ce rapport.

Sources lues : AGENTS.md du worktree, constitution globale 1.9.0, skill bridget
(`~/.codex/skills/bridget/SKILL.md`, exigence d'écriture récente prioritaire sur
le contrat 148), spec.md, plan.md, tasks.md, data-model.md,
contracts/lineage.md, contracts/permissions.md, test-strategy.md r2,
checklists/requirements.md, ADR 149. Code lu : worktree Bridget base 148
`6807c22b` (code inchangé) et worktree T3 base 148 `33f6d04e11` (code inchangé).

## Verdict : REQUEST_CHANGES

Les contrats Lineage et le plan sont cohérents entre eux, réalisables sur le
code existant, et le volet G-L ne dépend pas de G-P. Deux corrections bloquent
l'APPROVE : la recette desktop de T040 contredit une règle absolue de la skill,
et la table de mapping des tâches oublie FR019.

## Findings

### F1 — HIGH : T040 exige une seconde application T3, interdite par la skill

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

### F2 — MEDIUM : FR019 absente de la table de mapping des tâches

- Fichier : `tasks.md:87-106`.
- Reproducteur logique : la spec définit FR001–FR019. La table n'a que 18
  lignes FR. FR019 (figer droits/modèle/définition avant premier effet, rejeu
  sans élargissement, changement de mode UI sans effet implicite) n'apparaît
  nulle part. La stratégie r2 la couvre pourtant (S149-01, S149-09, S149-24,
  S149-26 dans sa section 10).
- Correction : ajouter la ligne
  `| FR019 | T007,T010,T013,T015,T023,T027–T028,T039 |`.
  T007 publie le mode mutable par tour ; T010 fige la politique avant effet ;
  T013 applique la définition figée ; T015/T023 testent le rejeu figé ;
  T027/T028 tiennent la projection en lecture seule ; T039 prouve le retry après
  changement de droits.

### F3 — LOW : références obsolètes à 21 scénarios

- Fichiers : `plan.md:139`, `tasks.md:114`.
- Reproducteur : les deux textes citent « S149-01 à S149-21 ». La stratégie r2
  en compte 27 (S149-22 à S149-27 ajoutés en r2).
- Correction : remplacer par « S149-01 à S149-27 » ou par « la stratégie r2 ».
  Si F1 ajoute S149-28, écrire « S149-01 à S149-28 ».

### F4 — LOW : T029 vise le mauvais fichier pour l'arrêt

- Fichiers : `tasks.md:51` (T029) ; code T3 `Orchestrator.ts:474`
  (`case "thread.stop"`), `ThreadLifecycleService.ts:15-24`.
- Reproducteur : ThreadLifecycleService possède archive, unarchive, delete,
  update-metadata, set-runtime-mode, set-interaction-mode, set-model-selection.
  L'arrêt d'un fil est une commande `thread.stop` traitée dans Orchestrator.ts.
- Correction : viser `Orchestrator.ts` (geste `thread.stop`) et le service
  BridgetLineage pour l'arrêt enfant. ThreadLifecycleService reste pertinent
  pour FR019 : ses set-*-mode ne doivent jamais toucher un enfant en cours.

### F5 — INFO : préciser le domaine de séquence du watch 149

- Fichiers : `contracts/lineage.md:90` (« si leurs domaines diffèrent ») et
  `:151-152` ; code `protocol.rs:2219-2260` (`watch_ready_seq` exige ready seq0).
- Risque : en 147, le watch porte la séquence du magasin de vue de fil. En 149,
  le même format porterait la séquence de `native_delegation_projection_meta`.
  Le domaine est ici le même pour le flux et le snapshot, mais une implémentation
  par copie du watch 147 pourrait croiser les générations.
- Correction : une phrase dans lineage.md — « la séquence des événements watch
  est celle du magasin de tâches natives ; ready seq0 reste un marqueur, pas la
  séquence du magasin ». Aucun changement de schéma.

### F6 — INFO : audit des consommateurs `app_owned` à l'ajout de l'origine

- Fichiers : `packages/contracts/src/orchestrationV2.ts:668,1559,2334` (unions
  `origin`) ; `apps/server/src/orchestration-v2/Orchestrator.ts:1932,2091,2170,
  6576,6628,6762` (comparaisons directes `origin === "app_owned"`).
- Risque : ajouter `bridget_native` à l'union demande de vérifier chaque
  comparaison. Une reprise `app_owned` ne doit jamais prendre un fil natif.
- Correction : rien à changer dans les contrats — lineage.md:100-101 l'exige
  déjà. Consigner ce point comme garde de relecture pour T024/T026 (S149-16
  couvre déjà le négatif `activeTurnId` null et zéro reprise).

### F7 — LOW : fiche synthèse de la spec périmée

- Fichier : `spec.md:10-21` (« Tâches: 0/0 (0%) », `tasks.md: ✗`, `plan.md: ✗`).
- Reproducteur : tasks.md existe et compte 45 tâches à 0/45 ; plan.md existe.
- Correction : déjà couvert par T001 (clôture formelle de la spec, owner
  principal). Rappel pour T001 : mettre la fiche à 0/45 et cocher les fichiers.

## Réalisabilité vérifiée — preuves de code

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

Points de contrat vérifiés : première page fixe S et pages atomiques
(lineage.md:114-118, S149-27) ; curseur opaque lié root/génération/seq
(lineage.md:109-111) ; égalité génération/séquence avec `snapshot_changed` ;
seq entier sûr 0..9007199254740991 avec renouvellement de génération à la
borne (data-model.md:56-57) ; signal seulement après commit, lectures/refus/
ACK muets (lineage.md:155-160) ; résultat par fenêtres UTF8 (helpers
existants) ; reconnexion sans lecture cachée (idleTtlMs0, fermeture scoped,
pas de polling).

Risques stale génération / nouveaux réglages : la génération est persistée avec
le magasin (data-model.md:48-49) ; un changement impose un resync complet et
aucune page mélangée. La révision de permissions est un compteur séparé de la
séquence de projection ; les deux contrats gardent des noms distincts
(`revision` contre `generation`/`seq`). Aucun conflit trouvé.

Séparation G-L / G-P : les tâches T016–T035 (phases 2 et 3) n'éditent aucun
fichier de permission. Le filtre du pont (T022) appartient bien au volet G-L
annoncé du plan (`plan.md:155`). G-P fermée ne bloque pas G-L, comme écrit en
`plan.md:41,129`.

## Axes obligatoires

### Complexité (article XVIII)

Conforme. Pagination visée O(log N + P) par index root/parent (lineage.md:228).
Réconciliation T3 en O(n), une seule indexation (data-model.md, lineage.md:229).
`descendants_busy` est borné par un ensemble de visite (4096). Aucune double
boucle nouvelle prévue. Les helpers de frontière UTF8 existent et sont
réutilisés. Les annotations de complexité demandées au §18.3 seront à poser
dans le code (T016-T018, T025).

### Minimalisme & Frugalité (article XIX)

Conforme après exécution de la checklist. Aucune dépendance nouvelle
(`plan.md:22`). Aucun journal parallèle ni cache de corps (data-model.md:55-57,
plan invariant 11). Une seule ligne `native_delegation_projection_meta`. Le
marqueur facultatif `bridgetTaskRef` est justifié par les données historiques.
T032 crée un composant de journal « seulement si nécessaire ». Le service
BridgetLineage est unique et réutilise ProcessRunner et les gardes 147.

Potentiel minimalisme : ~15 lignes documentaires suppressibles à comportement
constant (deux références « S149-01 à S149-21 » périmées, champs périmés de la
fiche synthèse, formulation desktop de `plan.md:135`). Aucune ligne de code :
aucun code 149 n'existe encore. F1 ne supprime pas de ligne, elle réoriente la
preuve T040.

### Vertus LLM & Responsabilité Future (article XX)

Conforme. Le volume ajouté est borné : deux contrats TS nouveaux, un service de
domaine, des commandes CLI fermées, un marqueur. Chaque refus est nommé et
fermé ; les fallbacks sont interdits par écrit. Le code reste explicable :
chaque tâche nomme ses fichiers exacts. Les deux charges futures repérées
(audit des consommateurs `app_owned`, domaine de séquence du watch) sont
couverte par F5 et F6 et par les négatifs S149-16/S149-22. Aucune abstraction
sans usage réel détectée.

## Mapping 19 FR / 7 SC / 7 US vers 45 tâches

- 45 tâches comptées : phase0 3, phase1 12, phase2 8, phase3 12, phase4 7,
  phase5 3. « Progression : 0/45 » de tasks.md est correct.
- US1–US7 : présents dans les balises de tâches et la table. SC001–SC007 :
  couverts.
- FR001–FR018 : couverts. FR019 : absente — cf. F2, correction proposée.
- Stratégie r2 : 27 scénarios, tous les FR couverts selon sa section 10 ; le
  plan et tasks.md citent encore 21 scénarios — cf. F3. La recette UI de T040
  n'a pas encore de scénario — cf. F1, S149-28 proposé.

## Limites de cette revue

Revue documentaire et lecture de code ciblée. Aucun test, aucun build, aucun
modèle ni fournisseur lancé. Aucune fixture exécutée. La checklist reste non
cochée. Aucune tâche cochée. Le verdict porte sur le volet G-L ; G-P et le
contrat permissions sont hors périmètre de décision ici.
