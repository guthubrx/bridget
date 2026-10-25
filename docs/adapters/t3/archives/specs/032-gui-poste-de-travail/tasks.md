# Archives techniques de l’adaptateur T3

Document d’origine : [specs/032-gui-poste-de-travail/tasks.md](../../../../../../specs/032-gui-poste-de-travail/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**Fichier** : `crates/bridget-transport/src/codex_app_server.rs` · **Contrat** : C3
**Référence MIT** : `~/11.Repositories/t3code/packages/effect-codex-app-server`

<a id="extrait-2"></a>

## Extrait 2

- [ ] T3.1 **Lire la référence avant d'inventer.**
- [ ] T3.2 Brancher `item/reasoning/summaryTextDelta`, `summaryPartAdded`,
      `textDelta` → `reasoning`.
- [ ] T3.3 Brancher `item/commandExecution/outputDelta` → acte `command`.
- [ ] T3.4 Brancher `item/fileChange/patchUpdated` → acte `file`.
- [ ] T3.5 Brancher `item/plan/delta` → acte `plan`.
- [ ] T3.6 `item/*/requestApproval` → acte `approval` — **affiché, jamais
      validable**.
- [ ] T3.7 Oracle : flux avec raisonnement → `available: true` + résumé.
- [ ] T3.8 Oracle : flux **sans** raisonnement → `available: false`.
      *Contrôle positif : l'instrument voit la présence avant de juger
      l'absence.*
- [ ] T3.9 Mutant : ignorer `item/reasoning/*` → **nommer le test mort**.

<a id="extrait-3"></a>

## Extrait 3

**Fichier** : `crates/bridget-transport/src/acp.rs` · **Contrat** : C3
**Référence MIT** : `~/11.Repositories/t3code/packages/effect-acp`

<a id="extrait-4"></a>

## Extrait 4

- [ ] T5.1 Créer `assets/ui/` — `index.html`, `app.js`, `theme.css`.
- [ ] T5.2 **Retirer les 153 lignes embarquées** de `ui.rs` ; servir depuis
      `assets/`. *(Pas de duplication : l'embarqué disparaît.)*
- [ ] T5.3 Liste d'agents : nom · **état** · **machine** · dernier message ·
      heure · pastille.
      **Ce sont des AGENTS, pas des conversations** — d'où état et machine.
- [ ] T5.4 Tri par dernier message ; agents arrêtés en section **masqués**.
- [ ] T5.5 Fil : bulles agent à gauche, utilisateur à droite ; séparateurs de
      date ; événements système **en gris, centrés**.
- [ ] T5.6 Consommer `/v1/snapshot` puis s'abonner à `/v1/watch`
      — **abonnement AVANT snapshot**, comme le fait la route.
- [ ] T5.7 Charte, **relevée sur T3 Code (MIT)** :
      `apps/web/src/index.css` — `--color-border-subtle`,
      `--chat-composer-glass-surface`, `--code-background`,
      `--app-chrome-background`.
- [ ] T5.8 Contrôle positif de charte : **aucune bordure**, **un seul accent
      coloré**, **deux niveaux de gris de texte**.
