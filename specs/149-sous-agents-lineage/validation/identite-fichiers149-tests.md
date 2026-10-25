# Tests et revue : état de fil à nom borné (complément 149)

Date : 2026-10-11. Rôle : sous-agent de tests et de revue. Aucun fichier de production modifié, aucun Git, aucune base, aucun service, aucun modèle.

## Fichier touché

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/identite-fichiers149-tests.md#extrait-1).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/identite-fichiers149-tests.md#extrait-2).

## Résultats

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/identite-fichiers149-tests.md#extrait-3).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/identite-fichiers149-tests.md#extrait-4).

## Ce que les 5 tests prouvent (vraies écritures privées)

1. `nom_borne_deterministe_et_ecriture_privee` : IDs ASCII de 343 et 341 octets, UTF-8 de 342 octets, `%3A`, `../../etc/passwd`, `/etc/passwd`, antislash. Chaque chemin se trouve dans `threads/thread-states-v1/`, porte exactement 41 octets (`<uuid>.json`), est déterministe et distinct. Chaque ID est sauvegardé pour de vrai (suffixe atomique compris) en 0600 et rechargé à l'identique. Aucun résidu. Rien n'est créé hors de `threads/` (le dossier d'état ne contient que `threads`).
2. `legacy_migre_exactement_puis_ecrit_seulement_le_nouveau` : un ancien état complet (`seen`, `ended_turns`, `pending` avec `request_id`, `attempts`, `response`, `reply_requested`, `anchor_turn_id`, `turn_origins`, `notification_messages`, `seen_activities`, drapeaux) est relu à l'identique. La lecture n'écrit rien. La sauvegarde suivante écrit le nouveau chemin seul, l'ancien fichier reste identique octet pour octet, puis le nouveau prime.
3. `nouveau_prime_et_invalide_ne_ressuscite_pas_legacy` : nouveau fichier présent = prioritaire. Nouveau fichier invalide = état vide, pas l'ancien. Lien symbolique cassé = état vide.
4. `legacy_refuse_chemins_hors_dossier_et_noms_trop_longs` : des fichiers « appât » existent hors de `threads/` (`../outside`, antislash, chemin absolu) et avec des caractères interdits (`\n`, U+202E). Aucun n'est lu. ID vide et ID de 251 octets : état vide. ID de 250 octets : ancien fichier lu (borne incluse, créé par écriture directe car le fichier atomique dépasserait 255 octets).
5. `link_worker_garde_l_id_original_et_ecrit_le_nouveau_chemin` : `LinkWorker::new` garde `thread_id` d'origine (343 octets), `state_path` est le nouveau chemin, un ancien état est migré, toutes les sauvegardes vont au nouveau chemin, aucun ancien fichier n'est créé.

## Revue de production (migration, corrélation, droits)

**APPROVE.** Aucun finding bloquant.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/identite-fichiers149-tests.md#extrait-5).

Limites et points d'attention, non bloquants :

- Retour en arrière de version : un ancien binaire ne lit pas le nouveau fichier. Il reprendrait l'état gelé de l'ancien fichier (curseurs en retard, risque de remises en double). À savoir avant tout rollback.
- Un ancien fichier symlink est suivi à la lecture. Ce comportement existait avant ce correctif.
- Une collision d'UUID v5 (122 bits) est théoriquement possible et ignorée.

## Non vérifié

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/identite-fichiers149-tests.md#extrait-6).
