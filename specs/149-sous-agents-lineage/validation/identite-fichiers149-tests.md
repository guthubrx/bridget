# Tests et revue : état de fil à nom borné (complément 149)

Date : 2026-10-11. Rôle : sous-agent de tests et de revue. Aucun fichier de production modifié, aucun Git, aucune base, aucun service, aucun modèle.

## Fichier touché

`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-correctif-identite/crates/bridget-daemon/src/t3code.rs`, dans le bloc `#[cfg(test)]` seulement.

- 5 tests nouveaux, préfixe `spec149_state`, plus 3 petits helpers : environ 224 lignes après `rustfmt`. La cible « moins de 200 lignes » est dépassée de 24 lignes.
- 1 test existant adapté avec l'accord du principal : `spec099_journal_conserve_dix_mille_caracteres_unicode`. Il déduisait la racine par trois `parent()` de `state_path`. Le nouveau sous-dossier `thread-states-v1` invalide cette hypothèse du TEST. Il lit maintenant `worker.journal_dir`, qui vaut `<racine>/sessions/<agent_id>` (`t3code.rs:1488` et `:1522`). Net : 10 lignes retirées, 1 ajoutée.
- Production (rappel du diff) : +22 / −2, objectif tenu.

## Résultats

| Contrôle | Résultat |
| --- | --- |
| `--lib spec149_state` (release, offline, 4 jobs, cache chaud, `CARGO_INCREMENTAL=0`, `umask 077`, `TMPDIR=/tmp/b.149t`) | 5 PASS / 0 échec |
| Première série de régression `t3code::tests` | 84 PASS / 1 échec : `spec099_journal_conserve_dix_mille_caracteres_unicode`, `read_dir` NotFound (cause ci-dessus) |
| Dernière exécution, après correction et `rustfmt` : `t3code::tests` | **85 PASS / 0 échec** (1167 filtrés, 10,8 s) |
| `rustfmt --edition 2024` sur une copie : écart avec le fichier | aucun (production et tests) |
| `git diff --check` | OK |

Je n'ai pas lancé le test global (1840), ni de RED sur l'ancien code, ni de test T3.

## Ce que les 5 tests prouvent (vraies écritures privées)

1. `nom_borne_deterministe_et_ecriture_privee` : IDs ASCII de 343 et 341 octets, UTF-8 de 342 octets, `%3A`, `../../etc/passwd`, `/etc/passwd`, antislash. Chaque chemin se trouve dans `threads/thread-states-v1/`, porte exactement 41 octets (`<uuid>.json`), est déterministe et distinct. Chaque ID est sauvegardé pour de vrai (suffixe atomique compris) en 0600 et rechargé à l'identique. Aucun résidu. Rien n'est créé hors de `threads/` (le dossier d'état ne contient que `threads`).
2. `legacy_migre_exactement_puis_ecrit_seulement_le_nouveau` : un ancien état complet (`seen`, `ended_turns`, `pending` avec `request_id`, `attempts`, `response`, `reply_requested`, `anchor_turn_id`, `turn_origins`, `notification_messages`, `seen_activities`, drapeaux) est relu à l'identique. La lecture n'écrit rien. La sauvegarde suivante écrit le nouveau chemin seul, l'ancien fichier reste identique octet pour octet, puis le nouveau prime.
3. `nouveau_prime_et_invalide_ne_ressuscite_pas_legacy` : nouveau fichier présent = prioritaire. Nouveau fichier invalide = état vide, pas l'ancien. Lien symbolique cassé = état vide.
4. `legacy_refuse_chemins_hors_dossier_et_noms_trop_longs` : des fichiers « appât » existent hors de `threads/` (`../outside`, antislash, chemin absolu) et avec des caractères interdits (`\n`, U+202E). Aucun n'est lu. ID vide et ID de 251 octets : état vide. ID de 250 octets : ancien fichier lu (borne incluse, créé par écriture directe car le fichier atomique dépasserait 255 octets).
5. `link_worker_garde_l_id_original_et_ecrit_le_nouveau_chemin` : `LinkWorker::new` garde `thread_id` d'origine (343 octets), `state_path` est le nouveau chemin, un ancien état est migré, toutes les sauvegardes vont au nouveau chemin, aucun ancien fichier n'est créé.

## Revue de production (migration, corrélation, droits)

**APPROVE.** Aucun finding bloquant.

- Droits : le nouveau fichier est écrit par `private_write` (0600), son dossier par `create_private_dir`. L'ancien fichier n'est jamais réécrit.
- Pas de sortie de dossier : le chemin nouveau est un UUID. Le chemin ancien n'est lu qu'avec un nom d'un seul composant, sans `/`, `\`, ni contrôle, de 250 octets au plus. Testé.
- Pas de perte de corrélation : `pending`, `request_id`, `response` et `reply_requested` passent intacts. Le protocole garde l'ID d'origine.
- `uninstall` fait `remove_dir_all(paths.dir)` (`t3code.rs:530`) : le sous-dossier est retiré aussi.

Limites et points d'attention, non bloquants :

- Retour en arrière de version : un ancien binaire ne lit pas le nouveau fichier. Il reprendrait l'état gelé de l'ancien fichier (curseurs en retard, risque de remises en double). À savoir avant tout rollback.
- Un ancien fichier symlink est suivi à la lecture. Ce comportement existait avant ce correctif.
- Une collision d'UUID v5 (122 bits) est théoriquement possible et ignorée.

## Non vérifié

Pas de RED sur l'ancien code (consigne). Pas de test global ni de test T3. Pas de test avec un vrai T3 ni un vrai service.
