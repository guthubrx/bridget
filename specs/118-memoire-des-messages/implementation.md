# Journal 118 — Un grand fil T3 n'est recopié qu'une fois

- **Base** : main `f165f47c` — **Date** : 2026-09-25 — **Statut** : Implemented, livré le 2026-09-25 03:36

## Diagnostic (données réelles)
- Pont : `bridget-observation` remis à `sol_city_ai` (e86d89f9) toutes les ~6 s ; `journal du fil
  106521cc : journal ACP saturé` toutes les ~3 s depuis 00:13 UTC.
- Catalogue des sources : `ea69b65b` (opus-city-glmkrkr4) bascule entre deux événements et aucun.
- Journal `ea69b65b/2026-09-25.jsonl` : 146 644 lignes, 1 363 identifiants distincts, jusqu'à 131
  copies chacun ; 336 Mo avec la veille. État du fil : `seen` plein à SEEN_BOUND (1 000).
- Mécanisme : éviction des plus anciens sous le nombre de messages visibles → recopie à chaque
  lecture → file d'écriture (256) pleine → journal jamais rattrapé. `refresh` annonçait la source
  prête avant la projection, puis indisponible après : deux notifications par lecture.
- L'instrumentation 117 a bien nommé le motif (« journal non rattrapé »).

## Correction (`t3code.rs`)
- `ThreadState::forget_invisible` : au-delà de SEEN_BOUND, n'oublie que les identifiants que T3 ne
  montre plus (messages, et faits d'activité par leur activité) ; `remember` n'évince plus.
- Index `HashSet` des messages recopiés dans `project_journal` : O(S + M) au lieu de O(S × M).
- Annonce « prête » anticipée gardée pour les seules lectures qui portent une lacune : le daemon
  ignore la lacune d'une source non déclarée (test `spec101_http_capacites_honnetes…`).

## Écarté
- `glmwrk1`, `glmwrk2`, `opus_city_ai` sans événement déclaré : dernier tour sans origine prouvée
  (fils Claude, réveils d'arrière-plan), comportement voulu de la 116 ; journaux petits.
- Journal de 336 Mo de `ea69b65b` : laissé en place, suppression à la décision de l'utilisateur.

## Vérifications
- `spec118_oubli_limite_a_ce_que_t3_ne_montre_plus`, `spec118_un_grand_fil_n_est_recopie_qu_une_fois`,
  `spec118_journal_en_retard_une_seule_annonce_sans_alternance` ; les deux derniers échouent sur
  l'ancien code (le dernier rend exactement `[prête, vide, prête, vide]`).
- fmt OK ; clippy `-D warnings` OK ; recette complète : **1550 réussis, 0 échec, 52 ignorés**.

## Livraison
- 03:36 : fusion `e63a7abd`, construction, relance du pont seul (`kickstart -k com.bridget.t3`).
- Après relance : `seen` du fil = 1 363, relus depuis le journal existant, aucune recopie ; journal
  +0 octet en 2 min ; source `ea69b65b` stable avec ses deux événements ; deux notifications à la
  relance elle-même, plus aucune ensuite.
