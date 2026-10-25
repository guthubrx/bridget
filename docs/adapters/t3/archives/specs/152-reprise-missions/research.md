# Archives techniques de l’adaptateur T3

Document d’origine : [specs/152-reprise-missions/research.md](../../../../../../specs/152-reprise-missions/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-11. R1 à R26 conservent la recherche historique du checkout principal
`/Users/moi/Nextcloud/10.Scripts/64.bridget` (base `1f2c661d`), de la contre-revue
Codex de la 150 (`specs/150-socle-delegation/adversarial-review-codex.md`,
objections 1 à 4 et 7 de la ronde 1) et de T3 (`RestartContinuation.ts`,
`EffectWorker.ts`, via `research-t3-bridget.md` A3 et A4). Les lignes citées sont
celles de cette base. L'audit frais du worktree 152 est en R27 ; il prime pour
les références au code livré par 150 et 151. Les décisions R22 à R26 priment
sur les propositions remplacées des rondes précédentes.
`N` = `crates/bridget-daemon/src/daemon/native_delegation.rs`,
`D` = `crates/bridget-daemon/src/daemon.rs`, `E` = `execution_store.rs`.

<a id="extrait-2"></a>

## Extrait 2

- **Constat** : `cancel_tree` collecte les noms (tâches, liens de flotte) puis
  appelle `prepare_managed_stop` et `await_managed_stop` pour chacun, en ordre
  inverse (`N:1380-1486`). `prepare_managed_stop` a des effets de bord :
  `invalidate_for_stop`, `mark_stopped`, ajout à `managed_terminal_instances`
  (`D:9255-9318`). Il n'existe donc pas de vérification préalable. Un membre
  `NotManaged` laisse la moitié de l'arbre arrêtée. Le nouvel essai rejoue tout
  (`NotFound` compte comme succès, donc sans danger mais sans économie).
- **Décision** : extraire de `prepare_managed_stop` une classification pure
  `classify_stop(st, name)` : supervisé, marqueur, déjà arrêté, introuvable, non
  géré. `prepare_managed_stop` l'utilise. `cancel_tree` classe tout l'arbre avant
  le premier arrêt (FR-018, FR-019). Les succès sont écrits dans
  `cancel_stopped` de la tâche racine (FR-020).
- **Raison** : T3 vérifie d'abord récursivement puis arrête. La classification
  existe déjà dans le code, fondue avec l'effet. On la sépare, on ne la réécrit pas.
- **Alternatives** : annulation sans vérification mais avec journal (rejeté :
  laisse un arbre à moitié mort) ; verrou global d'annulation (rejeté : non
  demandé).

<a id="extrait-3"></a>

## Extrait 3

- **Décision** : la suspension est un fait de la tâche (`suspended_at`), pas un
  état. Les champs vivent dans le payload JSON de la tâche avec `#[serde(default)]`
  (`delegation.rs:15-58`). Aucune migration.
- **Raison** : un nouvel état changerait la vue Lineage 149 et `pending()`
  (`delegation.rs:169`).
- **Coût accepté** : la vue Lineage T3 montre `working` pendant la pause
  (question 3).

<a id="extrait-4"></a>

## Extrait 4

T3 rejoue les événements persistés au démarrage et reprend le tour racine
interrompu. Un enfant non repris « doit encore un résultat ». Bridget reprend la
mission avec le message d'origine et une borne de reprises. Bridget ajoute le gel
des horloges, que T3 n'a pas.

<a id="extrait-5"></a>

## Extrait 5

Sur décision du principal, le quickstart est aligné sur H6 et le contrat de
livraison de production : contrôle réel après activation coordonnée sans
interrompre les autres fils T3. L’observation de phase 8 reste à faire.
Aucune relance actuelle, aucune nouvelle approbation, aucun redémarrage T3.
Git/disque/runtime devront être prouvés séparément. T032 conserve les refus
de droits/identités invalides. T033/T035 restent en mémoire sans publication.
