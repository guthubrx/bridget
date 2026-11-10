# Lot 11 - Modules nommés par rôle (T119-T124)

But : 18 modules portent un numéro de session dans leur nom (`sentinel244.rs`). Ils prennent un nom qui dit leur rôle. Seuls les **noms de fichiers et de modules** changent. Table et procédure : `releves.md` section 9 (`sed -n '109,114p'`).
Branche : `session-260-lot11-modules`. Worktree : `.worktrees/260-lot11-modules`. Code exécutable : noms seulement. « Activation en attente » dans le journal (R8).
**Porte D10 (bloquante pour ce lot)** : même gêne que D9 (conflits avec toute branche ouverte).
Interdits propres au lot : ne pas modifier `specs/`, `docs/rapports`, ni aucune chaîne littérale (R6.3). Les noms de **fonctions** numérotés ne changent pas.

## T119 - Porte D10, dépôt calme, prérequis

1. D10 `en attente` : **le lot ne démarre pas.** Écrire « différé (porte D10) » pour T119 à T124 et passer au lot suivant.
2. D10 `tranchée` : appliquer la réponse. Vérifier le calme comme au lot 10 (T114, mêmes quatre commandes ; écrire `validation/lot11-calme.txt`).
3. Prérequis : lot 10 (format) intégré. Sinon : « différé (lot 10 non intégré) » (renommer avant de formater ferait entrer en conflit les deux lots).
4. Préparer la ligne de base de noms transformée (voir T124) : les identifiants de test contiennent le chemin du module, donc les noms de tests **changent**. Préparer maintenant le fichier `validation/lot11-renommages.txt`, une ligne `ancien nouveau` par module de la table ci-dessous (noms de module sans `.rs` ni dossier), par exemple `sentinel244 sentinel`.

## Table des 18 renommages

Chemins relatifs à `crates/bridget-daemon/src/` sauf mention. Le dossier ne change pas.

| Ancien | Nouveau |
|--------|---------|
| `agent_tracking_sync230.rs` | `agent_tracking_sync.rs` |
| `daemon/agent_tracking230.rs` | `daemon/agent_tracking.rs` |
| `daemon/late_worker_prepares240.rs` | `daemon/late_worker_prepares.rs` |
| `daemon/native_admission_guidance211.rs` | `daemon/native_admission_guidance.rs` |
| `daemon/native_provider_catalogue220.rs` | `daemon/native_provider_catalogue.rs` |
| `daemon/native_quota_questions246.rs` | `daemon/native_quota_questions.rs` |
| `daemon/native_workspace_preparation210.rs` | `daemon/native_workspace_preparation.rs` |
| `daemon/specialists222.rs` | `daemon/specialist_requests.rs` |
| `delivery_spool244.rs` | `delivery_spool.rs` |
| `mcp_directory211.rs` | `mcp_directory.rs` |
| `provider_catalogue220.rs` | `provider_catalogue.rs` |
| `remote_worker_capacity234.rs` | `remote_worker_capacity.rs` |
| `remote_worker_proof198.rs` | `remote_worker_proof.rs` |
| `reply_outbox244.rs` | `reply_outbox.rs` |
| `sentinel244.rs` | `sentinel.rs` |
| `t3code_agent_tracking230.rs` | `t3code_agent_tracking.rs` |
| `workspace_failure210.rs` | `workspace_failure.rs` |
| `crates/bridget-transport/src/native_hook_profile171.rs` | `native_hook_profile.rs` |

## Méthode pour un module (T120 à T122)

Soit `ANCIEN` le nom du module (fichier sans `.rs`) et `NOUVEAU` le nouveau nom.

1. **Collision** : `test ! -e <nouveau chemin>` et `rg -n '\b(mod|use)\b.*\bNOUVEAU\b' crates --glob '*.rs'`. Une collision (un module ou un import déjà nommé `NOUVEAU`) : arrêter ce module, le noter. Ne pas créer d'alias.
2. Déplacer : `git mv <ancien chemin> <nouveau chemin>`.
3. Remplacer l'identifiant dans tout le code Rust (`perl`, car le `sed` de macOS n'a pas `\b`) :
   ```sh
   perl -pi -e 's/\bANCIEN\b(?!-)/NOUVEAU/g' $(rg -l '\bANCIEN\b' crates --glob '*.rs')
   ```
   Le `(?!-)` protège les chaînes comme `"sentinel244-{}"` et `"spool244-{}"` (préfixes d'identifiants durables, **à ne jamais changer**).
4. **Contrôle des chaînes** (R6.3) :
   ```sh
   git diff -U0 | rg '^\+.*"' | rg -v '#\[path'
   ```
   Attendu : aucune ligne. Une ligne qui contient un littéral : annuler la substitution sur cette ligne (remettre l'ancien texte) et noter. Les lignes `#[path = "..."]` qui nomment le fichier **doivent** changer : elles pointent le nouveau fichier.
5. `C-CHECK-LEGER` : code 0. Erreur `E0255` (nom défini deux fois) : arrêter ce module, annuler (`git checkout -- . && git mv <nouveau> <ancien>`), noter.
6. Commit par module : `refactor(daemon): Renommer le module <ANCIEN> en <NOUVEAU>`.

Cas de `specialists222.rs` : le nouveau nom est `specialist_requests` (le mot `specialists` existe déjà dans `agent_profile/specialists.rs`).
Fichiers de tests associés : un fichier `<ANCIEN>_tests.rs` ou `<ANCIEN>_<x>_tests.rs` qui porte lui aussi le numéro garde son nom (hors périmètre). Seuls les 18 fichiers de la table sont renommés.

| Tâche | Modules (lignes de la table, dans l'ordre) |
|-------|---------------------------------------------|
| T120 | 1 à 6 |
| T121 | 7 à 12 |
| T122 | 13 à 18 |

Après T120 et T122 : `C-CHECK` complet et `C-CRATE` des deux crates.

## T123 - Contrôle des restes

```sh
for a in $(awk '{print $1}' "$V/lot11-renommages.txt"); do echo "== $a"; rg -n "\b$a\b" crates --glob '*.rs' | head -3; done
```

Attendu : aucune ligne pour les 18 anciens noms, sauf les chaînes protégées (`sentinel244-`, `spool244-`) et des noms de fonctions numérotés (hors périmètre). Écrire le résultat dans `validation/lot11-restes.txt`. Chercher aussi dans les scripts et la configuration : `rg -n '<ancien nom>' scripts Makefile .github Cargo.toml crates/*/Cargo.toml` : aucune ligne attendue ; une ligne trouvée est corrigée seulement si elle nomme le **fichier** Rust.

## T124 - Porte du lot

1. `C-TESTS` (`<nom>` = `lot11`), puis `C-ECHECS` avec la base **transformée** : les noms d'échecs de la base contiennent l'ancien chemin. Transformer d'abord :
   ```sh
   cp "$V/base-echecs.txt" "$V/base-echecs-lot11.txt"
   while read a n; do perl -pi -e "s/\b$a\b/$n/g" "$V/base-echecs-lot11.txt"; done < "$V/lot11-renommages.txt"
   sort -o "$V/base-echecs-lot11.txt" "$V/base-echecs-lot11.txt"
   diff "$V/base-echecs-lot11.txt" "$V/lot11-echecs.txt"
   ```
   Attendu : aucune ligne.
2. `C-NOMS` avec la même transformation sur `base-noms.txt` (`base-noms-lot11.txt`) : aucune différence.
3. Mettre à jour `base-echecs.txt` et `base-noms.txt` avec les versions transformées, puis le dire dans le journal (nouvelle ligne de base).
4. Bilan : modules renommés, modules arrêtés, « activation en attente ». Fin de lot (`quickstart.md`).

## Arrêts propres à ce lot

- Une substitution modifie un littéral durable : annuler la ligne, ne jamais la laisser (R6.3).
- Conflit Git sur un fichier renommé à l'intégration : arrêt du lot (R4).
- Un script de la machine (`scripts/`, unités de service) cite un chemin de fichier Rust renommé : ne pas modifier, noter (R6.8).
