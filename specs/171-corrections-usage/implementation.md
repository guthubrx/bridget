# Journal d'implémentation - Corrections d'usage 171

## Métadonnées
- **Session** : 171-corrections-usage (correction, validation automatique)
- **Branche** : session-171-corrections-usage
- **Démarré** : 2026-10-17
- **Base** : main 73b03a1f + correctifs 171 jusque-là actifs mais seulement dans
  `/Users/moi/.cache/bridget-delivery171/own-usage171/worktree-config171-r2/source`

## Défauts traités

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/171-corrections-usage/implementation.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/171-corrections-usage/implementation.md#extrait-2).

### 4. Tests du lot 171 rouges, dont régressions 149/162
- **Constat** (compilations isolées par copie, cache Cargo séparé) : main 26/26,
  lot 171 13 échecs dont 4 anciens tests.
- **Causes et corrections code** :
  - hook sous un HOME via lien : comparaison chemin résolu/racine non résolue ;
  - capture depuis un cwd via lien (~/10.Scripts) : garde G054 ;
  - `managed-settings.json` macOS en root:admin 0644 refusé (G063).
- **Tests alignés sur le contrat 171** : révision v2, absence de
  `--permission-prompts` (Claude 2.1.251), preuve parent sans transplantation.
- **Ignorés avec motif** : 5 recettes à données privées réelles, 5 tests de la
  conception de transplantation remplacée par la configuration de destination.

### 4 bis. Régressions révélées par la comparaison avec main (suite complète Mac)
- main 2825 PASS/42 FAIL ; lot 171 initial 2838 PASS/56 FAIL, même environnement.
- **Worktree disparu avant lancement** : la tâche restait `starting`. Le correctif
  « lancement en vol » sautait tout contrôle ; un `stat` du cwd est rétabli.
- **Instrumentation `bridget_catalogue_observation171`** écrite sur stderr à
  chaque appel CLI et côté maître : retirée (cassait build-id et lineage CLI).
- **Tests alignés sur des contrats 171 voulus** : `extensions.worktreeConfig`
  admis, refus d'outil par hook remis avec mention, code
  `remote_start_failed:<cause>`, délai 154 sous `cfg(test)` à 20 s.
- **Non modifié** : `spec168_permis_reste_detenu_apres_timeout_demandeur_jusqu_au_drain_reel`
  échoue aussi sur main lancé seul (démarrage de `/usr/bin/python3`).
- **Course entre `git worktree add` simultanés** dans un même dépôt
  (`commondir` absent) : extraction sérialisée par dépôt.
- **Bancs alignés** : contexte ancien 152 en révisions v1, faux fournisseur e2e
  sans `--permission-prompts`, réglage de délai socket tolérant (EINVAL Darwin).
- Bancs 152/154/157 : à lancer avec un TMPDIR résolu (`/private/tmp/b.XXXX`) ;
  avec `/tmp`, main échoue aussi sur 16 d'entre eux.

### 5. Tâche c3b80e27 bloquée en annulation
- **Cause** : enveloppe tuée avant sa preuve de drainage ; groupe absent mais
  commande `unknown` pour toujours.
- **Correction** : preuve alternative si le sous-arbre cgroup v2 du service ne
  contient que le worker et qu'aucun enfant n'est suivi.
- **Vérification** : réelle après activation (cgroup Cartae relevé : 2819215 seul).

### 6. Correctifs hors Git
- Copie privée intégrée dans cette branche puis fusionnée dans main.
