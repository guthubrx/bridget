# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/identite-fichiers149-tests.md](../../../../../../../specs/149-sous-agents-lineage/validation/identite-fichiers149-tests.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-correctif-identite/crates/bridget-daemon/src/t3code.rs`, dans le bloc `#[cfg(test)]` seulement.

<a id="extrait-2"></a>

## Extrait 2

- 5 tests nouveaux, préfixe `spec149_state`, plus 3 petits helpers : environ 224 lignes après `rustfmt`. La cible « moins de 200 lignes » est dépassée de 24 lignes.
- 1 test existant adapté avec l'accord du principal : `spec099_journal_conserve_dix_mille_caracteres_unicode`. Il déduisait la racine par trois `parent()` de `state_path`. Le nouveau sous-dossier `thread-states-v1` invalide cette hypothèse du TEST. Il lit maintenant `worker.journal_dir`, qui vaut `<racine>/sessions/<agent_id>` (`t3code.rs:1488` et `:1522`). Net : 10 lignes retirées, 1 ajoutée.
- Production (rappel du diff) : +22 / −2, objectif tenu.

<a id="extrait-3"></a>

## Extrait 3

| Contrôle | Résultat |
| --- | --- |
| `--lib spec149_state` (release, offline, 4 jobs, cache chaud, `CARGO_INCREMENTAL=0`, `umask 077`, `TMPDIR=/tmp/b.149t`) | 5 PASS / 0 échec |
| Première série de régression `t3code::tests` | 84 PASS / 1 échec : `spec099_journal_conserve_dix_mille_caracteres_unicode`, `read_dir` NotFound (cause ci-dessus) |
| Dernière exécution, après correction et `rustfmt` : `t3code::tests` | **85 PASS / 0 échec** (1167 filtrés, 10,8 s) |
| `rustfmt --edition 2024` sur une copie : écart avec le fichier | aucun (production et tests) |
| `git diff --check` | OK |

<a id="extrait-4"></a>

## Extrait 4

Je n'ai pas lancé le test global (1840), ni de RED sur l'ancien code, ni de test T3.

<a id="extrait-5"></a>

## Extrait 5

- Droits : le nouveau fichier est écrit par `private_write` (0600), son dossier par `create_private_dir`. L'ancien fichier n'est jamais réécrit.
- Pas de sortie de dossier : le chemin nouveau est un UUID. Le chemin ancien n'est lu qu'avec un nom d'un seul composant, sans `/`, `\`, ni contrôle, de 250 octets au plus. Testé.
- Pas de perte de corrélation : `pending`, `request_id`, `response` et `reply_requested` passent intacts. Le protocole garde l'ID d'origine.
- `uninstall` fait `remove_dir_all(paths.dir)` (`t3code.rs:530`) : le sous-dossier est retiré aussi.

<a id="extrait-6"></a>

## Extrait 6

Pas de RED sur l'ancien code (consigne). Pas de test global ni de test T3. Pas de test avec un vrai T3 ni un vrai service.
